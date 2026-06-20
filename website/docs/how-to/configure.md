# How to Configure Tantex

For all available options see the [Configuration reference](../reference/configuration.md). This page shows common recipes.

---

## Use a config file

Create `tantex.config.json` inside your data directory (default: `./data/`). Missing fields fall back to defaults.

```json
{
  "http_port": 8080,
  "socket_path": "/var/run/tantex.sock",
  "data_dir": "/mnt/search-data"
}
```

## Change the port or socket path

::: code-group

```sh [CLI flags]
./tantex --port 8080 --socket /var/run/tantex.sock
```

```sh [Environment variables]
TANTEX_HTTP_PORT=8080 TANTEX_SOCKET_PATH=/var/run/tantex.sock ./tantex
```

:::

## Change the data directory

```sh
./tantex --data-dir /mnt/search-data
```

All indexes and the config file are stored under this directory.

## Tune for maximum ingest throughput

::: code-group

```sh [CLI flags]
./tantex --writer-heap-size 8000000000 --auto-commit-doc-count 20000000 \
         --auto-commit-interval 60 --threads 12 --index-threads-pct 67
```

```sh [Environment variables]
TANTEX_WRITER_HEAP_SIZE=8000000000 \
TANTEX_AUTO_COMMIT_DOC_COUNT=20000000 \
TANTEX_AUTO_COMMIT_INTERVAL_SECS=60 \
TANTEX_NUM_INDEXING_THREADS=12 \
TANTEX_INDEX_THREADS_PCT=67 \
./tantex
```

:::

Or apply at runtime without restarting:

```sh
curl -X POST http://localhost:7200/api/config \
  -H 'Content-Type: application/json' \
  -d '{"writer_heap_size": 8000000000, "auto_commit_doc_count": 20000000, "auto_commit_interval_secs": 60}'
```

## Tune for low search latency

```sh
./tantex --auto-commit-doc-count 100000 --auto-commit-interval 5 --writer-heap-size 500000000
```

## Adjust the thread split

`--threads` is the total budget. `--index-threads-pct` controls the split between tantivy index threads and the rayon JSON parse pool. At the defaults (8 threads, 63%) the split is 5 index + 3 parse.

```sh
# 16-core machine
./tantex --threads 16 --index-threads-pct 63
```

## Protect the API with a key

```sh
./tantex --api-key "my-secret-key"
```

See [Authentication](../reference/configuration.md#api_key) for all authentication options.

## Enable verbose logging

```sh
RUST_LOG=debug ./tantex
# Fine-grained:
RUST_LOG=tantex::engine=debug,tantivy::indexer=info ./tantex
```
