# tantex

A high-performance, single-binary full-text search server built on [tantivy](https://github.com/quickwit-oss/tantivy).

- **5M docs/sec** ingestion via zero-copy shared-memory
- **HTTP REST API** + embedded dashboard
- **Unix socket** binary protocol for maximum throughput
- **Single binary** — statically-linked, zero dependencies
- **Runtime tuning** — change settings without restart

## Quick start

```bash
# Install
curl -fsSL https://raw.githubusercontent.com/tna5/tantex/main/install.sh | sh

# Start the server
tantex
# → Open http://localhost:7200
```

## Create an index and search

```bash
# Create index
curl -X POST http://localhost:7200/api/indexes \
  -H 'Content-Type: application/json' \
  -d '{
    "name": "articles",
    "schema": {
      "fields": [
        { "name": "title", "type": "text", "stored": true, "indexed": true },
        { "name": "body",  "type": "text", "stored": true, "indexed": true }
      ]
    }
  }'

# Add documents
curl -X POST http://localhost:7200/api/indexes/articles/ingest \
  -H 'Content-Type: application/x-ndjson' \
  -d '{"title": "Rust tips", "body": "..."}
{"title": "Full text search", "body": "..."}'

# Search
curl -X POST http://localhost:7200/api/indexes/articles/search \
  -H 'Content-Type: application/json' \
  -d '{"query": "rust", "limit": 5}'
```

## Installation options

1. **Automated installer** (recommended) — `curl | sh` script that detects your OS and architecture
2. **Download binary** — grab the latest release for Linux/macOS from [GitHub releases](https://github.com/tna5/tantex/releases)
3. **Build from source** — requires Rust ≥ 1.75

See [Installation guide](https://tna5.github.io/tantex/docs/tutorial/installation) for details.

## Documentation

Full documentation at **[tna5.github.io/tantex](https://tna5.github.io/tantex/)**

- [Getting started](https://tna5.github.io/tantex/docs/tutorial/getting-started) — 5-minute walkthrough
- [Configuration reference](https://tna5.github.io/tantex/docs/reference/configuration) — all environment variables
- [HTTP API](https://tna5.github.io/tantex/docs/reference/http-api) — complete endpoint reference
- [JavaScript client](https://tna5.github.io/tantex/docs/how-to/javascript-client) — npm package `@tna5/tantex`
- [Query syntax](https://tna5.github.io/tantex/docs/reference/query-syntax) — boolean, phrase, range queries

## Features

| | |
|---|---|
| **HTTP REST API** | JSON endpoints, SSE metrics, works from any language |
| **Embedded dashboard** | Live metrics, schema builder, search UI — no external frontend needed |
| **API keys** | bcrypt-hashed authentication for external clients |
| **Multiple transports** | HTTP, Unix socket, or high-throughput SHM ingest via mmap |

## Development

```bash
# Build
make

# Run tests
cargo test

# Build server only (skip dashboard)
cargo build --release
```

## License

MIT

Built on [tantivy](https://github.com/quickwit-oss/tantivy)
