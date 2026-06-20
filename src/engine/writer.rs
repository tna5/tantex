use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use std::time::Duration;

use crossbeam_channel::Receiver;
use rayon::prelude::*;
use serde::de::{self, DeserializeSeed, Deserializer, MapAccess, Visitor};
use tantivy::merge_policy::MergePolicy;
use tantivy::query::QueryParser;
use tantivy::schema::{Field, FieldType, OwnedValue, Schema};
use tantivy::{IndexWriter, TantivyDocument};

use crate::shm::buffer::ShmBuffer;

/// Lightweight field descriptor pre-computed once per index, used in the hot parsing loop.
#[derive(Clone)]
enum FieldKind {
    Text,
    U64,
    I64,
    F64,
    Date,
    Bool,
    Bytes,
    Json,
    Ip,
}

/// Pre-computed map: field name → (Field id, FieldKind).
type FieldCache = HashMap<String, (Field, FieldKind)>;

fn build_field_cache(schema: &Schema, field_map: &HashMap<String, Field>) -> FieldCache {
    field_map
        .iter()
        .filter_map(|(name, &field)| {
            let kind = match schema.get_field_entry(field).field_type() {
                FieldType::Str(_) => FieldKind::Text,
                FieldType::U64(_) => FieldKind::U64,
                FieldType::I64(_) => FieldKind::I64,
                FieldType::F64(_) => FieldKind::F64,
                FieldType::Date(_) => FieldKind::Date,
                FieldType::Bool(_) => FieldKind::Bool,
                FieldType::Bytes(_) => FieldKind::Bytes,
                FieldType::JsonObject(_) => FieldKind::Json,
                FieldType::IpAddr(_) => FieldKind::Ip,
                _ => return None,
            };
            Some((name.clone(), (field, kind)))
        })
        .collect()
}

/// Configuration for the writer thread.
#[derive(Debug, Clone)]
pub struct WriterConfig {
    pub heap_size: usize,
    pub num_threads: usize,
    pub auto_commit_doc_count: usize,
    pub auto_commit_interval_secs: u64,
    pub merge_target_docs: usize,
    pub max_merge_factor: usize,
    pub min_num_segments: usize,
    /// Percentage of num_threads given to tantivy index threads (0-100). Rest = rayon parse.
    pub index_threads_pct: u32,
    /// Multiplier applied to auto_commit_doc_count for the hard-limit commit.
    pub hard_commit_multiplier: u32,
}

/// Commands sent to the writer thread.
pub enum WriterCommand {
    AddDocuments {
        documents: Vec<serde_json::Value>,
        response: tokio::sync::oneshot::Sender<Result<u64, String>>,
    },
    AddDocumentsFromShm {
        shm_data: Vec<u8>,
        doc_count: u32,
        response: tokio::sync::oneshot::Sender<Result<u64, String>>,
    },
    AddDocumentsFromShmRef {
        shm: Arc<ShmBuffer>,
        length: usize,
        doc_count: u32,
        response: tokio::sync::oneshot::Sender<Result<u64, String>>,
    },
    Commit {
        response: tokio::sync::oneshot::Sender<Result<(), String>>,
    },
    DeleteByQuery {
        query_str: String,
        response: tokio::sync::oneshot::Sender<Result<(), String>>,
    },
    SetMergePolicy {
        target_docs: usize,
        max_factor: usize,
        min_segments: usize,
        response: tokio::sync::oneshot::Sender<Result<(), String>>,
    },
    Shutdown,
}

/// Handle to the writer thread; holds the sender and join handle.
pub struct WriterHandle {
    pub sender: crossbeam_channel::Sender<WriterCommand>,
    pub pending_docs: Arc<AtomicU64>,
    pub total_docs_ingested: Arc<AtomicU64>,
    pub raw_bytes_ingested: Arc<AtomicU64>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl WriterHandle {
    pub fn spawn(
        index: tantivy::Index,
        schema: Schema,
        field_map: HashMap<String, Field>,
        config: WriterConfig,
    ) -> Self {
        let (tx, rx) = crossbeam_channel::unbounded();
        let pending_docs = Arc::new(AtomicU64::new(0));
        let total_docs_ingested = Arc::new(AtomicU64::new(0));
        let raw_bytes_ingested = Arc::new(AtomicU64::new(0));
        let counter = Arc::clone(&pending_docs);
        let ingested = Arc::clone(&total_docs_ingested);
        let raw_bytes = Arc::clone(&raw_bytes_ingested);

        let thread = std::thread::spawn(move || {
            writer_loop(index, schema, field_map, config, rx, None, counter, ingested, raw_bytes);
        });

        Self {
            sender: tx,
            pending_docs,
            total_docs_ingested,
            raw_bytes_ingested,
            thread: Some(thread),
        }
    }

    pub fn spawn_with_merge_policy<M: MergePolicy + 'static>(
        index: tantivy::Index,
        schema: Schema,
        field_map: HashMap<String, Field>,
        config: WriterConfig,
        merge_policy: M,
    ) -> Self {
        let (tx, rx) = crossbeam_channel::unbounded();
        let pending_docs = Arc::new(AtomicU64::new(0));
        let total_docs_ingested = Arc::new(AtomicU64::new(0));
        let raw_bytes_ingested = Arc::new(AtomicU64::new(0));
        let counter = Arc::clone(&pending_docs);
        let ingested = Arc::clone(&total_docs_ingested);
        let raw_bytes = Arc::clone(&raw_bytes_ingested);
        let boxed_policy: Box<dyn MergePolicy> = Box::new(merge_policy);

        let thread = std::thread::spawn(move || {
            writer_loop(index, schema, field_map, config, rx, Some(boxed_policy), counter, ingested, raw_bytes);
        });

        Self {
            sender: tx,
            pending_docs,
            total_docs_ingested,
            raw_bytes_ingested,
            thread: Some(thread),
        }
    }

    pub fn send(&self, cmd: WriterCommand) {
        self.sender.send(cmd).expect("Writer thread died");
    }

    pub fn shutdown(&mut self) {
        let _ = self.sender.send(WriterCommand::Shutdown);
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
    }
}

/// Fill one field of `doc` from a serde_json::Value, with the same
/// type-coercion rules as the legacy code (forgiving: silently skips
/// values that don't fit the field kind).
fn fill_field_from_value(
    doc: &mut TantivyDocument,
    field: Field,
    kind: &FieldKind,
    val: &serde_json::Value,
) {
    if let serde_json::Value::Array(arr) = val {
        for item in arr {
            fill_field_from_value(doc, field, kind, item);
        }
        return;
    }
    match kind {
        FieldKind::Text => {
            if let Some(s) = val.as_str() {
                doc.add_field_value(field, s);
            } else {
                let s = val.to_string();
                doc.add_field_value(field, s.as_str());
            }
        }
        FieldKind::U64 => {
            if let Some(n) = val.as_u64() {
                doc.add_field_value(field, n);
            } else if let Some(n) = val.as_i64() {
                doc.add_field_value(field, n as u64);
            } else if let Some(n) = val.as_f64() {
                doc.add_field_value(field, n as u64);
            }
        }
        FieldKind::I64 => {
            if let Some(n) = val.as_i64() {
                doc.add_field_value(field, n);
            } else if let Some(n) = val.as_u64() {
                doc.add_field_value(field, n as i64);
            } else if let Some(n) = val.as_f64() {
                doc.add_field_value(field, n as i64);
            }
        }
        FieldKind::F64 => {
            if let Some(n) = val.as_f64() {
                doc.add_field_value(field, n);
            } else if let Some(n) = val.as_i64() {
                doc.add_field_value(field, n as f64);
            } else if let Some(n) = val.as_u64() {
                doc.add_field_value(field, n as f64);
            }
        }
        FieldKind::Date => {
            if let Some(s) = val.as_str() {
                if let Ok(odt) = tantivy::time::OffsetDateTime::parse(
                    s,
                    &tantivy::time::format_description::well_known::Rfc3339,
                ) {
                    doc.add_field_value(field, tantivy::DateTime::from_utc(odt));
                }
            } else if let Some(n) = val.as_i64() {
                doc.add_field_value(field, tantivy::DateTime::from_timestamp_secs(n));
            }
        }
        FieldKind::Bool => {
            if let Some(b) = val.as_bool() {
                doc.add_field_value(field, b);
            }
        }
        FieldKind::Bytes => {
            if let Some(s) = val.as_str() {
                doc.add_field_value(field, s.as_bytes().to_vec());
            }
        }
        FieldKind::Json => {
            doc.add_field_value(field, serde_json_to_owned_value(val));
        }
        FieldKind::Ip => {
            use std::net::IpAddr;
            use tantivy::schema::IntoIpv6Addr;
            if let Some(s) = val.as_str() {
                if let Ok(ip) = s.parse::<IpAddr>() {
                    doc.add_field_value(field, ip.into_ipv6_addr());
                }
            }
        }
    }
}

/// AddDocuments path: a serde_json::Value is already in hand.
fn json_to_document(
    value: &serde_json::Value,
    field_cache: &FieldCache,
) -> Result<TantivyDocument, String> {
    let obj = value.as_object().ok_or("Document must be a JSON object")?;
    let mut doc = TantivyDocument::new();
    for (key, val) in obj {
        if let Some((field, kind)) = field_cache.get(key) {
            fill_field_from_value(&mut doc, *field, kind, val);
        }
    }
    Ok(doc)
}

/// Streaming-JSON path used by SHM ingestion.
/// Walks the top-level JSON object once, writing each known field directly
/// into a `TantivyDocument` without materializing the intermediate
/// `Map<String, Value>`. Text fields borrow from the input buffer where
/// possible (no escape sequences) — huge alloc win for big body strings.
struct DocSeed<'a> {
    field_cache: &'a FieldCache,
}

impl<'de, 'a> DeserializeSeed<'de> for DocSeed<'a> {
    type Value = TantivyDocument;
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(DocVisitor {
            field_cache: self.field_cache,
        })
    }
}

struct DocVisitor<'a> {
    field_cache: &'a FieldCache,
}

impl<'de, 'a> Visitor<'de> for DocVisitor<'a> {
    type Value = TantivyDocument;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a JSON object representing a document")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut doc = TantivyDocument::new();
        while let Some(key) = map.next_key::<Cow<str>>()? {
            match self.field_cache.get(key.as_ref()) {
                Some((field, kind)) => {
                    // General path: parse as a serde_json Value so that
                    // fill_field_from_value can handle arrays, Ip, and all
                    // other types (including text) uniformly.
                    let val: serde_json::Value = map.next_value()?;
                    fill_field_from_value(&mut doc, *field, kind, &val);
                }
                None => {
                    // Unknown field — skip value entirely without allocating.
                    let _ = map.next_value::<de::IgnoredAny>()?;
                }
            }
        }
        Ok(doc)
    }
}

/// Convert a serde_json::Value to a tantivy OwnedValue.
fn serde_json_to_owned_value(val: &serde_json::Value) -> OwnedValue {
    match val {
        serde_json::Value::Null => OwnedValue::Null,
        serde_json::Value::Bool(b) => OwnedValue::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                OwnedValue::I64(i)
            } else if let Some(u) = n.as_u64() {
                OwnedValue::U64(u)
            } else if let Some(f) = n.as_f64() {
                OwnedValue::F64(f)
            } else {
                OwnedValue::Null
            }
        }
        serde_json::Value::String(s) => OwnedValue::Str(s.clone()),
        serde_json::Value::Array(arr) => {
            OwnedValue::Array(arr.iter().map(serde_json_to_owned_value).collect())
        }
        serde_json::Value::Object(map) => {
            let entries: std::collections::BTreeMap<String, OwnedValue> = map
                .iter()
                .map(|(k, v)| (k.clone(), serde_json_to_owned_value(v)))
                .collect();
            OwnedValue::Object(entries)
        }
    }
}

/// Parse one NDJSON line straight into a TantivyDocument, then index it.
/// Returns (indexed, errors). Uses sonic-rs (SIMD JSON) — ~2× faster than
/// serde_json on Apple Silicon NEON for the small object shapes we ingest.
#[inline]
fn parse_and_index_line(
    line: &str,
    writer: &IndexWriter,
    field_cache: &FieldCache,
) -> (u64, u64) {
    let line = line.trim();
    if line.is_empty() {
        return (0, 0);
    }
    let mut de = sonic_rs::Deserializer::from_str(line);
    let doc = match (DocSeed { field_cache }).deserialize(&mut de) {
        Ok(d) => d,
        Err(_) => return (0, 1),
    };
    match writer.add_document(doc) {
        Ok(_) => (1, 0),
        Err(_) => (0, 1),
    }
}

/// Parallel parse + parallel add_document for an NDJSON blob.
fn ingest_ndjson(
    text: &str,
    writer: &IndexWriter,
    field_cache: &FieldCache,
    rayon_pool: &rayon::ThreadPool,
) -> (u64, u64) {
    rayon_pool.install(|| {
        text.par_lines()
            .map(|line| parse_and_index_line(line, writer, field_cache))
            .reduce(|| (0u64, 0u64), |a, b| (a.0 + b.0, a.1 + b.1))
    })
}

/// Parallel index for an already-parsed Vec<Value>.
fn ingest_values(
    documents: Vec<serde_json::Value>,
    writer: &IndexWriter,
    field_cache: &FieldCache,
    rayon_pool: &rayon::ThreadPool,
) -> (u64, u64) {
    rayon_pool.install(|| {
        documents
            .into_par_iter()
            .map(|val| match json_to_document(&val, field_cache) {
                Ok(doc) => match writer.add_document(doc) {
                    Ok(_) => (1u64, 0u64),
                    Err(_) => (0, 1),
                },
                Err(_) => (0, 1),
            })
            .reduce(|| (0u64, 0u64), |a, b| (a.0 + b.0, a.1 + b.1))
    })
}

fn writer_loop(
    index: tantivy::Index,
    schema: Schema,
    field_map: HashMap<String, Field>,
    config: WriterConfig,
    rx: Receiver<WriterCommand>,
    merge_policy: Option<Box<dyn MergePolicy>>,
    pending_docs: Arc<AtomicU64>,
    total_docs_ingested: Arc<AtomicU64>,
    raw_bytes_ingested: Arc<AtomicU64>,
) {
    use crate::engine::merge_policy::TargetDocCountMergePolicy;

    // Split the thread budget between tantivy's internal indexing pool and
    // the rayon pool used for parse + add_document fan-out. The split ratio
    // is exposed in config (index_threads_pct).
    let total_threads = config.num_threads.max(2);
    let pct = config.index_threads_pct.clamp(10, 90) as usize;
    let index_threads = ((total_threads * pct) / 100).max(2);
    let parse_threads = (total_threads.saturating_sub(index_threads)).max(2);

    let writer: IndexWriter = if index_threads > 1 {
        index
            .writer_with_num_threads(index_threads, config.heap_size)
            .expect("Failed to create IndexWriter")
    } else {
        index
            .writer(config.heap_size)
            .expect("Failed to create IndexWriter")
    };

    // Set the custom merge policy upfront — merges run on tantivy's background threads.
    if merge_policy.is_some() {
        writer.set_merge_policy(Box::new(TargetDocCountMergePolicy {
            target_num_docs: config.merge_target_docs,
            max_merge_factor: config.max_merge_factor,
            min_num_segments: config.min_num_segments.max(2),
        }));
    }

    // Pre-compute field cache once.
    let field_cache = build_field_cache(&schema, &field_map);

    // Dedicated rayon pool sized to the parse half of the thread budget.
    let rayon_pool = rayon::ThreadPoolBuilder::new()
        .num_threads(parse_threads)
        .thread_name(|i| format!("tant2-parse-{}", i))
        .build()
        .expect("Failed to create rayon thread pool");

    log::info!(
        "Writer started: {} tantivy index threads + {} rayon parse threads",
        index_threads,
        parse_threads
    );

    // `writer` is borrowed mutably only for commit / wait_merging_threads.
    // add_document takes &self, so the rayon fan-out below uses &writer safely.
    let mut writer = writer;

    let timeout = Duration::from_secs(config.auto_commit_interval_secs);
    let mut docs_since_commit: usize = 0;
    let mut errors_since_commit: u64 = 0;

    // Throttled progress logging: emit one line every ≥5 s of active ingest,
    // showing slice rate since the last log. No spam for tiny batches, but a
    // steady heartbeat under load.
    let mut last_progress = std::time::Instant::now();
    let mut last_progress_total: u64 = 0;
    let progress_interval = Duration::from_secs(5);

    // Helper closure to decide & run auto-commit. With NRT non-critical and
    // big hard limits, this very rarely fires inside the ingest path —
    // the timer branch below usually wins.
    let should_auto_commit = |docs_since_commit: usize, rx_empty: bool, cfg: &WriterConfig| {
        let mult = cfg.hard_commit_multiplier.max(1) as usize;
        let hard_limit = cfg.auto_commit_doc_count.saturating_mul(mult);
        docs_since_commit >= hard_limit
            || (docs_since_commit >= cfg.auto_commit_doc_count && rx_empty)
    };

    // Helper to throttle progress logs. Logs at most once per `progress_interval`.
    fn maybe_log_progress(
        total: u64,
        last_total: &mut u64,
        last_time: &mut std::time::Instant,
        interval: Duration,
    ) {
        let elapsed = last_time.elapsed();
        if elapsed < interval {
            return;
        }
        let delta = total.saturating_sub(*last_total);
        if delta == 0 {
            return;
        }
        let rate = (delta as f64 / elapsed.as_secs_f64()) as u64;
        log::info!(
            "ingest: {} docs total (+{} in {:.1}s, {} docs/sec)",
            format_with_underscores(total),
            format_with_underscores(delta),
            elapsed.as_secs_f64(),
            format_with_underscores(rate)
        );
        *last_total = total;
        *last_time = std::time::Instant::now();
    }

    fn format_with_underscores(n: u64) -> String {
        let s = n.to_string();
        let bytes = s.as_bytes();
        let mut out = String::with_capacity(bytes.len() + bytes.len() / 3);
        for (i, b) in bytes.iter().enumerate() {
            if i > 0 && (bytes.len() - i) % 3 == 0 { out.push('_'); }
            out.push(*b as char);
        }
        out
    }

    loop {
        match rx.recv_timeout(timeout) {
            Ok(WriterCommand::AddDocuments {
                documents,
                response,
            }) => {
                let raw_bytes: u64 = documents.iter()
                    .map(|d| d.to_string().len() as u64 + 1)
                    .sum();
                let (indexed, errors) = ingest_values(documents, &writer, &field_cache, &rayon_pool);
                raw_bytes_ingested.fetch_add(raw_bytes, Ordering::Relaxed);
                pending_docs.fetch_add(indexed, Ordering::Relaxed);
                let new_total = total_docs_ingested.fetch_add(indexed, Ordering::Relaxed) + indexed;
                docs_since_commit += indexed as usize;
                errors_since_commit += errors;
                // Ack immediately so the client can pipeline the next batch.
                let _ = response.send(Ok(indexed));
                maybe_log_progress(new_total, &mut last_progress_total, &mut last_progress, progress_interval);

                if should_auto_commit(docs_since_commit, rx.is_empty(), &config) {
                    if let Err(e) = writer.commit() {
                        log::error!("Auto-commit failed: {}", e);
                    } else {
                        log::info!(
                            "Auto-committed after {} docs ({} errors)",
                            docs_since_commit,
                            errors_since_commit
                        );
                        docs_since_commit = 0;
                        errors_since_commit = 0;
                        pending_docs.store(0, Ordering::Relaxed);
                    }
                }
            }
            Ok(WriterCommand::AddDocumentsFromShm {
                shm_data,
                doc_count: _,
                response,
            }) => {
                raw_bytes_ingested.fetch_add(shm_data.len() as u64, Ordering::Relaxed);
                // SAFETY: content written by JS JSON.stringify — always valid UTF-8.
                let text = unsafe { std::str::from_utf8_unchecked(&shm_data) };
                let (indexed, errors) = ingest_ndjson(text, &writer, &field_cache, &rayon_pool);
                pending_docs.fetch_add(indexed, Ordering::Relaxed);
                let new_total = total_docs_ingested.fetch_add(indexed, Ordering::Relaxed) + indexed;
                docs_since_commit += indexed as usize;
                errors_since_commit += errors;
                let _ = response.send(Ok(indexed));
                maybe_log_progress(new_total, &mut last_progress_total, &mut last_progress, progress_interval);

                if should_auto_commit(docs_since_commit, rx.is_empty(), &config) {
                    if let Err(e) = writer.commit() {
                        log::error!("Auto-commit failed: {}", e);
                    } else {
                        log::info!(
                            "Auto-committed after {} docs ({} errors)",
                            docs_since_commit,
                            errors_since_commit
                        );
                        docs_since_commit = 0;
                        errors_since_commit = 0;
                        pending_docs.store(0, Ordering::Relaxed);
                    }
                }
            }
            Ok(WriterCommand::AddDocumentsFromShmRef {
                shm,
                length,
                doc_count: _,
                response,
            }) => {
                raw_bytes_ingested.fetch_add(length as u64, Ordering::Relaxed);
                let data = shm.read_slice(0, length);
                // SAFETY: content written by JS JSON.stringify — always valid UTF-8.
                let text = unsafe { std::str::from_utf8_unchecked(data) };
                let (indexed, errors) = ingest_ndjson(text, &writer, &field_cache, &rayon_pool);
                pending_docs.fetch_add(indexed, Ordering::Relaxed);
                let new_total = total_docs_ingested.fetch_add(indexed, Ordering::Relaxed) + indexed;
                docs_since_commit += indexed as usize;
                errors_since_commit += errors;
                let _ = response.send(Ok(indexed));
                maybe_log_progress(new_total, &mut last_progress_total, &mut last_progress, progress_interval);

                if should_auto_commit(docs_since_commit, rx.is_empty(), &config) {
                    if let Err(e) = writer.commit() {
                        log::error!("Auto-commit failed: {}", e);
                    } else {
                        log::info!(
                            "Auto-committed after {} docs ({} errors)",
                            docs_since_commit,
                            errors_since_commit
                        );
                        docs_since_commit = 0;
                        errors_since_commit = 0;
                        pending_docs.store(0, Ordering::Relaxed);
                    }
                }
            }
            Ok(WriterCommand::Commit { response }) => match writer.commit() {
                Ok(_) => {
                    log::info!(
                        "Explicit commit ({} docs, {} errors)",
                        docs_since_commit,
                        errors_since_commit
                    );
                    docs_since_commit = 0;
                    errors_since_commit = 0;
                    pending_docs.store(0, Ordering::Relaxed);
                    let _ = response.send(Ok(()));
                }
                Err(e) => {
                    let _ = response.send(Err(format!("Commit failed: {}", e)));
                }
            },
            Ok(WriterCommand::DeleteByQuery { query_str, response }) => {
                let default_fields: Vec<tantivy::schema::Field> = field_cache
                    .values()
                    .filter_map(|(field, kind)| matches!(kind, FieldKind::Text).then_some(*field))
                    .collect();
                let query_parser = QueryParser::for_index(&index, default_fields);
                match query_parser.parse_query(&query_str) {
                    Ok(query) => match writer.delete_query(Box::new(query)) {
                        Ok(_) => match writer.commit() {
                            Ok(_) => {
                                log::info!("Delete-by-query committed: {:?}", query_str);
                                docs_since_commit = 0;
                                errors_since_commit = 0;
                                pending_docs.store(0, Ordering::Relaxed);
                                let _ = response.send(Ok(()));
                            }
                            Err(e) => {
                                let _ = response.send(Err(format!("Commit after delete failed: {}", e)));
                            }
                        },
                        Err(e) => {
                            let _ = response.send(Err(format!("Delete query failed: {}", e)));
                        }
                    },
                    Err(e) => {
                        let _ = response.send(Err(format!("Invalid query: {}", e)));
                    }
                }
            }

            Ok(WriterCommand::SetMergePolicy { target_docs, max_factor, min_segments, response }) => {
                use crate::engine::merge_policy::TargetDocCountMergePolicy;
                writer.set_merge_policy(Box::new(TargetDocCountMergePolicy {
                    target_num_docs: target_docs,
                    max_merge_factor: max_factor,
                    min_num_segments: min_segments.max(2),
                }));
                log::info!(
                    "Merge policy updated: target_docs={}, max_factor={}, min_segments={}",
                    target_docs, max_factor, min_segments
                );
                let _ = response.send(Ok(()));
            }

            Ok(WriterCommand::Shutdown) => {
                if docs_since_commit > 0 {
                    if let Err(e) = writer.commit() {
                        log::error!("Final commit on shutdown failed: {}", e);
                    } else {
                        log::info!(
                            "Final commit on shutdown ({} docs, {} errors)",
                            docs_since_commit,
                            errors_since_commit
                        );
                        pending_docs.store(0, Ordering::Relaxed);
                    }
                }
                // Skip wait_merging_threads() — it blocks until all background
                // merges finish, which can take minutes under heavy load.
                // IndexWriter::Drop calls segment_updater.kill() to signal
                // merge threads to abort; tantivy rolls back incomplete merges
                // safely on next startup.
                break;
            }
            Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                if docs_since_commit > 0 {
                    if let Err(e) = writer.commit() {
                        log::error!("Timer auto-commit failed: {}", e);
                    } else {
                        log::info!(
                            "Timer auto-commit ({} docs, {} errors)",
                            docs_since_commit,
                            errors_since_commit
                        );
                        docs_since_commit = 0;
                        errors_since_commit = 0;
                        pending_docs.store(0, Ordering::Relaxed);
                    }
                }
            }
            Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                log::info!("Writer channel disconnected, shutting down");
                if docs_since_commit > 0 {
                    let _ = writer.commit();
                }
                break;
            }
        }
    }
}
