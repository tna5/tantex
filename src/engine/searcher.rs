use std::collections::HashMap;
use std::sync::{Arc, atomic::{AtomicU64, Ordering}};
use std::time::Instant;

use tantivy::collector::{Count, TopDocs};
use tantivy::query::QueryParser;
use tantivy::schema::{Field, FieldType, OwnedValue, Schema};
use tantivy::{DocAddress, IndexReader, ReloadPolicy, TantivyDocument};

use crate::engine::query::rewrite_query;
use crate::protocol::messages::{SearchHit, SearchResponse, SegmentInfo};

pub struct SearchEngine {
    reader: IndexReader,
    schema: Schema,
    field_map: HashMap<String, Field>,
    subfield_routes: HashMap<String, Field>,
    pub search_count: Arc<AtomicU64>,
    pub search_latency_us: Arc<AtomicU64>,
}

impl SearchEngine {
    pub fn new(
        index: &tantivy::Index,
        schema: Schema,
        field_map: HashMap<String, Field>,
        subfield_routes: HashMap<String, Field>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()?;
        Ok(Self {
            reader,
            schema,
            field_map,
            subfield_routes,
            search_count: Arc::new(AtomicU64::new(0)),
            search_latency_us: Arc::new(AtomicU64::new(0)),
        })
    }

    pub fn search(
        &self,
        query_str: &str,
        limit: usize,
        offset: usize,
    ) -> Result<SearchResponse, Box<dyn std::error::Error>> {
        let t0 = Instant::now();
        let searcher = self.reader.searcher();

        // Rewrite selector path queries to internal sub-field names.
        let rewritten = rewrite_query(query_str, &self.subfield_routes);

        // Default fields: text fields only, excluding internal sub-fields.
        // Users who want to search a specific selector use the `selectors.X:` prefix
        // which gets rewritten above.
        let default_fields: Vec<Field> = self
            .field_map
            .values()
            .filter(|f| {
                let entry = self.schema.get_field_entry(**f);
                !entry.name().starts_with("__sub__")
                    && matches!(entry.field_type(), FieldType::Str(_))
            })
            .copied()
            .collect();

        let query_parser = QueryParser::for_index(searcher.index(), default_fields);
        let query = query_parser.parse_query(&rewritten)?;

        let (top_docs, total_count) =
            searcher.search(&query, &(TopDocs::with_limit(limit + offset), Count))?;

        let hits: Vec<SearchHit> = top_docs
            .into_iter()
            .skip(offset)
            .map(|(score, doc_address)| {
                let doc = self.retrieve_doc(&searcher, doc_address);
                SearchHit { score, doc }
            })
            .collect();

        let elapsed_us = t0.elapsed().as_micros() as u64;
        self.search_count.fetch_add(1, Ordering::Relaxed);
        self.search_latency_us.fetch_add(elapsed_us, Ordering::Relaxed);

        Ok(SearchResponse {
            total_hits: total_count as u64,
            hits,
            elapsed_us,
        })
    }

    fn retrieve_doc(
        &self,
        searcher: &tantivy::Searcher,
        doc_address: DocAddress,
    ) -> serde_json::Value {
        match searcher.doc::<TantivyDocument>(doc_address) {
            Ok(doc) => self.document_to_json(&doc),
            Err(e) => {
                log::error!("Failed to retrieve doc: {}", e);
                serde_json::Value::Null
            }
        }
    }

    fn document_to_json(&self, doc: &TantivyDocument) -> serde_json::Value {
        let mut map = serde_json::Map::new();

        for (field, field_entry) in self.schema.fields() {
            if !field_entry.is_stored() {
                continue;
            }
            let name = field_entry.name().to_string();
            // Internal sub-fields are not stored, so this check is redundant,
            // but guard anyway to keep the response clean.
            if name.starts_with("__sub__") {
                continue;
            }

            let values: Vec<&OwnedValue> = doc.get_all(field).collect();

            if values.is_empty() {
                continue;
            }

            if values.len() == 1 {
                map.insert(name, owned_value_to_json(values[0]));
            } else {
                let arr: Vec<serde_json::Value> =
                    values.iter().map(|v| owned_value_to_json(v)).collect();
                map.insert(name, serde_json::Value::Array(arr));
            }
        }

        serde_json::Value::Object(map)
    }

    pub fn num_docs(&self) -> u64 {
        self.reader.searcher().num_docs()
    }

    pub fn num_segments(&self) -> usize {
        self.reader.searcher().segment_readers().len()
    }

    pub fn segment_info(&self) -> Result<Vec<SegmentInfo>, Box<dyn std::error::Error>> {
        let searcher = self.reader.searcher();
        let mut segments = Vec::new();

        for seg_reader in searcher.segment_readers() {
            let segment_id = seg_reader.segment_id().uuid_string();
            let num_docs = seg_reader.num_docs();
            let num_deleted_docs = seg_reader.num_deleted_docs();
            let size_bytes = match seg_reader.space_usage() {
                Ok(usage) => usage.total().get_bytes(),
                Err(_) => 0,
            };

            segments.push(SegmentInfo {
                segment_id,
                num_docs,
                num_deleted_docs,
                size_bytes,
            });
        }

        Ok(segments)
    }
}

fn owned_value_to_json(val: &OwnedValue) -> serde_json::Value {
    match val {
        OwnedValue::Null => serde_json::Value::Null,
        OwnedValue::Str(s) => serde_json::Value::String(s.clone()),
        OwnedValue::U64(n) => serde_json::json!(*n),
        OwnedValue::I64(n) => serde_json::json!(*n),
        OwnedValue::F64(n) => serde_json::json!(*n),
        OwnedValue::Bool(b) => serde_json::Value::Bool(*b),
        OwnedValue::Date(d) => {
            serde_json::Value::String(d.into_utc().to_string())
        }
        OwnedValue::Bytes(b) => {
            serde_json::Value::Array(b.iter().map(|byte| serde_json::json!(*byte)).collect())
        }
        OwnedValue::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(owned_value_to_json).collect())
        }
        OwnedValue::Object(entries) => {
            let mut map = serde_json::Map::new();
            for (k, v) in entries {
                map.insert(k.clone(), owned_value_to_json(v));
            }
            serde_json::Value::Object(map)
        }
        _ => serde_json::Value::Null,
    }
}
