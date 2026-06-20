# Getting Started with Tantex

This tutorial walks you from zero to a working search index in about ten minutes. By the end you will have:

- Installed the tantex server
- Created an index with a custom schema
- Ingested a small batch of documents
- Run a full-text search and read the results

**Prerequisites:** `curl` and `jq` (optional, for pretty-printing JSON).

> **New to tantex?** Start with [Installation](./installation.md) to download or build the binary.

---

## 1. Start the server

```sh
./target/release/tantex
```

You should see output similar to:

```
[INFO] HTTP server listening on :7200
```

The server opens two interfaces simultaneously:

- **HTTP** on `http://127.0.0.1:7200` — the REST API and embedded dashboard
- **Unix socket** at `/tmp/tantex.sock` — the binary protocol for high-throughput clients

The dashboard is available at [http://localhost:7200](http://localhost:7200).

---

## 2. Create your first index

An index requires a **schema** — a list of fields with their types and options. Send a `POST` request to create one:

```sh
curl -s -X POST http://localhost:7200/api/indexes \
  -H 'Content-Type: application/json' \
  -d '{
    "name": "articles",
    "schema": {
      "fields": [
        { "name": "id",      "type": "u64",  "stored": true, "indexed": true, "fast": true },
        { "name": "title",   "type": "text", "stored": true, "indexed": true },
        { "name": "body",    "type": "text", "stored": true, "indexed": true },
        { "name": "author",  "type": "text", "stored": true, "indexed": true, "tokenizer": "raw" },
        { "name": "published_at", "type": "date", "stored": true, "indexed": true, "fast": true }
      ]
    }
  }'
```

A successful response looks like:

```json
{
  "success": true,
  "field_ids": { "id": 0, "title": 1, "body": 2, "author": 3, "published_at": 4 }
}
```

> **Tip:** The `author` field uses the `"raw"` tokenizer, which indexes the value as a single token (no splitting). This is ideal for exact-match filters on categorical values.

---

## 3. Ingest documents

Use the JavaScript client to ingest documents:

::: code-group

```bash [npm]
npm install @tna5/tantex
```

```bash [bun]
bun add @tna5/tantex
```

:::

Create a script `ingest.js`:

```ts
import { Tantex } from '@tna5/tantex'

const client = new Tantex()

// Create an index
await client.createIndex('articles', {
  fields: [
    { name: 'id',    type: 'u64', fast: true },
    { name: 'title', type: 'text' },
    { name: 'body',  type: 'text' },
  ],
})

// Ingest documents
const docs = [
  { id: 1, title: 'Rust performance', body: 'Benchmarks...' },
  { id: 2, title: 'Full text search',  body: 'How it works...' },
]

await client.ingest('articles', docs)

// Make them searchable
await client.commit('articles')

console.log('Done!')
```

Run it:

```sh
node ingest.js
# or: bun ingest.js
```

See [JavaScript client](../how-to/javascript-client.md) for more examples (streaming, high-throughput ingestion, search).

---

## 4. Run a search

Once documents are ingested and committed, search them:

```sh
curl -s -X POST http://localhost:7200/api/indexes/articles/search \
  -H 'Content-Type: application/json' \
  -d '{"query": "rust performance", "limit": 5}' | jq .
```

Response:

```json
{
  "total_hits": 14,
  "hits": [
    {
      "score": 3.42,
      "doc": {
        "id": 7,
        "title": "Rust performance tips",
        "body": "...",
        "author": "alice",
        "published_at": "2024-03-15T10:00:00Z"
      }
    }
  ],
  "elapsed_us": 212
}
```

- `total_hits` — total number of matching documents across all pages
- `hits` — the current page, each entry with a BM25 `score` and the stored fields in `doc`
- `elapsed_us` — server-side query latency in microseconds (excludes HTTP overhead)

---

## 5. Explore the dashboard

Open [http://localhost:7200](http://localhost:7200) in your browser. You will see:

- **Dashboard** — live document counts and ingest rate per index via Server-Sent Events
- **Indexes** — create, browse, and delete indexes; view schema and run searches
- **Metrics** — historical ingest rate graphs
- **Settings** — tune commit policy, thread counts, and merge parameters at runtime

---

## Next steps

- [Design a schema](../how-to/design-schema.md) — understand field types, tokenizers, and storage options
- [Ingest documents at high speed](../how-to/ingest-documents.md) — use shared-memory ingest for millions of docs/sec
- [Configuration reference](../reference/configuration.md) — tune memory, threads, and commit behaviour
- [Query syntax](../reference/query-syntax.md) — boolean operators, phrase queries, range filters
