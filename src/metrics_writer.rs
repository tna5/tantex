use std::collections::HashMap;
use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::{SystemTime, UNIX_EPOCH};

use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

use crate::engine::index_manager::IndexManager;

pub async fn metrics_writer_loop(manager: Arc<RwLock<IndexManager>>, path: String) {
    log::info!("Metrics recording → {}", path);

    let file = match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        Ok(f) => f,
        Err(e) => {
            log::error!("Cannot open metrics file {}: {}", path, e);
            return;
        }
    };
    let mut writer = std::io::BufWriter::new(file);

    // Per-index previous values for delta computation
    let mut prev_live_total: HashMap<String, u64> = HashMap::new();
    let mut prev_search_count: HashMap<String, u64> = HashMap::new();
    let mut prev_latency_us: HashMap<String, u64> = HashMap::new();
    let mut prev_time = tokio::time::Instant::now();

    let mut ticker = interval(Duration::from_secs(1));
    ticker.tick().await; // skip the immediate first tick

    loop {
        ticker.tick().await;

        let now_instant = tokio::time::Instant::now();
        let elapsed_s = now_instant.duration_since(prev_time).as_secs_f64().max(0.1);
        let elapsed_ms = now_instant.duration_since(prev_time).as_millis() as u64;
        prev_time = now_instant;

        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mgr = manager.read().await;
        let snapshot = mgr.list_indexes();
        drop(mgr);

        let indexes_json: Vec<serde_json::Value> = snapshot
            .indexes
            .iter()
            .map(|idx| {
                let live_total = idx.doc_count + idx.pending_docs;
                let prev_lt = *prev_live_total.get(&idx.name).unwrap_or(&live_total);
                let ingest_rate = if live_total >= prev_lt {
                    ((live_total - prev_lt) as f64 / elapsed_s) as u64
                } else {
                    0
                };
                prev_live_total.insert(idx.name.clone(), live_total);

                let sc = idx.search_count;
                let prev_sc = *prev_search_count.get(&idx.name).unwrap_or(&sc);
                let delta_sc = sc.saturating_sub(prev_sc);
                let search_rate = delta_sc as f64 / elapsed_s;
                prev_search_count.insert(idx.name.clone(), sc);

                let sl = idx.search_latency_us;
                let prev_sl = *prev_latency_us.get(&idx.name).unwrap_or(&sl);
                let delta_sl = sl.saturating_sub(prev_sl);
                let avg_latency_us = if delta_sc > 0 { delta_sl / delta_sc } else { 0 };
                prev_latency_us.insert(idx.name.clone(), sl);

                serde_json::json!({
                    "name": idx.name,
                    "doc_count": idx.doc_count,
                    "pending_docs": idx.pending_docs,
                    "live_total": live_total,
                    "num_segments": idx.num_segments,
                    "search_count_total": sc,
                    "total_docs_ingested": idx.total_docs_ingested,
                    "ingest_rate": ingest_rate,
                    "search_rate_hz": (search_rate * 100.0).round() / 100.0,
                    "avg_search_latency_us": avg_latency_us,
                })
            })
            .collect();

        let line = serde_json::json!({
            "ts": ts,
            "elapsed_ms": elapsed_ms,
            "indexes": indexes_json,
        });

        if let Ok(s) = serde_json::to_string(&line) {
            let _ = writeln!(writer, "{}", s);
            let _ = writer.flush();
        }
    }
}

pub fn metrics_file_path() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    // Format: tant2_metrics_YYYYMMDD_HHMMSS.jsonl
    // Use seconds since epoch as a simple unique suffix
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Convert to a human-readable datetime string
    // We do it manually to avoid adding a chrono dependency
    let secs_in_day = ts % 86400;
    let days = ts / 86400;
    // Days since 1970-01-01 → approximate date (good enough for filenames)
    let h = secs_in_day / 3600;
    let m = (secs_in_day % 3600) / 60;
    let s = secs_in_day % 60;

    // Simple Gregorian approximation for YYYYMMDD
    let (year, month, day) = days_to_ymd(days);

    format!("tant2_metrics_{:04}{:02}{:02}_{:02}{:02}{:02}.jsonl", year, month, day, h, m, s)
}

fn days_to_ymd(days: u64) -> (u64, u64, u64) {
    // Days since Unix epoch (1970-01-01) → (year, month, day)
    let mut remaining = days;
    let mut year = 1970u64;
    loop {
        let days_in_year = if is_leap(year) { 366 } else { 365 };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        year += 1;
    }
    let months = [31u64, if is_leap(year) { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut month = 1u64;
    for &dim in &months {
        if remaining < dim {
            break;
        }
        remaining -= dim;
        month += 1;
    }
    (year, month, remaining + 1)
}

fn is_leap(y: u64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}
