# JavaScript / TypeScript Client

The `@tna5/tantex` npm package provides a high-level HTTP client (and optional Unix socket client for local high-throughput ingestion).

**Supported runtimes:** Node.js ≥ 18, Bun ≥ 1.0.

---

## Installation

::: code-group

```bash [npm]
npm install @tna5/tantex
```

```bash [bun]
bun add @tna5/tantex
```

:::

---

## Quick start

```ts
import { Tantex } from '@tna5/tantex'

// Create a client (HTTP by default)
const client = new Tantex({
  url: 'http://localhost:7200',
  apiKey: 'tant2_...',  // optional, only if server has keys configured
})

// Create an index
await client.createIndex('articles', {
  fields: [
    { name: 'id',    type: 'u64',  stored: true, fast: true },
    { name: 'title', type: 'text', stored: true },
    { name: 'body',  type: 'text', stored: true },
  ],
  compression: 'zstd:3',
})

// Ingest documents
await client.ingest('articles', [
  { id: 1, title: 'Hello world', body: 'First post.' },
  { id: 2, title: 'Second post',  body: 'Another post.' },
])

// Commit (make docs visible to searches)
await client.commit('articles')

// Search
const results = await client.search('articles', 'world', { limit: 10 })
console.log(results.hits)
// [
//   {
//     score: 2.15,
//     doc: { id: 1, title: 'Hello world', body: 'First post.' }
//   }
// ]
```

---

## Index management

### Create an index

```ts
await client.createIndex('products', {
  fields: [
    { name: 'id',       type: 'u64',  stored: true, indexed: true, fast: true },
    { name: 'name',     type: 'text', stored: true, indexed: true },
    { name: 'category', type: 'text', stored: true, indexed: true, tokenizer: 'raw' },
    { name: 'price',    type: 'f64',  stored: true, indexed: false, fast: true },
  ],
  compression: 'lz4',
  block_size: 16384,
})
```

Field types: `text`, `u64`, `i64`, `f64`, `date`, `bool`, `bytes`, `json`.

See [Field types](../reference/field-types.md) for details.

### List indexes

```ts
const { indexes } = await client.listIndexes()
console.log(indexes.map(i => i.name))
```

### Get an index

```ts
const info = await client.getIndex('articles')
console.log(info.doc_count, info.num_segments)
```

### Delete an index

```ts
await client.deleteIndex('articles')
```

---

## Ingest documents

### Simple batch ingest

```ts
const docs = [
  { id: 1, title: 'Hello', body: 'World' },
  { id: 2, title: 'Rust',  body: 'is fast' },
]

const result = await client.ingest('articles', docs, {
  batchSize: 1000,  // documents per request
  onProgress: (indexed, total) => {
    console.log(`${indexed} / ${total} indexed`)
  },
})

console.log(result)
// { indexed: 2, errors: [] }
```

### Stream large files

For large files or streaming pipelines, use `ingestStream` to keep memory bounded:

```ts
import { createReadStream } from 'node:fs'
import { createInterface } from 'node:readline'

async function* readNdjson(path) {
  const rl = createInterface({ input: createReadStream(path) })
  for await (const line of rl) {
    if (line.trim()) yield JSON.parse(line)
  }
}

const result = await client.ingestStream(
  'articles',
  readNdjson('./dump.ndjson'),
  { batchSize: 5000 }
)
```

### Making documents searchable

Documents ingested via HTTP are buffered. They become visible to searches only after a commit — either an explicit call, or the auto-commit timer fires.

```ts
await client.commit('articles')
```

---

## Search

```ts
const results = await client.search('articles', 'rust performance', {
  limit: 20,
  offset: 0,
})

console.log(`${results.total_hits} total hits (${results.elapsed_us} µs)`)
for (const hit of results.hits) {
  console.log(hit.score.toFixed(2), hit.doc.title)
}
```

Query syntax: full-text, field-scoped, phrase, boolean, range, wildcard, boost. See [Query syntax](../reference/query-syntax.md).

---

## Error handling

All methods throw `TantexError` on HTTP errors.

```ts
import { Tantex, TantexError } from '@tna5/tantex'

try {
  await client.search('missing-index', 'query')
} catch (err) {
  if (err instanceof TantexError) {
    console.error(err.status, err.message)  // 404, "index not found"
  }
}
```

---

## Configuration and metrics

### Get server config

```ts
const config = await client.getConfig()
console.log(config.auto_commit_doc_count, config.num_indexing_threads)
```

### Update server config

```ts
await client.setConfig({
  auto_commit_doc_count: 500_000,
  auto_commit_interval_secs: 10,
})
```

### Get metrics

```ts
const metrics = await client.getMetrics()
console.log(metrics.totalDocs, metrics.totalIndexes)
```

---

## Authentication

If the server requires authentication, pass the API key in the constructor via the `apiKey` option:

```ts
const client = new Tantex({
  url: 'https://search.example.com',
  apiKey: 'your-secret-key-here',
})
```

The key is sent as an `X-Api-Key` header with each request. See [Configuration — Authentication](../reference/configuration.md#api_key) to set a key on the server.

---

## High-throughput local ingestion

For maximum ingest speed on a local machine with a Unix socket, use `TantexSocketClient` (binary protocol, MessagePack framing):

```ts
import { TantexSocketClient } from '@tna5/tantex'

const client = new TantexSocketClient('/tmp/tantex.sock')
await client.connect()

await client.createIndex('bench', {
  fields: [
    { name: 'id',    type: 'u64', fast: true },
    { name: 'title', type: 'text' },
  ],
})

// Achieves several million docs/sec
const result = await client.ingest('bench', docs, {
  batchSize: 5000,
  onProgress: (n) => console.log(`${n} indexed`),
})

await client.commit('bench')
client.close()
```

**Tip:** Open multiple `TantexSocketClient` connections and run them in parallel to saturate all indexing threads.

---

## Types

Full TypeScript definitions are included:

```ts
import type {
  TantexOptions,
  SchemaDefinition,
  FieldDefinition,
  FieldType,
  IndexInfo,
  IndexDetails,
  SearchResult,
  SearchHit,
  IngestResult,
  ServerConfig,
  ApiKeyInfo,
} from '@tna5/tantex'
```
