# Compression

Tantex applies compression to the **doc store** (stored fields only) — not to the inverted index. This reduces disk usage and memory footprint for large result sets, with minimal impact on search latency (decompression happens on retrieval, not during search scoring).

---

## Compression codecs

| Codec | Compression ratio | Speed | Best for |
|---|---|---|---|
| `"none"` | — | Fastest retrieval | Small datasets, latency-sensitive workloads |
| `"lz4"` | ~1.5–2× | Very fast (decompression) | Default; balanced for most use cases |
| `"zstd"` | ~2–3× | Fast | Better ratio than lz4, slight latency tradeoff |
| `"zstd:1"` to `"zstd:9"` | ~2–6× | Variable (depends on level) | High compression at levels 8–9; slower decompression |
| `"zstd:3"` | ~2.5–3× | Fast | Recommended sweet spot: good ratio, fast decompression |

**Note:** Compression is configured at index creation time and cannot be changed per-index afterward. Compression ratios vary by content type — text-heavy data compresses better than binary or already-compressed data.

---

## Block size

The `block_size` parameter controls how many bytes are grouped together before compression. Default: `16384` (16 KB).

```json
{
  "name": "articles",
  "schema": {
    "fields": [
      { "name": "title", "type": "text", "stored": true },
      { "name": "body",  "type": "text", "stored": true }
    ],
    "compression": "zstd:3",
    "block_size": 16384
  }
}
```

### Size vs. speed tradeoff

- **Larger block size** (e.g. `65536`): better compression ratio, slower document retrieval (decompresses more data than needed)
- **Smaller block size** (e.g. `4096`): faster document retrieval, worse compression ratio (overhead per block)

For most workloads, `16384` is optimal. Increase it for archival storage (large docs, queries returning many hits); decrease it for latency-sensitive APIs (small result sets, expensive decompression).

---

## Compression during ingest

Compression is **not applied to the indexing path** — only to completed segments at commit time. Ingest throughput is unaffected by the compression setting. Large compression levels (e.g. `zstd:9`) impose a one-time cost when the segment is finalized, not per-document.

---

## Choosing a compression setting

| Scenario | Recommendation |
|---|---|
| Balanced (default) | `"lz4"` |
| High cardinality, large result sets | `"zstd:3"` |
| Archival storage, rare queries | `"zstd:9"` |
| Very low latency (microsecond-scale) | `"none"` |
| Bandwidth-constrained (e.g. cloud storage egress) | `"zstd:5"` or `"zstd:6"` |

---

## Storage overhead

Example: A 10 GB index with 10 million documents.

- **No compression** (`"none"`): ~10 GB on disk
- **lz4**: ~5–7 GB on disk (50–70% reduction)
- **zstd:3**: ~4–5 GB on disk (50–60% reduction, similar to lz4)
- **zstd:9**: ~3–4 GB on disk (60–70% reduction, longer finalization time)

See [How to design a schema](../how-to/design-schema.md) for integration with field definitions.
