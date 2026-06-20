use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use axum::response::sse::Event;
use futures::stream::{self, Stream};
use serde_json::json;
use std::convert::Infallible;
use sysinfo::System;
use tokio::sync::RwLock;

use crate::engine::index_manager::IndexManager;

struct SseState {
    prev_ingested: HashMap<String, u64>,
    prev_search_count: HashMap<String, u64>,
    sys: System,
}

impl SseState {
    fn new() -> Self {
        let mut sys = System::new();
        sys.refresh_memory();
        Self {
            prev_ingested: HashMap::new(),
            prev_search_count: HashMap::new(),
            sys,
        }
    }
}

pub fn metrics_stream(
    index_manager: Arc<RwLock<IndexManager>>,
) -> impl Stream<Item = Result<Event, Infallible>> {
    stream::unfold(
        (index_manager, SseState::new()),
        |(mgr, mut state)| async move {
            tokio::time::sleep(Duration::from_secs(1)).await;

            let indexes = {
                let locked = mgr.read().await;
                locked.list_indexes().indexes
            };

            let total_docs: u64 = indexes.iter().map(|i| i.doc_count).sum();
            let total_segments: usize = indexes.iter().map(|i| i.num_segments).sum();
            let total_pending: u64 = indexes.iter().map(|i| i.pending_docs).sum();

            let mut total_ingest_rate: u64 = 0;
            let mut total_search_rate: f64 = 0.0;

            let per_index: Vec<serde_json::Value> = indexes.iter().map(|idx| {
                let prev_ing = state.prev_ingested
                    .get(&idx.name).copied()
                    .unwrap_or(idx.total_docs_ingested);
                let ingest_rate = idx.total_docs_ingested.saturating_sub(prev_ing);
                total_ingest_rate += ingest_rate;

                let prev_sc = state.prev_search_count
                    .get(&idx.name).copied()
                    .unwrap_or(idx.search_count);
                let search_rate = idx.search_count.saturating_sub(prev_sc) as f64;
                total_search_rate += search_rate;

                json!({
                    "name":                idx.name,
                    "doc_count":           idx.doc_count,
                    "num_segments":        idx.num_segments,
                    "pending_docs":        idx.pending_docs,
                    "total_docs_ingested": idx.total_docs_ingested,
                    "raw_bytes_ingested":  idx.raw_bytes_ingested,
                    "ingest_rate":         ingest_rate,
                    "search_rate":         search_rate,
                    "search_count":        idx.search_count,
                    "search_latency_us":   idx.search_latency_us,
                })
            }).collect();

            // Update per-index state
            for idx in &indexes {
                state.prev_ingested.insert(idx.name.clone(), idx.total_docs_ingested);
                state.prev_search_count.insert(idx.name.clone(), idx.search_count);
            }

            // System RAM
            state.sys.refresh_memory();
            let ram_used_mb = state.sys.used_memory() / (1024 * 1024);
            let ram_total_mb = state.sys.total_memory() / (1024 * 1024);

            let payload = json!({
                "type":                 "metrics",
                "status":               "online",
                "totalDocs":            total_docs,
                "totalIndexes":         indexes.len(),
                "totalSegments":        total_segments,
                "totalPendingDocs":     total_pending,
                "ingestRate":           total_ingest_rate,
                "totalSearchRate":      total_search_rate,
                "totalMergesInProgress": 0,
                "ramUsedMb":            ram_used_mb,
                "ramTotalMb":           ram_total_mb,
                "indexes":              per_index,
            });

            let event = Ok::<Event, Infallible>(Event::default().data(payload.to_string()));
            Some((event, (mgr, state)))
        },
    )
}
