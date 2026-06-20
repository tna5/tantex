# Indexing pipeline

This page explains what happens inside tantex when you ingest a document — from the API call to the document becoming visible in search results.

---

## The writer thread

Each index has exactly **one dedicated OS thread** (not a Tokio task) that owns the `tantivy::IndexWriter`. This is a tantivy requirement: `IndexWriter` is `!Send` and `!Sync`.

The writer thread runs a loop, receiving commands through a [crossbeam](https://docs.rs/crossbeam-channel) unbounded channel:

```
Async handler                 Writer thread (OS thread)
     │                               │
     │── WriterCommand::AddDocuments ──>│
     │                               │ parse → add to buffer
     │── WriterCommand::Commit ──────>│
     │                               │ flush + commit segment
     │<── oneshot response ──────────│
```

The async handler acquires a short-lived read lock on `IndexManager` to clone the channel sender, then **drops the lock before blocking on the response**. This means searches are never blocked by ongoing writes.

---

## Document lifecycle

When you call `POST /api/indexes/{name}/ingest` or send `MSG_INGEST_BATCH` over the socket, the following happens:

1. **Deserialisation** — JSON documents are parsed in a [Rayon](https://docs.rs/rayon) thread pool using `sonic-rs` (SIMD JSON). Each document is streamed directly into a `TantivyDocument` without materialising an intermediate `Map<String, Value>`.

2. **Writer buffer** — Documents are added to the `IndexWriter`'s in-memory buffer. Nothing is flushed to disk yet.

3. **Commit** — When a commit fires (see below), the buffer is flushed to disk as a new **segment**. Until that moment, the documents are invisible to searches.

### SHM path

The shared-memory ingest path (`MSG_INGEST_SHM`) bypasses serialisation entirely: the client writes NDJSON directly into a memory-mapped file, and the server reads from the same mapping. See [SHM ingestion](shm-ingestion.md) for details.

---

## Segments

A **segment** is a self-contained, immutable set of indexed documents written to disk. It consists of several files (posting lists, stored fields, column-store, etc.) under `{data_dir}/{index_name}/`.

After a commit, the new segment is immediately visible to the searcher. Over time, the merge policy compacts small segments into larger ones to keep query performance stable. See [Commit & merge policy](commit-merge-policy.md) for the full rules.

---

## When documents become visible

Documents are **not searchable until a commit occurs**. Commits are triggered automatically by three conditions (whichever fires first):

| Trigger | Config field | Default |
|---|---|---|
| Idle — writer channel has been empty for N seconds | `auto_commit_interval_secs` | 30 s |
| Soft count — N documents are buffered and the channel is idle | `auto_commit_doc_count` | 10 000 000 |
| Hard count — N × multiplier documents are buffered regardless of idle state | `auto_commit_doc_count × hard_commit_multiplier` | 40 000 000 |

To force an immediate commit:

```sh
curl -X POST http://localhost:7200/api/indexes/articles/commit
```

See [Commit & merge policy](commit-merge-policy.md) for the detailed mechanics and tuning advice.
