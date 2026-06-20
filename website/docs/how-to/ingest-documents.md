# How to Ingest Documents

This guide covers the two ingest paths available in tantex and when to choose each.

---

## Batch ingest via the Unix socket (`MSG_INGEST_BATCH`)

Send a JSON array of documents in a single `MSG_INGEST_BATCH` message. Each element must match the index schema.

### JavaScript (Bun/Node)

```js
import { Tantex } from '@tna5/tantex'

const client = new Tantex()
// HTTP (default): new Tantex({ url: 'http://localhost:7200' })

const docs = [
  { id: 1, title: 'Hello world',  body: 'First document.' },
  { id: 2, title: 'Second doc',   body: 'Another document.' },
]

const result = await client.ingest('articles', docs)
console.log(result)
// { indexed: 2, errors: [] }
```

### Guidelines for batch ingest

- **Batch size:** 1 000 – 10 000 documents per call is a good default. Larger batches reduce per-message overhead but increase memory pressure during MessagePack serialisation.
- **Commit:** Documents ingested via the socket are buffered. They become visible to searches only after a commit — either an explicit `MSG_COMMIT`, or the auto-commit timer/threshold fires. Call `commit` after the final batch if you need immediate visibility.

```js
await client.commit('articles')
```

---

## High-speed ingest via shared memory (`MSG_INGEST_SHM`)

The SHM path achieves sustained ingest rates of several million documents per second by eliminating serialisation: the client writes NDJSON directly into a memory-mapped file; the server reads from the same file without any copying.

### Step-by-step

1. **Initialise the SHM session.** The server creates a memory-mapped file and returns its path.

```js
const session = await client.initShm(256 * 1024 * 1024) // 256 MB buffer
```

2. **Open the same file from the client side.** The `ShmSession` returned by `initShm` handles this automatically via Bun FFI (`mmap`). The client writes NDJSON into the buffer.

3. **Signal the server.** Call `ingestFromBuffer` (or `MSG_INGEST_SHM`) with the index name and the number of bytes written.

```js
// Build a batch of NDJSON documents into the SHM buffer
const encoder = new TextEncoder()
let offset = 0

for (let i = 0; i < 10000; i++) {
  const doc = { id: i, title: `Document ${i}`, body: `Body text ${i}` }
  const line = JSON.stringify(doc) + '\n'
  const bytes = encoder.encode(line)
  session.write(bytes, offset)
  offset += bytes.length
}

// Tell the server to read and index the buffer
const result = await session.ingest('articles', offset, 10000)
console.log(result) // { indexed: 10000, errors: [] }
```

4. **Repeat** steps 2–3 for successive batches. The buffer is reused between calls.

5. **Close the session** when finished.

```js
await client.closeShm()
session.close()
```

### SHM constraints

- The total bytes written per `MSG_INGEST_SHM` call must not exceed `TANTEX_SHM_BUFFER_SIZE` (default 256 MB).
- One SHM session per connection. Open a new connection for parallel sessions.
- The NDJSON in the buffer must be valid UTF-8 and newline-delimited (one JSON object per line, no trailing commas).

---

## Choosing between batch and SHM

| Criterion | `MSG_INGEST_BATCH` | `MSG_INGEST_SHM` |
|---|---|---|
| Ease of use | High — just send a JSON array | Medium — requires mmap on the client |
| Throughput | ~200 k – 500 k docs/sec | ~2 M – 5 M docs/sec |
| Memory pressure | Higher (serialised twice) | Lower (zero-copy) |
| Language support | Any (via the binary protocol) | Languages with FFI/mmap support |
| Suitable for | Moderate volumes, arbitrary clients | Bulk historical loads, high-throughput pipelines |

---

## Making documents visible

Ingested documents are **not immediately searchable**. They become visible after a commit. Tantex commits automatically based on:

- **Document count:** when `auto_commit_doc_count` pending documents accumulate and the writer is idle
- **Time:** after `auto_commit_interval_secs` seconds of inactivity
- **Hard limit:** when pending docs reach `auto_commit_doc_count × hard_commit_multiplier`, even under continuous ingest

To force an immediate commit:

```js
await client.commit('articles')
```

Or via HTTP:

```sh
curl -X POST http://localhost:7200/api/indexes/articles/commit
```

See [Commit and merge policy](../explanation/commit-merge-policy.md) for the full explanation.
