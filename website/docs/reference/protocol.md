# Binary Protocol Reference

The Unix Domain Socket protocol is tantex's high-throughput interface. It uses a simple binary framing layer over MessagePack-serialised payloads.

---

## Frame format

Every message — both requests from the client and responses from the server — uses the same frame structure:

```
┌──────────────────────┬────────────┬─────────────────────────────────┐
│  Length (4 bytes LE) │ Type (1 B) │ Payload (variable, MessagePack)  │
└──────────────────────┴────────────┴─────────────────────────────────┘
```

| Field | Size | Description |
|---|---|---|
| Length | 4 bytes, little-endian `u32` | Total byte count of **Type + Payload** (not including these 4 bytes). |
| Type | 1 byte | Message type constant (see table below). |
| Payload | `Length - 1` bytes | MessagePack-encoded request or response struct. |

### Example — encoding a `MSG_LIST_INDEXES` request

```
04 00 00 00   // Length = 4 (1 type byte + 3 payload bytes)
03            // MSG_LIST_INDEXES = 0x03
80            // MessagePack encoding of {} (empty map)
```

---

## Message type constants

### Requests

| Constant | Value | Description |
|---|---|---|
| `MSG_CREATE_INDEX` | `0x01` | Create a new index |
| `MSG_DELETE_INDEX` | `0x02` | Delete an index |
| `MSG_LIST_INDEXES` | `0x03` | List all indexes |
| `MSG_GET_INDEX` | `0x04` | Get details for one index |
| `MSG_GET_SEGMENTS` | `0x05` | List segments for one index |
| `MSG_INGEST_BATCH` | `0x10` | Ingest a batch of JSON documents |
| `MSG_INIT_SHM` | `0x11` | Initialise a shared-memory session |
| `MSG_INGEST_SHM` | `0x12` | Ingest documents from shared memory |
| `MSG_CLOSE_SHM` | `0x13` | Close and release a shared-memory session |
| `MSG_SEARCH` | `0x20` | Execute a search query |
| `MSG_COMMIT` | `0x21` | Force a commit on an index |
| `MSG_GET_CONFIG` | `0x30` | Read the current configuration |
| `MSG_SET_CONFIG` | `0x31` | Update tunable configuration values |

### Responses

| Constant | Value | Description |
|---|---|---|
| `MSG_RESPONSE_OK` | `0x80` | Success response; payload is the operation-specific response struct. |
| `MSG_RESPONSE_ERR` | `0x81` | Error response; payload is `{ "code": u16, "message": string }`. |

---

## Request / response payloads

All payloads are MessagePack maps. Field names match the JSON keys shown here.

### `MSG_CREATE_INDEX` (0x01)

**Request:**
```json
{
  "name": "articles",
  "schema": {
    "fields": [
      { "name": "id", "type": "u64", "stored": true, "indexed": true, "fast": true, "tokenizer": "default" }
    ],
    "compression": "lz4",
    "block_size": 16384
  }
}
```

**Response (OK):**
```json
{ "success": true, "field_ids": { "id": 0 } }
```

---

### `MSG_DELETE_INDEX` (0x02)

**Request:** `{ "name": "articles" }`

**Response (OK):** `{ "success": true }`

---

### `MSG_LIST_INDEXES` (0x03)

**Request:** `{}`

**Response (OK):**
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

### `MSG_GET_INDEX` (0x04)

**Request:** `{ "name": "articles" }`

**Response (OK):** Same as `GET /api/indexes/{name}`.

---

### `MSG_GET_SEGMENTS` (0x05)

**Request:** `{ "name": "articles" }`

**Response (OK):**
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

### `MSG_INGEST_BATCH` (0x10)

Send an in-memory batch of JSON documents. Suitable for moderate document counts; for millions of documents per second, prefer the SHM path.

**Request:**
```json
{
  "index": "articles",
  "documents": [
    { "id": 1, "title": "Hello world", "body": "First document." },
    { "id": 2, "title": "Second doc",  "body": "Another document." }
  ]
}
```

**Response (OK):**
```json
{ "indexed": 2, "errors": [] }
```

`errors` is an array of per-document error strings. A partial success (some documents indexed, others rejected) returns `MSG_RESPONSE_OK` with the `errors` field populated.

---

### `MSG_INIT_SHM` (0x11)

Allocate a shared-memory buffer. The file is created at `/tmp/tantex_shm_{session_id}`. After this call the client should open and mmap the same path.

**Request:**
```json
{ "buffer_size": 268435456 }
```

**Response (OK):**
```json
{ "shm_path": "/tmp/tantex_shm_a1b2c3d4..." }
```

---

### `MSG_INGEST_SHM` (0x12)

Tell the server to read and index NDJSON from the shared-memory buffer.

**Request:**
```json
{
  "index": "articles",
  "length": 1048576,
  "doc_count": 10000
}
```

| Field | Description |
|---|---|
| `index` | Index name to write to. |
| `length` | Number of bytes to read from offset 0 of the SHM buffer. |
| `doc_count` | Expected number of newline-delimited JSON documents. Used for progress tracking only; the server will parse as many lines as `length` contains. |

**Response (OK):**
```json
{ "indexed": 10000, "errors": [] }
```

---

### `MSG_CLOSE_SHM` (0x13)

Release the shared-memory buffer. The server deletes the backing file.

**Request:** `{}`

**Response (OK):** `{ "success": true }`

---

### `MSG_SEARCH` (0x20)

**Request:**
```json
{
  "index": "articles",
  "query": "rust performance",
  "limit": 10,
  "offset": 0
}
```

`limit` defaults to `10`. `offset` defaults to `0`.

**Response (OK):**
```json
{
  "total_hits": 14,
  "hits": [
    { "score": 3.42, "doc": { "id": 7, "title": "...", "body": "..." } }
  ],
  "elapsed_us": 212
}
```

---

### `MSG_COMMIT` (0x21)

**Request:** `{ "index": "articles" }`

**Response (OK):** `{ "success": true }`

---

### `MSG_GET_CONFIG` (0x30)

**Request:** `{}`

**Response (OK):** Full `ConfigResponse` struct (same as `GET /api/config`).

---

### `MSG_SET_CONFIG` (0x31)

**Request:** Partial config — include only the fields to update.

```json
{ "auto_commit_doc_count": 500000 }
```

**Response (OK):** `{ "success": true }`

---

## Error response

When the server returns `MSG_RESPONSE_ERR` (0x81), the payload is:

```json
{ "code": 404, "message": "Index 'articles' not found" }
```

`code` is an HTTP-style status code (400 for client errors, 500 for server errors, etc.).

---

## Connection lifecycle

1. Client opens a Unix domain socket connection to `TANTEX_SOCKET_PATH`.
2. Client sends requests one at a time. The protocol is strictly sequential — each request must receive a response before the next request can be sent.
3. Client closes the connection when done.

Each connection maintains its own optional SHM session. If the connection is closed while a SHM session is active, the server releases the buffer automatically.
