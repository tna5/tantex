# How to Search an Index

---

## Basic search via HTTP

```sh
curl -X POST http://localhost:7200/api/indexes/articles/search \
  -H 'Content-Type: application/json' \
  -d '{"query": "rust performance", "limit": 10}'
```

---

## Retrieve all documents

Use the wildcard `*` to match every document:

```sh
curl -X POST http://localhost:7200/api/indexes/articles/search \
  -H 'Content-Type: application/json' \
  -d '{"query": "*", "limit": 100}'
```

---

## Paginate through results

Use `limit` and `offset` together. `total_hits` tells you how many documents match in total.

```sh
# Page 1
curl -s -X POST http://localhost:7200/api/indexes/articles/search \
  -H 'Content-Type: application/json' \
  -d '{"query": "search engine", "limit": 20, "offset": 0}'

# Page 2
curl -s -X POST http://localhost:7200/api/indexes/articles/search \
  -H 'Content-Type: application/json' \
  -d '{"query": "search engine", "limit": 20, "offset": 20}'
```

---

## Filter by a specific field

```sh
# Exact match on a raw-tokenized field
curl -X POST http://localhost:7200/api/indexes/articles/search \
  -H 'Content-Type: application/json' \
  -d '{"query": "author:alice AND title:rust"}'
```

---

## Range filter on a numeric or date field

Requires `fast: true` on the field.

```sh
# Documents with price_cents between 1000 and 5000 (inclusive)
curl -X POST http://localhost:7200/api/indexes/products/search \
  -H 'Content-Type: application/json' \
  -d '{"query": "price_cents:[1000 TO 5000]"}'

# Articles published in 2024
curl -X POST http://localhost:7200/api/indexes/articles/search \
  -H 'Content-Type: application/json' \
  -d '{"query": "published_at:[2024-01-01T00:00:00Z TO 2024-12-31T23:59:59Z]"}'
```

---

## Boolean queries

```sh
# Must contain "rust", must not contain "python"
curl -X POST http://localhost:7200/api/indexes/articles/search \
  -H 'Content-Type: application/json' \
  -d '{"query": "+rust -python"}'

# Either "rust" or "golang" in body, and author is alice
curl -X POST http://localhost:7200/api/indexes/articles/search \
  -H 'Content-Type: application/json' \
  -d '{"query": "(body:rust OR body:golang) AND author:alice"}'
```

---

## Read scores and stored fields

Each hit in the response includes:
- `score` — the BM25 relevance score (higher = more relevant)
- `doc` — a JSON object with all stored fields

```json
{
  "total_hits": 3,
  "hits": [
    {
      "score": 4.21,
      "doc": {
        "id": 42,
        "title": "Rust performance guide",
        "author": "alice",
        "published_at": "2024-06-01T00:00:00Z"
      }
    }
  ],
  "elapsed_us": 187
}
```

`elapsed_us` is the server-side query time in microseconds, measured from the moment tantivy starts parsing the query to the moment the result list is assembled. It excludes HTTP and network overhead.

---

## Search via the JavaScript client

```js
const result = await client.search('articles', '+rust -python', 20, 0)
console.log(result.total_hits, 'hits')

for (const { score, doc } of result.hits) {
  console.log(`[${score.toFixed(2)}] ${doc.title} — ${doc.author}`)
}
```

---

## Query syntax quick reference

See [Query syntax reference](../reference/query-syntax.md) for the full language. Common patterns:

| Goal | Query |
|---|---|
| Any field contains term | `rust` |
| Specific field | `title:rust` |
| Exact phrase | `"full text search"` |
| Both terms | `rust AND performance` |
| Either term | `rust OR golang` |
| Exclude term | `-python` or `NOT python` |
| Prefix wildcard | `rust*` |
| All documents | `*` |
| Numeric range | `price:[10 TO 100]` |
| Date range | `date:[2024-01-01T00:00:00Z TO *]` |
| Nested JSON key | `metadata.color:blue` |
