# Configuration Reference

Configuration is loaded in order of increasing priority:

1. **Built-in defaults**
2. **Config file** — `{data_dir}/tantex.config.json`
3. **Environment variables** — `TANTEX_*`
4. **CLI flags** — highest priority, override everything

`data_dir` itself is resolved from environment variables and CLI flags first (before the config file is read), so you can always point `--data-dir` to the right location.

---

## Config file

Place a file named `tantex.config.json` inside your data directory (default: `./data/tantex.config.json`). All fields are optional — missing fields fall back to defaults.

```json
{
  "socket_path": "/tmp/tantex.sock",
  "http_port": 3000,
  "data_dir": "./data",
  "api_key": "your-secret-key",
  "writer_heap_size": 4000000000,
  "auto_commit_doc_count": 10000000,
  "auto_commit_interval_secs": 30,
  "hard_commit_multiplier": 4,
  "num_indexing_threads": 8,
  "index_threads_pct": 63,
  "shm_buffer_size": 268435456,
  "merge_target_docs": 20000000,
  "max_merge_factor": 10,
  "min_num_segments": 2
}
```

---

## CLI flags

| Flag | Description |
|---|---|
| `--port <N>` | HTTP server port (default: `7200`) |
| `--socket <PATH>` | Unix socket path (default: `/tmp/tantex.sock`) |
| `--data-dir <DIR>` | Index storage directory (default: `./data`). Config file is read from `<DIR>/tantex.config.json`. |
| `--api-key <KEY>` | Protect the HTTP API and dashboard with this key |
| `--threads <N>` | Total indexing thread budget (default: `8`) |
| `--index-threads-pct <N>` | % of threads for tantivy vs parse pool (default: `63`) |
| `--writer-heap-size <N>` | Tantivy writer heap in bytes (default: `4000000000`) |
| `--shm-buffer-size <N>` | SHM buffer size per session in bytes (default: `268435456`) |
| `--auto-commit-doc-count <N>` | Soft commit threshold in documents (default: `10000000`) |
| `--auto-commit-interval <N>` | Idle commit timer in seconds (default: `30`) |
| `--hard-commit-multiplier <N>` | Hard commit at N × soft threshold (default: `4`) |
| `--merge-target-docs <N>` | Target segment size for merges (default: `20000000`) |
| `--max-merge-factor <N>` | Max segments merged in one pass (default: `10`) |
| `--min-segments <N>` | Min segments before a merge is scheduled (default: `2`) |
| `--help` | Print usage and exit |

Flags accept both `--flag value` and `--flag=value` forms.

---

## Environment variables

| Variable | Default | Description |
|---|---|---|
| `TANTEX_SOCKET_PATH` | `/tmp/tantex.sock` | Unix socket path |
| `TANTEX_HTTP_PORT` | `7200` | HTTP port |
| `TANTEX_DATA_DIR` | `./data` | Data directory (config file is read from here) |
| `TANTEX_API_KEY` | — | API key for authentication |
| `TANTEX_WRITER_HEAP_SIZE` | `4000000000` | Writer heap in bytes |
| `TANTEX_SHM_BUFFER_SIZE` | `268435456` | SHM buffer size per session in bytes |
| `TANTEX_AUTO_COMMIT_DOC_COUNT` | `10000000` | Soft commit threshold (documents) |
| `TANTEX_AUTO_COMMIT_INTERVAL_SECS` | `30` | Idle commit timer (seconds) |
| `TANTEX_HARD_COMMIT_MULTIPLIER` | `4` | Hard commit multiplier |
| `TANTEX_NUM_INDEXING_THREADS` | `8` | Total thread budget |
| `TANTEX_INDEX_THREADS_PCT` | `63` | % of threads for tantivy |
| `TANTEX_MERGE_TARGET_DOCS` | `20000000` | Target segment size |
| `TANTEX_MAX_MERGE_FACTOR` | `10` | Max segments merged per pass |
| `TANTEX_MIN_NUM_SEGMENTS` | `2` | Min segments before merge |

---

## Authentication (`api_key`) {#api_key}

When `api_key` is set, all `/api/*` requests must include the key (except `/api/auth/status` and `/api/auth/login`).

Pass the key via:
- `X-Api-Key: <key>` header
- `Authorization: Bearer <key>` header
- `?api_key=<key>` query parameter (useful for EventSource)
- Cookie — set automatically by the dashboard login page

The dashboard shows a login form when auth is required. See [Authentication](http-api.md#authentication) in the HTTP API reference.

---

## Runtime configuration

All tunable fields (everything except `socket_path`, `http_port`, `data_dir`) can be updated without restarting:

```sh
curl -X POST http://localhost:7200/api/config \
  -H 'Content-Type: application/json' \
  -d '{"auto_commit_doc_count": 1000000, "auto_commit_interval_secs": 5}'
```

See [HTTP API — Update configuration](http-api.md#update-configuration).

---

## Presets

### Max ingest throughput

```json
{
  "writer_heap_size": 4000000000,
  "auto_commit_doc_count": 10000000,
  "auto_commit_interval_secs": 60,
  "hard_commit_multiplier": 4,
  "num_indexing_threads": 8,
  "index_threads_pct": 63,
  "merge_target_docs": 20000000,
  "max_merge_factor": 10
}
```

### Balanced

```json
{
  "writer_heap_size": 2000000000,
  "auto_commit_doc_count": 1000000,
  "auto_commit_interval_secs": 30,
  "hard_commit_multiplier": 4,
  "num_indexing_threads": 8,
  "index_threads_pct": 63,
  "merge_target_docs": 10000000,
  "max_merge_factor": 10
}
```

### Low latency

```json
{
  "writer_heap_size": 500000000,
  "auto_commit_doc_count": 100000,
  "auto_commit_interval_secs": 5,
  "hard_commit_multiplier": 4,
  "num_indexing_threads": 8,
  "index_threads_pct": 63,
  "merge_target_docs": 5000000,
  "max_merge_factor": 6
}
```
