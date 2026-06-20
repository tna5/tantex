use std::env;
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub socket_path: String,
    pub http_port: u16,
    pub dashboard_dir: String,
    pub data_dir: String,
    pub shm_buffer_size: usize,
    pub writer_heap_size: usize,
    pub auto_commit_doc_count: usize,
    pub auto_commit_interval_secs: u64,
    pub merge_target_docs: usize,
    pub max_merge_factor: usize,
    pub min_num_segments: usize,
    pub num_indexing_threads: usize,
    /// Percentage of the thread budget given to tantivy's internal index_threads
    /// (the rest goes to the rayon parse pool). 0-100.
    pub index_threads_pct: u32,
    /// Hard auto-commit fires when pending docs reach this multiple of
    /// `auto_commit_doc_count`. Soft auto-commit fires at the base value when
    /// the writer channel is idle.
    pub hard_commit_multiplier: u32,
    /// If set, the HTTP dashboard and API require this key (via cookie, header, or query param).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
}

/// Partial config for runtime updates (only tunable fields, no path changes)
#[derive(Debug, Deserialize, Default)]
pub struct ConfigPatch {
    pub shm_buffer_size: Option<usize>,
    pub writer_heap_size: Option<usize>,
    pub auto_commit_doc_count: Option<usize>,
    pub auto_commit_interval_secs: Option<u64>,
    pub merge_target_docs: Option<usize>,
    pub max_merge_factor: Option<usize>,
    pub min_num_segments: Option<usize>,
    pub num_indexing_threads: Option<usize>,
    pub index_threads_pct: Option<u32>,
    pub hard_commit_multiplier: Option<u32>,
}

/// Extract the value for a CLI flag in either `--flag value` or `--flag=value` form.
fn flag_value<'a>(args: &'a [String], i: usize, flag: &str) -> Option<&'a str> {
    let arg = &args[i];
    if arg == flag {
        args.get(i + 1).map(|s| s.as_str())
    } else if let Some(rest) = arg.strip_prefix(&format!("{}=", flag)) {
        Some(rest)
    } else {
        None
    }
}

impl Config {
    /// Load config: defaults < config file < env vars < CLI flags
    pub fn from_env() -> Self {
        let mut cfg = Self::defaults();
        let args: Vec<String> = env::args().collect();

        // ── Pass 1: resolve data_dir only (env + CLI) so we know where the config file lives
        if let Ok(v) = env::var("TANTEX_DATA_DIR") {
            cfg.data_dir = v;
        }
        for i in 0..args.len() {
            if let Some(v) = flag_value(&args, i, "--data-dir") {
                cfg.data_dir = v.to_string();
            }
        }

        // ── Layer 2: JSON config file (read from data_dir)
        let config_path = Path::new(&cfg.data_dir).join("tantex.config.json");
        if config_path.exists() {
            match std::fs::read_to_string(&config_path) {
                Ok(contents) => match serde_json::from_str::<Config>(&contents) {
                    Ok(file_cfg) => {
                        cfg = file_cfg;
                        log::info!("Loaded config from {}", config_path.display());
                    }
                    Err(e) => log::warn!("Failed to parse {}: {}", config_path.display(), e),
                },
                Err(e) => log::warn!("Failed to read {}: {}", config_path.display(), e),
            }
        }

        // ── Layer 3: env vars override
        if let Ok(v) = env::var("TANTEX_SOCKET_PATH") { cfg.socket_path = v; }
        if let Some(v) = env::var("TANTEX_HTTP_PORT").ok().and_then(|v| v.parse().ok()) { cfg.http_port = v; }
        if let Ok(v) = env::var("TANTEX_DASHBOARD_DIR") { cfg.dashboard_dir = v; }
        if let Ok(v) = env::var("TANTEX_DATA_DIR") { cfg.data_dir = v; }
        if let Some(v) = env::var("TANTEX_SHM_BUFFER_SIZE").ok().and_then(|v| v.parse().ok()) { cfg.shm_buffer_size = v; }
        if let Some(v) = env::var("TANTEX_WRITER_HEAP_SIZE").ok().and_then(|v| v.parse().ok()) { cfg.writer_heap_size = v; }
        if let Some(v) = env::var("TANTEX_AUTO_COMMIT_DOC_COUNT").ok().and_then(|v| v.parse().ok()) { cfg.auto_commit_doc_count = v; }
        if let Some(v) = env::var("TANTEX_AUTO_COMMIT_INTERVAL_SECS").ok().and_then(|v| v.parse().ok()) { cfg.auto_commit_interval_secs = v; }
        if let Some(v) = env::var("TANTEX_MERGE_TARGET_DOCS").ok().and_then(|v| v.parse().ok()) { cfg.merge_target_docs = v; }
        if let Some(v) = env::var("TANTEX_MAX_MERGE_FACTOR").ok().and_then(|v| v.parse().ok()) { cfg.max_merge_factor = v; }
        if let Some(v) = env::var("TANTEX_NUM_INDEXING_THREADS").ok().and_then(|v| v.parse().ok()) { cfg.num_indexing_threads = v; }
        if let Some(v) = env::var("TANTEX_INDEX_THREADS_PCT").ok().and_then(|v| v.parse().ok()) { cfg.index_threads_pct = v; }
        if let Some(v) = env::var("TANTEX_HARD_COMMIT_MULTIPLIER").ok().and_then(|v| v.parse().ok()) { cfg.hard_commit_multiplier = v; }
        if let Some(v) = env::var("TANTEX_MIN_NUM_SEGMENTS").ok().and_then(|v| v.parse().ok()) { cfg.min_num_segments = v; }
        if let Ok(v) = env::var("TANTEX_API_KEY") {
            if !v.is_empty() { cfg.api_key = Some(v); }
        }

        // ── Layer 4: CLI flags (highest priority)
        for i in 0..args.len() {
            let arg = &args[i];

            if let Some(v) = flag_value(&args, i, "--port").and_then(|s| s.parse().ok()) {
                cfg.http_port = v;
            } else if let Some(v) = flag_value(&args, i, "--socket") {
                cfg.socket_path = v.to_string();
            } else if let Some(v) = flag_value(&args, i, "--data-dir") {
                cfg.data_dir = v.to_string();
            } else if let Some(v) = flag_value(&args, i, "--api-key") {
                if !v.is_empty() { cfg.api_key = Some(v.to_string()); }
            } else if let Some(v) = flag_value(&args, i, "--shm-buffer-size").and_then(|s| s.parse().ok()) {
                cfg.shm_buffer_size = v;
            } else if let Some(v) = flag_value(&args, i, "--writer-heap-size").and_then(|s| s.parse().ok()) {
                cfg.writer_heap_size = v;
            } else if let Some(v) = flag_value(&args, i, "--auto-commit-doc-count").and_then(|s| s.parse().ok()) {
                cfg.auto_commit_doc_count = v;
            } else if let Some(v) = flag_value(&args, i, "--auto-commit-interval").and_then(|s| s.parse().ok()) {
                cfg.auto_commit_interval_secs = v;
            } else if let Some(v) = flag_value(&args, i, "--merge-target-docs").and_then(|s| s.parse().ok()) {
                cfg.merge_target_docs = v;
            } else if let Some(v) = flag_value(&args, i, "--max-merge-factor").and_then(|s| s.parse().ok()) {
                cfg.max_merge_factor = v;
            } else if let Some(v) = flag_value(&args, i, "--min-segments").and_then(|s| s.parse().ok()) {
                cfg.min_num_segments = v;
            } else if let Some(v) = flag_value(&args, i, "--threads").and_then(|s| s.parse().ok()) {
                cfg.num_indexing_threads = v;
            } else if let Some(v) = flag_value(&args, i, "--index-threads-pct").and_then(|s| s.parse().ok()) {
                cfg.index_threads_pct = v;
            } else if let Some(v) = flag_value(&args, i, "--hard-commit-multiplier").and_then(|s| s.parse().ok()) {
                cfg.hard_commit_multiplier = v;
            } else if arg == "--help" {
                eprintln!("tantex - Full-text search server\n");
                eprintln!("Usage: tantex [OPTIONS]\n");
                eprintln!("Options:");
                eprintln!("  --port <N>                   HTTP server port (default: 7200)");
                eprintln!("  --socket <PATH>              Unix socket path (default: /tmp/tantex.sock)");
                eprintln!("  --data-dir <DIR>             Index storage directory (default: ./data)");
                eprintln!("                               The config file is read from <DIR>/tantex.config.json");
                eprintln!("  --api-key <KEY>              Protect the HTTP API and dashboard with this key");
                eprintln!("  --threads <N>                Total indexing thread budget (default: 8)");
                eprintln!("  --index-threads-pct <N>      % of threads for tantivy vs parse pool (default: 63)");
                eprintln!("  --writer-heap-size <N>       Tantivy writer heap in bytes (default: 4000000000)");
                eprintln!("  --shm-buffer-size <N>        SHM buffer size in bytes (default: 268435456)");
                eprintln!("  --auto-commit-doc-count <N>  Soft commit threshold in docs (default: 10000000)");
                eprintln!("  --auto-commit-interval <N>   Idle commit timer in seconds (default: 30)");
                eprintln!("  --hard-commit-multiplier <N> Hard commit at N×soft threshold (default: 4)");
                eprintln!("  --merge-target-docs <N>      Target segment size for merges (default: 20000000)");
                eprintln!("  --max-merge-factor <N>       Max segments merged in one pass (default: 10)");
                eprintln!("  --min-segments <N>           Min segments before a merge (default: 2)");
                eprintln!("  --help                       Show this help message\n");
                eprintln!("All options can also be set via environment variables (TANTEX_*)");
                eprintln!("or in <data-dir>/tantex.config.json.");
                eprintln!("Priority: CLI flags > environment variables > config file > defaults\n");
                eprintln!("Examples:");
                eprintln!("  tantex --port 8080");
                eprintln!("  tantex --data-dir /mnt/indexes --threads 16");
                eprintln!("  tantex --api-key secret123");
                eprintln!("  TANTEX_HTTP_PORT=8080 tantex");
                std::process::exit(0);
            }
        }

        cfg
    }

    /// Apply a partial update (only tunable fields)
    pub fn apply_patch(&mut self, patch: ConfigPatch) {
        if let Some(v) = patch.shm_buffer_size { self.shm_buffer_size = v; }
        if let Some(v) = patch.writer_heap_size { self.writer_heap_size = v; }
        if let Some(v) = patch.auto_commit_doc_count { self.auto_commit_doc_count = v; }
        if let Some(v) = patch.auto_commit_interval_secs { self.auto_commit_interval_secs = v; }
        if let Some(v) = patch.merge_target_docs { self.merge_target_docs = v; }
        if let Some(v) = patch.max_merge_factor { self.max_merge_factor = v; }
        if let Some(v) = patch.min_num_segments { self.min_num_segments = v; }
        if let Some(v) = patch.num_indexing_threads { self.num_indexing_threads = v; }
        if let Some(v) = patch.index_threads_pct { self.index_threads_pct = v; }
        if let Some(v) = patch.hard_commit_multiplier { self.hard_commit_multiplier = v; }
    }

    fn defaults() -> Self {
        Self {
            socket_path: "/tmp/tantex.sock".to_string(),
            http_port: 7200,
            dashboard_dir: "./dashboard/.output/public".to_string(),
            data_dir: "./data".to_string(),
            shm_buffer_size: 256 * 1024 * 1024,
            // 4 GB heap → fewer segment flushes; matches the bench sweet spot.
            writer_heap_size: 4_000_000_000,
            // 10M soft / 40M hard maximises sustained throughput on M2 benches
            // (commit pauses are amortised against many docs).
            auto_commit_doc_count: 10_000_000,
            auto_commit_interval_secs: 30,
            merge_target_docs: 20_000_000,
            max_merge_factor: 10,
            min_num_segments: 2,
            num_indexing_threads: 8,
            // 63% to tantivy index threads, ~37% to rayon parse pool.
            // At the default 8-thread budget this resolves to 5 index + 3 parse,
            // matching the split benchmarked as fastest on M2 (4P+4E).
            index_threads_pct: 63,
            // hard limit = 4× soft limit fires under continuous ingest when the
            // soft "idle" condition never triggers.
            hard_commit_multiplier: 4,
            api_key: None,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::from_env()
    }
}
