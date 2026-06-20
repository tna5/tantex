---
layout: home

hero:
  name: "tantex"
  text: "Full-text search at scale"
  tagline: "Built on tantivy · HTTP + Unix socket · Embedded dashboard · Single binary"
  image:
    src: /logo.svg
    alt: tantex
  actions:
    - theme: brand
      text: Get Started
      link: /docs/tutorial/getting-started
    - theme: alt
      text: GitHub
      link: https://github.com/USERNAME/tantex

features:
  - icon: ⚡
    title: Up to 5M docs/sec
    details: Zero-copy shared-memory ingest via mmap files eliminates socket overhead entirely.
  - icon: 🦀
    title: Built on tantivy
    details: World-class BM25 full-text engine written in Rust. Tantex wraps it in a production-ready server with HTTP API and live dashboard.
  - icon: 🌐
    title: HTTP REST API
    details: JSON endpoints, SSE metrics stream, works from any language. Plus a low-level binary protocol for maximum throughput.
  - icon: 📊
    title: Embedded dashboard
    details: Live metrics, schema builder, and search UI — served directly from the binary, no extra process needed.
  - icon: 🔧
    title: Runtime tuning
    details: Change commit thresholds, thread counts, and merge policy on the fly without restarting the server.
  - icon: 🔐
    title: API key auth
    details: bcrypt-hashed keys for external clients. Localhost is always trusted — the dashboard works out of the box.
---

## Quick start

```bash
# Build
cargo build --release
./target/release/tantex

# Create an index
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

# Search
curl -X POST http://localhost:7200/api/indexes/articles/search \
  -H 'Content-Type: application/json' \
  -d '{"query": "rust performance", "limit": 5}'
```
