# Shared-Memory Ingestion

This document explains how the shared-memory (SHM) ingest path works, what makes it fast, and when to use it.

---

## The problem with conventional ingest

In a conventional ingest pipeline, documents travel through multiple serialisation steps:

```
Application memory
  → JSON string
    → MessagePack encoding
      → Unix socket write
        → Kernel copy (socket buffer)
          → Server reads from socket
            → MessagePack decode
              → JSON parse
                → tantivy document
```

Each arrow represents a copy or transform. At high throughput (millions of documents per second), these copies become the bottleneck — not the disk or the indexing engine.

---

## The SHM approach

Shared memory eliminates the socket transfer entirely. The client and server map the **same file** into their respective address spaces. The client writes NDJSON directly; the server reads it in place — no bytes are copied across the socket, only a small control message is sent.

```
Application memory
  → NDJSON write into mmap'd file    (client side)
                │
                └──── same physical pages ────┐
                                              │
                                    Server reads from mmap
                                      → JSON parse (sonic-rs)
                                        → tantivy document
```

---

## Step-by-step flow

### 1. `MSG_INIT_SHM`

The client sends a buffer size (bytes). The server:
1. Generates a session ID (UUID v4)
2. Creates a file at `/tmp/tantex_shm_{session_id}`
3. Opens and mmap's the file (read-write, length = `buffer_size`)
4. Returns `{ "shm_path": "/tmp/tantex_shm_..." }` to the client

### 2. Client opens the same file

The client independently opens the path returned in step 1 and mmap's it for writing. In the Bun client, this uses FFI calls to `mmap` via libSystem:

```js
const session = await client.initShm(256 * 1024 * 1024)
// session.write(bytes, offset) maps directly into the shared pages
```

### 3. `MSG_INGEST_SHM`

The client writes one or more batches of NDJSON (newline-delimited JSON, one document per line) into the buffer starting at offset 0, then sends:

```json
{ "index": "articles", "length": 1048576, "doc_count": 10000 }
```

The server:
1. Reads `length` bytes from offset 0 of its mmap view
2. Splits on newlines
3. Parses each line with **sonic-rs** (a high-performance Rust JSON parser)
4. Feeds each parsed document to the index writer via the `WriterCommand::AddDocumentsFromShm` channel message
5. Returns `{ "indexed": N, "errors": [] }`

### 4. Repeat

The client overwrites the buffer with the next batch and sends another `MSG_INGEST_SHM`. The buffer is reused in-place — no reallocation needed between batches.

### 5. `MSG_CLOSE_SHM`

When the client is done, it sends `MSG_CLOSE_SHM`. The server drops the `ShmBuffer` struct, which:
1. Calls `munmap` on the server's view
2. Deletes the backing file from `/tmp/`

The client should also unmap its own view after this call.

---

## Why it is fast

| Factor | Effect |
|---|---|
| Zero socket copy | The payload bytes never travel through the kernel socket buffer |
| sonic-rs JSON parser | 2–3× faster JSON parsing than serde_json for simple objects |
| Writer thread pipeline | The async handler sends a channel message and awaits a oneshot; the writer thread batches documents into tantivy without holding any async lock |
| No MessagePack encoding/decoding of documents | Documents stay as raw UTF-8 NDJSON until the server parses them |

On Apple M2 hardware with a warm OS cache, the SHM path achieves ~2–5 million documents per second for typical 100-500 byte documents.

---

## Constraints and limitations

- **One SHM session per connection.** Open separate connections for parallel SHM sessions.
- **Buffer size limit.** Each `MSG_INGEST_SHM` call cannot exceed the `buffer_size` requested in `MSG_INIT_SHM` (default 256 MB).
- **The buffer is not persistent.** The backing file is in `/tmp/` and is deleted when the session closes or the server shuts down.
- **NDJSON only.** The buffer format is one JSON object per line, UTF-8 encoded. Malformed lines produce per-document errors in the response; the rest of the batch continues.
- **Bun only (for the mmap client).** Node.js does not have built-in Unix mmap support. The `MSG_INGEST_BATCH` path (which goes over the socket) works with any runtime.

---

## When to use SHM vs batch ingest

Use **SHM** when:
- You are loading a large dataset (millions of documents)
- Your client can use Bun or a language with native mmap support
- Ingest throughput is the primary concern

Use **batch ingest** when:
- Your client is a language without easy mmap support
- You are ingesting a few thousand documents at a time
- Simplicity is more important than maximum throughput
