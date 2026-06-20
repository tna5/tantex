# How to Use the Dashboard

The tantex dashboard is embedded in the server binary and served automatically at `http://localhost:7200`. It provides a visual interface for managing indexes, running searches, and monitoring real-time metrics. No separate process is required.

---

## Accessing the dashboard

Start the server:

```sh
./tantex
```

Open [http://localhost:7200](http://localhost:7200) in your browser.

---

## Dashboard page — live metrics

The home page displays live statistics updated every second via Server-Sent Events:

- **Total documents** — sum of committed docs across all indexes
- **Indexes** — total number of indexes
- **Segments** — total number of tantivy segments
- **Ingest rate** — documents per second currently being indexed

Per-index cards below the summary show individual doc counts and ingest rates as documents arrive. If the tantex server is unreachable, the page shows "Server offline".

---

## Indexes page — manage indexes

Navigate to **Indexes** in the sidebar.

### View existing indexes

A table lists all indexes with their name, document count, and segment count. Click any row to open the index detail page.

### Create a new index

Click **New Index** in the top-right corner. The dialog lets you:

1. Set an index name
2. Add fields — specify name, type, storage options, and tokenizer for each field
3. Choose a compression algorithm and block size for the doc store

Click **Create** when ready. See [How to design a schema](design-schema.md) for guidance on choosing field types.

### Delete an index

Click the trash icon on any row and confirm the deletion. This permanently removes the index and all its data.

---

## Index detail page

Click an index name to open its detail page, which shows:

- **Schema** — all fields with their types and storage options
- **Commit** button — force an immediate commit so all pending documents become searchable
- **Search panel** — run ad-hoc queries against the index

### Running a search

Enter a query in the search box and press Enter (or click Search). The results panel shows:
- Number of total hits
- Each result's relevance score and stored fields as JSON

See [Query syntax reference](../reference/query-syntax.md) for query language documentation.

---

## Metrics page

Navigate to **Metrics** to see historical ingest rate graphs per index. The chart updates in real time.

---

## Settings page

Navigate to **Settings** to view and modify runtime configuration.

- **Read-only fields** — `socket_path` and `data_dir` are shown but cannot be changed at runtime.
- **Tunable fields** — all other parameters (writer heap, commit thresholds, thread counts, merge policy) can be updated and take effect immediately.

**Presets** in the header bar apply common configurations:
- **Max ingest** — optimises for bulk loading speed
- **Balanced** — a middle ground between throughput and freshness
- **Low latency** — commits frequently for near-real-time search visibility

Click **Save changes** (or press **⌘S** / **Ctrl+S**) to apply. The footer turns amber when there are unsaved changes and green briefly after a successful save.

---

## Authentication

If authentication is enabled (a key is set at server startup), the dashboard displays a login page before you can access any features.

**Login:**
1. Enter your API key in the text field
2. Click **Login**

The server sets a secure, HttpOnly cookie. Subsequent page loads are automatically authenticated.

**Logout:**
Click the **Logout** button in the top menu to clear the session cookie.

See [Authentication](../reference/configuration.md#api_key) to configure the API key at server startup.
