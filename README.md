# tantex

A high-performance, single-binary full-text search server built on [tantivy](https://github.com/quickwit-oss/tantivy).

- **5M docs/sec** ingestion via zero-copy shared-memory
- **HTTP REST API** + embedded dashboard
- **Unix socket** binary protocol for maximum throughput
- **Single binary** — statically-linked, zero dependencies
- **Runtime tuning** — change settings without restart

## Installation

```bash
curl -fsSL https://raw.githubusercontent.com/tna5/tantex/main/install.sh | sh
tantex
# → Open http://localhost:7200
```

Pre-built binaries for Linux and macOS are available on the [releases page](https://github.com/tna5/tantex/releases).

## Documentation

**[tna5.github.io/tantex](https://tna5.github.io/tantex/)**

## Features

| | |
|---|---|
| **HTTP REST API** | JSON endpoints, SSE metrics, works from any language |
| **Embedded dashboard** | Live metrics, schema builder, search UI — no external frontend needed |
| **API keys** | bcrypt-hashed authentication for external clients |
| **Multiple transports** | HTTP, Unix socket, or high-throughput SHM ingest via mmap |

<details>
<summary>Development</summary>

```bash
make              # Build (server + dashboard)
cargo build --release   # Server only
cargo test
```
</details>

## License

MIT — Built on [tantivy](https://github.com/quickwit-oss/tantivy)
