# tantex

A high-performance, single-binary full-text search server built on [tantivy](https://github.com/quickwit-oss/tantivy).

- **5M docs/sec** ingestion via zero-copy shared-memory (SHM)
- **HTTP REST API** + embedded dashboard — no external frontend needed
- **Unix socket** binary protocol for extreme throughput
- **Runtime tuning** — change commit thresholds, threads, and merge policy without restart
- **API key authentication** for external clients
- **Single statically-linked binary** — no dependencies, no containers

```bash
curl -fsSL https://raw.githubusercontent.com/tna5/tantex/main/install.sh | sh
tantex
# → Server ready on http://localhost:7200
```

---

## Quick start

**1. Start the server:**

```bash
tantex
```

**2. Create an index:**

```bash
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
```

**3. Add documents:**

```bash
curl -X POST http://localhost:7200/api/indexes/articles/ingest \
  -H 'Content-Type: application/x-ndjson' \
  -d '{
  {"title": "Rust performance", "body": "..."}
  {"title": "Full text search", "body": "..."}
  '
```

**4. Search:**

```bash
curl -X POST http://localhost:7200/api/indexes/articles/search \
  -H 'Content-Type: application/json' \
  -d '{"query": "rust performance", "limit": 5}'
```

Open [http://localhost:7200](http://localhost:7200) in your browser to see the dashboard.

---

## Features

| | |
|---|---|
| **⚡ Extreme throughput** | 5M documents/second via zero-copy SHM ingestion. TCP socket and HTTP for standard workflows. |
| **🦀 Built on tantivy** | World-class BM25 full-text engine in pure Rust. Production-grade with years of battle-testing. |
| **🌐 HTTP REST API** | JSON endpoints, SSE metrics stream, works from any language. Localhost is always trusted. |
| **📊 Embedded dashboard** | Live metrics, schema builder, search UI, and settings — served directly from the binary. |
| **🔧 Runtime tuning** | Change commit thresholds, thread counts, merge policy on the fly. No restart needed. |
| **🔐 API key auth** | bcrypt-hashed keys for external clients. Fine-grained per-index permissions (coming soon). |
| **📦 Single binary** | Statically-linked, zero dependencies. Drop it on any Linux/macOS server and go. |

---

## Installation

### Option 1: Automated installer (recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/tna5/tantex/main/install.sh | sh
```

Detects your OS (Linux, macOS) and architecture (x86_64, ARM64), downloads the latest release, and offers to install globally.

### Option 2: Download manually

Download from [GitHub releases](https://github.com/tna5/tantex/releases):

| OS | Architecture | Binary |
|---|---|---|
| Linux | x86_64 | `tantex-linux-x86_64` |
| macOS | Intel (x86_64) | `tantex-macos-x86_64` |
| macOS | Apple Silicon (ARM64) | `tantex-macos-arm64` |

```bash
chmod +x tantex-linux-x86_64
./tantex-linux-x86_64
```

### Option 3: Build from source

Requires Rust ≥ 1.75 and Bun (optional, for dashboard):

```bash
git clone https://github.com/tna5/tantex
cd tantex
make
./target/release/tantex
```

---

## Configuration

Tantex reads from environment variables:

```bash
TANTEX_HTTP_PORT=7200                          # HTTP API port
TANTEX_SOCKET_PATH=/tmp/tantex.sock            # Unix socket path
TANTEX_DATA_DIR=./data                         # Where to store indexes
TANTEX_WRITER_HEAP_SIZE=4000000000             # Tantivy buffer (bytes)
TANTEX_NUM_INDEXING_THREADS=8                  # Total writer threads
TANTEX_AUTO_COMMIT_DOC_COUNT=10000000          # Soft commit threshold
TANTEX_AUTO_COMMIT_INTERVAL_SECS=30            # Hard commit timer
RUST_LOG=info                                  # Log level
```

See [Configuration reference](https://tna5.github.io/tantex/docs/reference/configuration) for all options.

---

## Architecture

**Two servers, one process:**

1. **HTTP server** (Axum) on port 7200 — REST API (`/api/indexes`, `/api/search`) and embedded Nuxt dashboard
2. **Unix socket server** at `/tmp/tantex.sock` — binary MessagePack protocol for high-throughput clients

**Indexing:**

- Per-index **writer thread** (OS thread, not async) owns a `tantivy::IndexWriter`
- Commands arrive via unbounded channel as `WriterCommand` enum
- Auto-commits trigger on doc count or idle timer
- Document parsing uses **Rayon thread pool** + **sonic-rs** (SIMD JSON) for zero-copy deserialization

**Search:**

- Runs directly on tokio threads (non-blocking — `tantivy::Searcher` is `Send + Sync`)
- BM25 scoring with configurable field boosts

**SHM ingestion (high-throughput path):**

```
Client                          Server
  ↓
  [MSG_INIT_SHM]  ────────→  Create /tmp/tant_<uuid>
  ↓                             ↓
  [mmap file]  ←──────────  Return fd/path
  ↓
  [Write NDJSON]  (in-place, zero-copy)
  ↓
  [MSG_INGEST_SHM]  ────────→  Read from mmap
                                ↓
                            [tantivy::Document]
                                ↓
                            Writer thread
```

---

## API Overview

### Create an index

```bash
POST /api/indexes
{
  "name": "products",
  "schema": {
    "fields": [
      { "name": "id",    "type": "u64",  "fast": true, "stored": true },
      { "name": "title", "type": "text", "indexed": true, "stored": true },
      { "name": "price", "type": "f64",  "fast": true },
      { "name": "sku",   "type": "text", "tokenizer": "raw", "stored": true }
    ]
  }
}
```

### Ingest documents

```bash
POST /api/indexes/{index}/ingest
Content-Type: application/x-ndjson

{"id": 1, "title": "Widget", "price": 9.99, "sku": "W123"}
{"id": 2, "title": "Gadget", "price": 19.99, "sku": "G456"}
```

### Search

```bash
POST /api/indexes/{index}/search
{
  "query": "widget",
  "limit": 10,
  "offset": 0,
  "fields": ["title", "sku"],  // optional: which fields to return
  "boost": { "title": 2.0 }     // optional: field boosts
}
```

### Commit changes

```bash
POST /api/indexes/{index}/commit
```

See [HTTP API reference](https://tna5.github.io/tantex/docs/reference/http-api) for all endpoints.

---

## JavaScript / TypeScript client

The [`@tna5/tantex`](https://www.npmjs.com/package/@tna5/tantex) npm package provides:

- `TantexSocketClient` — Unix socket + binary protocol (Node.js)
- `TantexShmIngestor` — Shared-memory ultra-fast ingestion (Bun)
- `TantexHttpClient` — HTTP REST client (all runtimes)
- `Tantex` — High-level wrapper (recommended)

```bash
npm install @tna5/tantex
```

```ts
import { Tantex } from '@tna5/tantex'

const client = new Tantex()

await client.createIndex('products', {
  fields: [
    { name: 'title', type: 'text' },
    { name: 'price', type: 'f64', fast: true },
  ]
})

await client.ingest('products', [
  { title: 'Widget', price: 9.99 },
  { title: 'Gadget', price: 19.99 },
])

await client.commit('products')

const results = await client.search('products', {
  query: 'widget',
  limit: 10,
})
```

See [JavaScript client guide](https://tna5.github.io/tantex/docs/how-to/javascript-client) for streaming and high-throughput patterns.

---

## Performance

Benchmarks on a 2021 MacBook Pro (M1, 8 cores):

| Workload | Throughput | Notes |
|---|---|---|
| **SHM ingest** (Bun) | 5–6M docs/sec | Zero-copy, mmap-based |
| **HTTP ingest** (curl) | 50–100K docs/sec | Single request, batched |
| **Search (1K indexes)** | <5ms median | BM25 scoring, 10K doc limit |

See the [benchmark scripts](./scripts/) for reproducible tests.

---

## Documentation

Full docs at **[tna5.github.io/tantex](https://tna5.github.io/tantex/)**

- [Getting started](https://tna5.github.io/tantex/docs/tutorial/getting-started)
- [Configuration reference](https://tna5.github.io/tantex/docs/reference/configuration)
- [Field types & schema design](https://tna5.github.io/tantex/docs/how-to/design-schema)
- [Query syntax](https://tna5.github.io/tantex/docs/reference/query-syntax)
- [Indexing pipeline](https://tna5.github.io/tantex/docs/explanation/indexing)

---

## Development

### Build

```bash
# Full build (server + dashboard)
make

# Server only
cargo build --release

# Cross-compile for Linux (on macOS)
make TARGET=linux
```

### Run tests

```bash
cargo test
```

### Dashboard dev server

Requires the main server running on `:7200`:

```bash
cd dashboard
bun run dev
```

---

## License

MIT

Built on [tantivy](https://github.com/quickwit-oss/tantivy) · Inspired by [meilisearch](https://meilisearch.com) and [typesense](https://typesense.org)
