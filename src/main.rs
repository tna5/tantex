mod config;
mod engine;
mod http;
mod metrics_writer;
mod protocol;
mod server;
mod shm;

use std::io::Write;
use std::sync::Arc;
use tokio::sync::RwLock;

use config::Config;
use engine::index_manager::IndexManager;

/// Set up env_logger with a compact, coloured format and sensible defaults:
/// - tant2 modules → info (so the user sees what's happening)
/// - tantivy/internal libs → warn (merge/flush noise muted by default)
/// Override either with `RUST_LOG=…` (e.g. `RUST_LOG=tantivy=info` to debug merges).
fn init_logger() {
    // Defaults: see tant2's own events + tantivy *merges* (interesting), but
    // suppress tantivy's per-file directory chatter and tokio internals.
    let default = "info,\
        tantivy=warn,\
        tantivy::indexer::merger=info,\
        tantivy::indexer::segment_updater=info,\
        tokio=warn,want=warn,mio=warn,hyper=warn";
    let env = env_logger::Env::default().default_filter_or(default);
    env_logger::Builder::from_env(env)
        .format(|buf, record| {
            let ts = buf.timestamp_seconds();
            let level = record.level();
            let level_str = match level {
                log::Level::Error => "\x1b[31mERR\x1b[0m",
                log::Level::Warn  => "\x1b[33mWRN\x1b[0m",
                log::Level::Info  => "\x1b[32mINF\x1b[0m",
                log::Level::Debug => "\x1b[36mDBG\x1b[0m",
                log::Level::Trace => "\x1b[90mTRC\x1b[0m",
            };
            // Strip the crate prefix to keep lines short — "engine::writer" not "tantex::engine::writer".
            let target = record.target().strip_prefix("tantex::").unwrap_or(record.target());
            writeln!(
                buf,
                "\x1b[90m{}\x1b[0m {} \x1b[35m{:<22}\x1b[0m {}",
                ts,
                level_str,
                target,
                record.args()
            )
        })
        .init();
}

fn fmt_bytes(n: usize) -> String {
    if n >= 1_000_000_000 { format!("{:.2} GB", n as f64 / 1e9) }
    else if n >= 1_000_000 { format!("{:.1} MB", n as f64 / 1e6) }
    else if n >= 1_000 { format!("{:.1} KB", n as f64 / 1e3) }
    else { format!("{} B", n) }
}

fn fmt_num(n: usize) -> String {
    let mut s = String::new();
    let str = n.to_string();
    let bytes = str.as_bytes();
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 && (bytes.len() - i) % 3 == 0 { s.push('_'); }
        s.push(*b as char);
    }
    s
}

/// Print the boot banner directly to stderr — bypasses the log filter so the
/// user always sees the running config even when RUST_LOG=warn/error.
fn log_startup(c: &Config) {
    let dim = "\x1b[90m";
    let cyan = "\x1b[36m";
    let bold = "\x1b[1m";
    let reset = "\x1b[0m";
    eprintln!();
    eprintln!("  {cyan}╭─{reset} {bold}tantex{reset} {dim}server starting…{reset}");
    eprintln!("  {cyan}│{reset} socket  {}", c.socket_path);
    eprintln!("  {cyan}│{reset} http    :{}", c.http_port);
    eprintln!("  {cyan}│{reset} data    {}", c.data_dir);
    eprintln!("  {cyan}│{reset}");
    eprintln!("  {cyan}│{reset} writer heap         {}", fmt_bytes(c.writer_heap_size));
    eprintln!("  {cyan}│{reset} shm buffer          {}", fmt_bytes(c.shm_buffer_size));
    eprintln!("  {cyan}│{reset} threads             {} ({}% to tantivy, rest to rayon parse)",
        c.num_indexing_threads, c.index_threads_pct);
    eprintln!("  {cyan}│{reset}");
    eprintln!("  {cyan}│{reset} auto-commit soft    {} docs", fmt_num(c.auto_commit_doc_count));
    eprintln!("  {cyan}│{reset} auto-commit hard    {} docs ({}× soft)",
        fmt_num(c.auto_commit_doc_count.saturating_mul(c.hard_commit_multiplier as usize)),
        c.hard_commit_multiplier);
    eprintln!("  {cyan}│{reset} auto-commit timer   {} s idle", c.auto_commit_interval_secs);
    eprintln!("  {cyan}│{reset}");
    eprintln!("  {cyan}│{reset} merge target        {} docs/segment", fmt_num(c.merge_target_docs));
    eprintln!("  {cyan}│{reset} merge factor        max {}, min {} segments to trigger",
        c.max_merge_factor, c.min_num_segments);
    eprintln!("  {cyan}│{reset}");

    let fd_limit = unsafe {
        let mut rlim: libc::rlimit = std::mem::zeroed();
        if libc::getrlimit(libc::RLIMIT_NOFILE, &mut rlim) == 0 {
            Some(rlim.rlim_cur)
        } else {
            None
        }
    };
    if let Some(limit) = fd_limit {
        eprintln!("  {cyan}│{reset} fd limit             {}", fmt_num(limit as usize));
    }
    eprintln!("  {cyan}╰─{reset} {dim}listening on {} and :{}{reset}", c.socket_path, c.http_port);
    eprintln!();
}

/// Raise the soft `RLIMIT_NOFILE` to the hard limit (capped at 65536).
/// Prevents "Too many open files" errors when many indexes/segments are loaded.
fn raise_fd_limit() {
    unsafe {
        let mut rlim: libc::rlimit = std::mem::zeroed();
        if libc::getrlimit(libc::RLIMIT_NOFILE, &mut rlim) == 0 {
            let target = if rlim.rlim_max == libc::RLIM_INFINITY || rlim.rlim_max > 65536 {
                65536
            } else {
                rlim.rlim_max
            };
            if rlim.rlim_cur < target {
                let old = rlim.rlim_cur;
                rlim.rlim_cur = target;
                if libc::setrlimit(libc::RLIMIT_NOFILE, &mut rlim) == 0 {
                    log::info!("Raised RLIMIT_NOFILE from {} to {}", old, target);
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logger();
    raise_fd_limit();

    let save_metrics = std::env::args().any(|a| a == "--save-metrics");

    let config = Config::from_env();
    log_startup(&config);

    // Ensure data directory exists
    std::fs::create_dir_all(&config.data_dir)?;

    // Initialise index manager and reload any persisted indexes
    let mut index_manager = IndexManager::new(config.clone());
    index_manager.load_existing()?;

    let index_manager = Arc::new(RwLock::new(index_manager));
    let config = Arc::new(RwLock::new(config));

    // Metrics recording task
    if save_metrics {
        let path = metrics_writer::metrics_file_path();
        let mgr = Arc::clone(&index_manager);
        tokio::spawn(async move {
            metrics_writer::metrics_writer_loop(mgr, path).await;
        });
    }

    // Start both listeners — they both run indefinitely
    let socket_task = server::listener::start_server(Arc::clone(&config), Arc::clone(&index_manager));
    let http_task = http::start_http_server(config, Arc::clone(&index_manager));

    tokio::select! {
        res_socket = socket_task => res_socket?,
        res_http = http_task => res_http?,
        _ = tokio::signal::ctrl_c() => {
            log::info!("Shutdown signal received, dropping indexes and exiting...");
            // Try to drop readers first (frees Fds); if the read lock is
            // contended we just exit — the kernel reclaims everything.
            if let Ok(mut mgr) = index_manager.try_write() {
                mgr.shutdown();
            }
            log::info!("Goodbye.");
            std::process::exit(0);
        }
    }

    Ok(())
}
