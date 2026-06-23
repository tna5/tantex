# How to Design a Schema

A schema defines the fields of an index. Once created, a schema cannot be changed — you must delete and recreate the index to alter it. Take time to design it carefully before ingesting large amounts of data.

---

## Decide what you need from each field

For each field, ask:

1. **Do I need to retrieve the raw value in search results?** → `"stored": true`
2. **Do I need to search or filter on this field?** → `"indexed": true`
3. **Do I need to sort by this field, use range filters, or aggregate on it?** → `"fast": true`

Setting `stored: false` and `indexed: false` on a field makes no sense — the field would neither be retrievable nor searchable. Omit it from the schema instead.

---

## Choosing a field type

| Use case | Recommended type |
|---|---|
| Article body, product description | `text` |
| Unique ID, integer counter | `u64` |
| Temperature, price, score | `f64` |
| Event timestamp, creation date | `date` |
| Boolean flag | `bool` |
| Exact-match category, slug | `text` with `"tokenizer": "raw"` |
| Structured metadata | `json` |
| Binary attachment | `bytes` |
| IP address (v4 or v6) | `ip` |
| Tags, category list | `array<text>` with `"tokenizer": "raw"` |
| Multiple numeric IDs | `array<u64>` |

See [Field types reference](../reference/field-types.md) for the complete list.

---

## When to use `fast: true`

Fast fields build a column-oriented store (docvalues) alongside the inverted index. They are required for:

- **Range filters on numbers and dates:** `score:[1.0 TO 5.0]`, `created_at:[2024-01-01T00:00:00Z TO *]`
- **Sorting results** by a field (not yet exposed via the HTTP API but used internally)

Fast fields increase index size and slow down ingest slightly. Only enable them on fields you actually need for filtering or sorting.

---

## Choosing a tokenizer for text fields

| Tokenizer | Use when |
|---|---|
| `default` | General-purpose full-text search on natural language |
| `raw` | Exact-match on structured values (categories, slugs, identifiers) |
| `raw_lower` | Exact case-insensitive match (hostnames, log levels) |
| `sorted` | Name/phrase search where word order doesn't matter (`"Jean Dupont"` = `"Dupont Jean"`) |
| `en_stem` | English text where you want `running` to match `run` |
| `whitespace` | Tokenise by space only, preserve casing |
| `ngram` | Prefix search or autocomplete (generates character n-grams) |

> Use `"tokenizer": "raw"` on fields you will filter with exact equality. Applying `default` to a category field would make `electronics` and `consumer-electronics` match the query `electronics`.

---

## Per-path field mapping for `json` fields

When sub-paths of a json field need individual control over type, tokenizer, or indexing, use `fields` with `"indexed": false` on the json field itself. Tantex creates one internal field per declared path and routes values during ingest. Queries on `field.path:value` are rewritten transparently.

```json
{
  "name": "extra",
  "type": "json",
  "stored": true,
  "indexed": false,
  "fields": {
    "service":     { "type": "text", "tokenizer": "raw" },
    "host":        { "type": "text", "tokenizer": "raw" },
    "status_code": { "type": "u64",  "indexed": true, "fast": true },
    "message":     { "type": "text", "tokenizer": "default" }
  }
}
```

- `extra.service:nginx` — exact match on service name
- `extra.host:web-01` — exact match on hostname
- `extra.status_code:[500 TO *]` — range filter on numeric status code
- `extra.message:"connection refused"` — full-text search in log message

See [Per-path field mapping for json](../reference/field-types.md#per-path-field-mapping-for-json) for the full sub-field property reference.

---

## Example schemas

### Blog posts

```json
{
  "fields": [
    { "name": "id",           "type": "u64",  "stored": true,  "indexed": true,  "fast": true  },
    { "name": "title",        "type": "text", "stored": true,  "indexed": true  },
    { "name": "body",         "type": "text", "stored": true,  "indexed": true  },
    { "name": "author",       "type": "text", "stored": true,  "indexed": true,  "tokenizer": "raw" },
    { "name": "tags",         "type": "array<text>", "stored": true, "indexed": true, "tokenizer": "raw" },
    { "name": "published_at", "type": "date", "stored": true,  "indexed": true,  "fast": true  },
    { "name": "published",    "type": "bool", "stored": true,  "indexed": true  }
  ],
  "compression": "lz4"
}
```

### Product catalogue

```json
{
  "fields": [
    { "name": "sku",         "type": "text",  "stored": true,  "indexed": true, "tokenizer": "raw" },
    { "name": "name",        "type": "text",  "stored": true,  "indexed": true  },
    { "name": "description", "type": "text",  "stored": true,  "indexed": true  },
    { "name": "category",    "type": "text",  "stored": true,  "indexed": true, "tokenizer": "raw" },
    { "name": "price_cents", "type": "u64",   "stored": true,  "indexed": true, "fast": true  },
    { "name": "in_stock",    "type": "bool",  "stored": true,  "indexed": true  },
    { "name": "metadata",    "type": "json",  "stored": true,  "indexed": true  }
  ]
}
```

### Access logs

```json
{
  "fields": [
    { "name": "timestamp",   "type": "date", "stored": true, "indexed": true, "fast": true },
    { "name": "client_ip",   "type": "ip",   "stored": true, "indexed": true, "fast": true },
    { "name": "method",      "type": "text", "stored": true, "indexed": true, "tokenizer": "raw" },
    { "name": "path",        "type": "text", "stored": true, "indexed": true  },
    { "name": "status_code", "type": "u64",  "stored": true, "indexed": true, "fast": true },
    { "name": "latency_ms",  "type": "u64",  "stored": true, "fast": true  }
  ],
  "compression": "zstd:3"
}
```

---

## Storage options

### Compression

The `compression` field applies to the **doc store** (the stored fields) only. It does not affect the inverted index.

| Value | Trade-off |
|---|---|
| `"lz4"` (default) | Very fast compression/decompression, good ratio |
| `"zstd"` | Better ratio than lz4, slightly slower |
| `"zstd:3"` | Balanced (level 3 is a common sweet spot) |
| `"zstd:9"` | High ratio, noticeably slower retrieval |
| `"none"` | No compression — fastest retrieval, largest files |

### Block size

`block_size` is the number of bytes per compressed block in the doc store (default `16384` / 16 KB).

- **Larger blocks** → better compression ratio, slower random document retrieval (the whole block is decompressed to read one doc)
- **Smaller blocks** → faster retrieval, worse compression

For write-heavy workloads where retrieval latency matters less, try `65536` or `131072`.
