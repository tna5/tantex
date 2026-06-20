# Commit and Merge Policy

Understanding when documents become visible and how segments are managed is essential for tuning tantex's behaviour.

---

## What is a commit?

Tantivy's write path is a staged pipeline:

```
ingest call → IndexWriter RAM buffer → segment flush → commit → visible to readers
```

Documents ingested into a `u64` writer heap are not visible to searches until a **commit** is performed. A commit:
1. Flushes any in-memory segment to disk
2. Atomically updates the index `meta.json` to include the new segments
3. Signals readers (via `ReloadPolicy::OnCommitWithDelay`) that a new generation is available

This means **ingest does not block search**, and searches always see a consistent snapshot of the index (the last committed state).

---

## When does tantex commit?

Tantex's writer thread (`writer_loop`) runs an auto-commit state machine with three independent triggers:

### 1. Soft commit — idle threshold

```
pending_docs ≥ auto_commit_doc_count  AND  writer channel is idle
```

When the number of pending (uncommitted) documents reaches `auto_commit_doc_count` **and** no new ingest message arrives for a brief polling interval, a commit fires. This trigger is "soft" because it requires inactivity — under continuous ingest, the idle condition never fires.

**Default:** 10 000 000 documents.

### 2. Hard commit — absolute count

```
pending_docs ≥ auto_commit_doc_count × hard_commit_multiplier
```

Under sustained ingest, the soft commit might never trigger (the channel is never idle). The hard commit fires unconditionally when pending docs reach the absolute threshold. This bounds the RAM consumed by the writer heap and prevents the index from going too long without committing.

**Default:** 10 000 000 × 4 = 40 000 000 documents.

### 3. Timer commit — idle timer

```
elapsed since last commit ≥ auto_commit_interval_secs
```

A background timer fires if no commit has happened within the interval. This acts as a safety net — even if ingest stops mid-batch and both count-based triggers are below threshold, documents will eventually become visible.

**Default:** 30 seconds.

### Explicit commit

At any time, a client can force an immediate commit via `MSG_COMMIT` or `POST /api/indexes/{name}/commit`. This is the simplest way to make a batch visible immediately after loading.

---

## Commit latency

A commit in tantivy involves:
1. Flushing the in-memory term dictionary and postings to disk (one segment per flush)
2. Merging any segments scheduled by the merge policy
3. Writing the new `meta.json`

For large heaps (4 GB default), a single commit flush can write several hundred million postings. On NVMe storage this takes 1–10 seconds; on slower spinning disks it may take longer. During this time the writer thread is busy and cannot accept new ingest. Plan accordingly when tuning commit thresholds.

---

## What is a segment?

Tantivy's on-disk format is based on **segments** — self-contained, immutable index shards. Each commit flush produces at least one new segment. Over time, many small segments accumulate.

Searching with many segments is slower than searching with few large segments because each segment must be queried independently and results merged. The merge policy controls when segments are combined.

---

## The merge policy

Tantex uses a custom `TargetDocCountMergePolicy`. After each commit, it schedules merges using the following logic:

1. Find all segments smaller than `merge_target_docs`.
2. Group them and schedule a merge when at least `min_num_segments` segments qualify.
3. Never merge more than `max_merge_factor` segments in a single operation.

| Parameter | Default | Effect |
|---|---|---|
| `merge_target_docs` | 20 000 000 | Segments at or above this size are considered "large" and are not merged further. Increase to allow larger segments (fewer merges, more RAM per merge). |
| `min_num_segments` | 2 | A merge is only scheduled when at least this many small segments exist. |
| `max_merge_factor` | 10 | Maximum segments per merge. Limiting this prevents rare, very long merge operations. |

Merges run on tantivy's internal merge threads — they do not block ingest or search. Search sees the pre-merge and post-merge states as separate consistent snapshots.

---

## Practical implications

### "My documents are not appearing in search results"

They are likely uncommitted. Either:
- Wait for the auto-commit to fire (up to `auto_commit_interval_secs` seconds after ingest stops)
- Call `POST /api/indexes/{name}/commit` explicitly after ingesting

### "Ingest slowed down after a while"

The writer heap may be filling up and flushing frequently. Consider:
- Increasing `writer_heap_size` to defer flushes
- Raising `auto_commit_doc_count` to commit less often
- Checking available RAM — the heap is in-process memory

### "Search is slow"

May be caused by too many small segments. Check the segment count with `GET /api/indexes/{name}/segments`. If there are many segments, either:
- Wait for background merges to complete
- Lower `merge_target_docs` to trigger more aggressive merging
- Reduce `min_num_segments` to 2 (minimum) to merge smaller groups sooner

### "The server is using a lot of RAM"

The `writer_heap_size` controls per-index RAM use for the write path. With multiple indexes, the total is `writer_heap_size × number_of_indexes`. Reduce `writer_heap_size` if memory is constrained.
