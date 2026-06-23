# Field Types Reference

Every field in a tantex schema has a `type` that determines how it is indexed, stored, and queried. The field definition has the following shape:

```json
{
  "name": "field_name",
  "type": "text",
  "stored": true,
  "indexed": true,
  "fast": false,
  "tokenizer": "default"
}
```

| Property | Default | Description |
|---|---|---|
| `name` | — | Field name (required). Used as the key in JSON documents. |
| `type` | — | Field type (required). See table below. |
| `stored` | `true` | Whether the raw value is kept in the doc store and returned in search hits. |
| `indexed` | `true` | Whether the value is indexed for search/filtering. Set to `false` for store-only fields. |
| `fast` | `false` | Whether to build a column-oriented index (docvalues). Required for sorting, aggregations, and range filters on numeric/date fields. |
| `tokenizer` | `"default"` | Tokenizer to use. Only meaningful for `text`, `json`, `array<text>`, and `array<json>` fields. |
| `fields` | — | **`json` only.** Per-path sub-field definitions. Each key is a sub-path; value is a sub-field definition with `type`, `tokenizer`, `stored`, `indexed`, `fast`. See [Per-path field mapping for json](#per-path-field-mapping-for-json). |

---

## Scalar types

### `text`

Full-text indexed string. The value is tokenized before indexing.

```json
{ "name": "body", "type": "text", "stored": true, "indexed": true }
```

**Tokenizers:**

| Tokenizer | Behaviour |
|---|---|
| `default` | Lowercase + split on whitespace and punctuation |
| `raw` | Index the entire value as a single token (exact match) |
| `raw_lower` | Index the entire value as one token, lowercased — exact case-insensitive match (hostnames, log levels) |
| `en_stem` | English stemming (`running` → `run`) |
| `whitespace` | Split on whitespace only, no case folding |
| `ngram` | Character n-grams (prefix/substring search) |
| `sorted` | Word-splits, lowercases, sorts tokens alphabetically, then re-emits at positions 0…n. Phrase queries become order-independent: `"Jean Dupont"` matches `"Dupont Jean"`. Cross-element false positives are prevented by tantivy's per-value position gap. |

---

### `u64`

Unsigned 64-bit integer.

```json
{ "name": "views", "type": "u64", "stored": true, "indexed": true, "fast": true }
```

Use `fast: true` to enable sorting and range queries (`views:[100 TO *]`).

---

### `i64`

Signed 64-bit integer. Same options as `u64`.

```json
{ "name": "temperature", "type": "i64", "stored": true, "indexed": true, "fast": true }
```

---

### `f64`

64-bit floating-point number. Same options as `u64`.

```json
{ "name": "score", "type": "f64", "stored": true, "fast": true }
```

---

### `date`

RFC 3339 / ISO 8601 datetime string (e.g. `"2024-03-15T10:00:00Z"`). Stored and queried as a UTC timestamp.

```json
{ "name": "published_at", "type": "date", "stored": true, "indexed": true, "fast": true }
```

Range queries use the tantivy date syntax: `published_at:[2024-01-01T00:00:00Z TO 2024-12-31T23:59:59Z]`.

---

### `bool`

Boolean value (`true` / `false`).

```json
{ "name": "active", "type": "bool", "stored": true, "indexed": true }
```

---

### `bytes`

Raw binary blob. Only meaningful as a stored field; not full-text indexed.

```json
{ "name": "thumbnail", "type": "bytes", "stored": true, "indexed": false }
```

Documents provide the value as a base64-encoded string.

---

### `json`

Semi-structured JSON object. All string leaf values are tokenized and indexed; numeric and boolean leaves are stored. Nested paths use `.` separator in queries.

```json
{ "name": "metadata", "type": "json", "stored": true, "indexed": true }
```

Query a nested key: `metadata.color:blue`.

#### Per-path field mapping for json

When sub-paths of a json field need individual control over type, tokenizer, or indexing, use `fields` instead of a single `tokenizer`. Set `"indexed": false` on the json field itself; tantex creates one internal field per declared path and routes values at ingest time.

Each entry in `fields` is a sub-field definition with the following properties (all optional):

| Property | Default | Description |
|---|---|---|
| `type` | `"text"` | Field type: `text`, `u64`, `i64`, `f64`, `date`, `bool`, `ip` |
| `tokenizer` | `"default"` | Tokenizer name. Only meaningful for `type: "text"`. |
| `stored` | `false` | Store the sub-field value separately. Parent json field usually holds it. |
| `indexed` | `true` | Index the value for search/filtering. |
| `fast` | `false` | Build docvalues. Required for range filters and sorting on numeric/date sub-fields. |

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

At query time, paths are **automatically rewritten** to the internal field names — callers use the natural `field.path:value` syntax unchanged:

```
extra.service:nginx
extra.host:web-01
extra.status_code:[500 TO *]
extra.message:"connection refused"
```

Internally tantex creates one field per declared path (not stored by default). The stored json field is used for document retrieval. Internal field names are never exposed in search results.

---

### `ip`

IPv4 or IPv6 address. Both formats are accepted in documents (`"192.168.1.1"` or `"::1"`); IPv4 addresses are stored internally as IPv4-mapped IPv6 addresses (`::ffff:192.168.1.1`).

```json
{ "name": "client_ip", "type": "ip", "stored": true, "indexed": true, "fast": true }
```

---

## Array types

Tantex supports multi-valued fields using the `array<type>` prefix. At index time, tantex calls `add_field_value` once per element, so any number of values can be stored under a single field name.

In a JSON document, supply an array:

```json
{ "tags": ["rust", "search", "performance"] }
```

### `array<text>`

Multiple full-text values. All elements are tokenized and contribute to the same inverted index.

```json
{ "name": "tags", "type": "array<text>", "stored": true, "indexed": true, "tokenizer": "raw" }
```

### `array<u64>` / `array<i64>` / `array<f64>`

Multiple numeric values. Useful for storing and filtering on sets of numbers (e.g., multiple category IDs).

```json
{ "name": "category_ids", "type": "array<u64>", "stored": true, "indexed": true, "fast": true }
```

### `array<date>`

Multiple date values.

```json
{ "name": "event_dates", "type": "array<date>", "stored": true, "indexed": true }
```

### `array<bool>`

Multiple boolean values (uncommon but supported).

### `array<ip>`

Multiple IP addresses. Useful for fields such as `"alternate_ips"`.

```json
{ "name": "resolved_ips", "type": "array<ip>", "stored": true, "indexed": true }
```

---

## Notes

- Array types use the same underlying tantivy field type as their scalar counterpart. There is no schema distinction between a "single-value text field" and a "multi-value text field" at the storage level — the `array<*>` prefix in tantex is a hint to the ingest layer to iterate over the JSON array.
- Passing a scalar value to an `array<*>` field (or vice versa) is handled gracefully: a scalar is treated as a single-element array.
- The `tokenizer` property is ignored for all non-text types.
