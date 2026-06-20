# Tantex Documentation

Tantex is a high-performance full-text search server built on [tantivy](https://github.com/quickwit-oss/tantivy). It exposes two interfaces:

- **HTTP/REST API** — JSON over HTTP on port 7200 (dashboard + REST clients)
- **Unix Domain Socket** — binary-framed MessagePack protocol for high-throughput ingest and search

The dashboard (Nuxt 4) is embedded in the binary and served automatically.

---

## Documentation map

### Tutorials — learn by doing

| Document | Description |
|---|---|
| [Getting Started](tutorial/getting-started.md) | Build the server, start it, create an index, ingest documents, and run your first search |

### How-to Guides — solve a specific problem

| Document | Description |
|---|---|
| [Configure tantex](how-to/configure.md) | Common configuration recipes — port, data dir, tuning, auth |
| [Design a schema](how-to/design-schema.md) | Choose field types, storage options, and tokenizers |
| [Ingest documents](how-to/ingest-documents.md) | Batch ingest via HTTP or shared memory for maximum throughput |
| [Search an index](how-to/search.md) | Write queries, paginate results, read scores |
| [Use the dashboard](how-to/dashboard.md) | Manage indexes, run searches, monitor live metrics |
| [Use the JavaScript client](how-to/javascript-client.md) | Connect from Node/Bun, ingest via SHM, run searches |

### Reference — look up exact details

| Document | Description |
|---|---|
| [Configuration](reference/configuration.md) | Config file, CLI flags, environment variables, authentication |
| [Field types](reference/field-types.md) | All supported field types including `array<*>` and `ip` |
| [HTTP API](reference/http-api.md) | All REST endpoints, request/response schemas |
| [Binary protocol](reference/protocol.md) | Frame format, message type constants, MessagePack payloads |
| [Query syntax](reference/query-syntax.md) | Tantivy query language — terms, phrases, ranges, booleans |

### Explanation — understand the design

| Document | Description |
|---|---|
| [Indexing pipeline](explanation/indexing.md) | Document lifecycle from ingest to segment, commit triggers |
| [Search and scoring](explanation/searching.md) | Searcher, BM25 relevance, query parsing |
| [Compression](explanation/compression.md) | Doc store codecs, block size tradeoffs |
| [Shared-memory ingestion](explanation/shm-ingestion.md) | How zero-copy SHM ingest works and when to use it |
| [Commit and merge policy](explanation/commit-merge-policy.md) | Auto-commit thresholds and merge strategy |
