# Search and scoring

This page explains how tantex executes a search query and how relevance scores are computed.

---

## The searcher

Searches run directly on Tokio async threads — there is no dedicated search thread. `tantivy::Searcher` is `Send + Sync`, so multiple searches run fully concurrently.

The `IndexReader` is opened with `ReloadPolicy::OnCommitWithDelay`: each time a commit completes, the reader automatically picks up the new segment within milliseconds. A search always operates on a consistent **point-in-time snapshot** of the index — documents committed after the search started are not visible to that search.

```
Search request
  │
  ▼
Acquire read lock on IndexManager (shared — many searches run concurrently)
  │
  ▼
IndexReader.search()    ← runs on the async thread
  │   snapshot of committed segments, no locking on the index data
  │
  ▼
Return hits + score + elapsed_us
```

---

## Scoring — BM25

Tantex uses [BM25](https://en.wikipedia.org/wiki/Okapi_BM25) (Best Match 25), the same ranking function used by Elasticsearch and most production search engines.

The score reflects **how relevant a document is to the query**:

- **Term frequency** — how many times the query terms appear in the document
- **Inverse document frequency** — rare terms across the corpus score higher than common terms
- **Field length normalisation** — a match in a short field scores higher than the same match in a long field

The `score` value returned in hits is a raw BM25 float. It is comparable within a single query result set, but not across different queries or different indexes.

### What affects scoring

- **Tokenizer** — the `"default"` tokenizer lowercases and splits on whitespace and punctuation. The `"en_stem"` tokenizer additionally reduces words to their root (`running` → `run`). Tokenizer affects which terms are indexed and therefore which documents match and at what score.
- **Field boosts** — use `field^N` syntax in queries (e.g. `title:rust^3 body:rust`) to weight matches in a specific field higher.
- **Multiple terms** — all query terms contribute independently; a document matching more terms scores higher.

---

## Query parsing

By default, tantex searches **all indexed text fields** when no field is specified. The query parser is tantivy's standard parser.

Common query forms:

| Pattern | Example | Meaning |
|---|---|---|
| Terms | `rust performance` | Documents containing both terms (OR by default) |
| Field-scoped | `title:rust` | Match only in the `title` field |
| Phrase | `"full text search"` | Exact phrase match |
| Boolean AND | `rust AND NOT javascript` | Must contain `rust`, must not contain `javascript` |
| Boolean OR | `title:tantivy OR title:search` | |
| Range (numeric) | `views:[100 TO 5000]` | |
| Range (date) | `date:[2026-01-01 TO 2026-12-31]` | |
| Prefix/wildcard | `prog*` | Prefix match (end of term only) |
| Boost | `title:rust^3 body:rust` | Matches in `title` weighted 3×  |

See [Query syntax reference](../reference/query-syntax.md) for the complete specification.

---

## Search parameters

| Parameter | Default | Description |
|---|---|---|
| `query` | required | Query string |
| `limit` | `10` | Max hits to return |
| `offset` | `0` | Hits to skip (for pagination) |

The `elapsed_us` field in the response is server-side query latency in microseconds, excluding HTTP overhead. It covers query parsing, segment scanning, BM25 scoring, and result collection.
