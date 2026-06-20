# HTTP API Reference

The HTTP server listens on `127.0.0.1:7200` by default. All API routes are under `/api/`. Requests and responses use JSON (`Content-Type: application/json`).

---

## Authentication

When an API key is configured, all `/api/*` requests must include it (except `/api/auth/status` and `/api/auth/login`). See [Configuration — Authentication](configuration.md#api_key) for how to set a key.

Pass your key using one of:

```
X-Api-Key: your-secret-key-here
Authorization: Bearer your-secret-key-here
?api_key=your-secret-key-here      (query param — useful for EventSource which cannot set headers)
```

---

## Indexes

### List indexes

`GET /api/indexes`

List all indexes.

**Response:**
```json
{
  "indexes": [
    {
      "name": "articles",
      "doc_count": 1000000,
      "num_segments": 3,
      "pending_docs": 0,
      "search_count": 42,
      "search_latency_us": 318,
      "total_docs_ingested": 1050000
    }
  ]
}
```

---

### Create an index

`POST /api/indexes`

Create a new index.

**Request body:**
```json
{
  "name": "articles",
  "schema": {
    "fields": [
      { "name": "id",    "type": "u64",  "stored": true, "indexed": true, "fast": true },
      { "name": "title", "type": "text", "stored": true, "indexed": true },
      { "name": "body",  "type": "text", "stored": true, "indexed": true }
    ],
    "compression": "lz4",
    "block_size": 16384
  }
}
```

`compression` options: `"none"`, `"lz4"` (default), `"zstd"`, `"zstd:<level>"` (e.g. `"zstd:3"`).  
`block_size`: bytes per compressed block in the doc store (default `16384`).

**Response:**
```json
{
  "success": true,
  "field_ids": { "id": 0, "title": 1, "body": 2 }
}
```

**Errors:** `400 Bad Request` if the name is already in use or the schema is invalid.

---

### Get an index

`GET /api/indexes/{name}`

Get details for a single index.

**Response:**
```json
{
  "name": "articles",
  "schema": {
    "fields": [
      { "name": "id", "type": "u64", "stored": true, "indexed": true, "fast": true, "tokenizer": "default" }
    ],
    "compression": "lz4",
    "block_size": 16384
  },
  "doc_count": 1000000,
  "num_segments": 3,
  "pending_docs": 0,
  "search_count": 42,
  "search_latency_us": 318,
  "total_docs_ingested": 1050000
}
```

**Errors:** `404 Not Found` if the index does not exist.

---

### Delete an index

`DELETE /api/indexes/{name}`

Delete an index and all its data from disk.

**Response:**
```json
{ "success": true }
```

**Errors:** `400 Bad Request` if the index does not exist.

---

### Commit an index

`POST /api/indexes/{name}/commit`

Force an immediate commit on the named index, making all buffered documents visible to searches.

**Response:**
```json
{ "success": true }
```

**Errors:** `400 Bad Request` if the index does not exist.

---

### List segments

`GET /api/indexes/{name}/segments`

List all segments in an index.

**Response:**
```json
{
  "segments": [
    {
      "segment_id": "b3f7a2c1-...",
      "num_docs": 500000,
      "num_deleted_docs": 0,
      "size_bytes": 124456789
    }
  ]
}
```

---

### Search an index

`POST /api/indexes/{name}/search`

Search an index.

**Request body:**
```json
{
  "query": "rust performance",
  "limit": 10,
  "offset": 0
}
```

| Field | Type | Default | Description |
|---|---|---|---|
| `query` | string | required | Tantivy query string. See [Query syntax](query-syntax.md). |
| `limit` | integer | `10` | Maximum number of hits to return. |
| `offset` | integer | `0` | Number of hits to skip (for pagination). |

**Response:**
```json
{
  "total_hits": 14,
  "hits": [
    {
      "score": 3.42,
      "doc": {
        "id": 7,
        "title": "Rust performance tips",
        "body": "..."
      }
    }
  ],
  "elapsed_us": 212
}
```

| Field | Description |
|---|---|
| `total_hits` | Total number of matching documents (across all pages). |
| `hits` | Array of results for the current page. Each hit has a BM25 `score` and a `doc` object containing all stored fields. |
| `elapsed_us` | Server-side query latency in microseconds (excludes HTTP/network overhead). |

**Errors:** `400 Bad Request` for invalid query syntax; `404 Not Found` if the index does not exist.

---

## Metrics

### Get metrics snapshot

`GET /api/metrics`

Snapshot of aggregate metrics.

**Response:**
```json
{
  "totalDocs": 3000000,
  "totalIndexes": 3,
  "totalSegments": 9,
  "totalPendingDocs": 0,
  "indexes": [ ... ]
}
```

---

### Stream metrics (SSE)

`GET /api/metrics/stream`

Server-Sent Events stream. Emits one JSON event per second.

```
data: {"type":"metrics","status":"online","totalDocs":1000000,...}
```

To consume from JavaScript:

```js
const es = new EventSource('http://localhost:7200/api/metrics/stream')
es.onmessage = (e) => {
  const data = JSON.parse(e.data)
  console.log(data.totalDocs)
}
```

When the tantex server cannot be reached, a `status: "offline"` event is emitted.

---

## Configuration {#config}

### Get configuration

`GET /api/config`

Read the current runtime configuration.

**Response:**
```json
{
  "socket_path": "/tmp/tantex.sock",
  "data_dir": "./data",
  "shm_buffer_size": 268435456,
  "writer_heap_size": 4000000000,
  "auto_commit_doc_count": 10000000,
  "auto_commit_interval_secs": 30,
  "merge_target_docs": 20000000,
  "max_merge_factor": 10,
  "min_num_segments": 2,
  "num_indexing_threads": 8,
  "index_threads_pct": 63,
  "hard_commit_multiplier": 4
}
```

---

### Update configuration

`POST /api/config`

Update tunable configuration values at runtime. Omitted fields are not changed. `socket_path`, `data_dir`, and `http_port` cannot be changed at runtime.

**Request body (all fields optional):**
```json
{
  "shm_buffer_size": 268435456,
  "writer_heap_size": 4000000000,
  "auto_commit_doc_count": 1000000,
  "auto_commit_interval_secs": 5,
  "merge_target_docs": 10000000,
  "max_merge_factor": 10,
  "min_num_segments": 2,
  "num_indexing_threads": 8,
  "index_threads_pct": 63,
  "hard_commit_multiplier": 4
}
```

**Response:**
```json
{ "success": true }
```

---

## Authentication

### Check auth status

`GET /api/auth/status`

Returns whether authentication is currently required. This endpoint is always public.

**Response:**
```json
{ "auth_required": false }
```

Authentication is required when an API key is set at server startup.

---

### Login

`POST /api/auth/login`

Authenticate with the API key and receive a session cookie.

**Request body:**
```json
{ "key": "your-secret-key-here" }
```

**Response:**
```json
{ "success": true }
```

Sets an HttpOnly, SameSite=Strict cookie (`tantex_key`) valid for the session.

**Errors:** `401 Unauthorized` if the key is incorrect or missing.

---

### Logout

`POST /api/auth/logout`

Clear the session cookie.

**Response:**
```json
{ "success": true }
```
