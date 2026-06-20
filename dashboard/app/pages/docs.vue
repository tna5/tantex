<template>
  <div class="p-6 space-y-8">
    <div class="space-y-1">
      <h1 class="text-lg font-semibold">API Reference</h1>
      <p class="text-sm text-muted-foreground">
        All endpoints are served by the tantex Rust server on its HTTP port.
        Authenticate with <code class="bg-muted rounded px-1.5 py-0.5 text-xs">X-Api-Key: &lt;key&gt;</code>
        or <code class="bg-muted rounded px-1.5 py-0.5 text-xs">Authorization: Bearer &lt;key&gt;</code>
        (only required once an API key has been created).
      </p>
    </div>

    <section v-for="group in groups" :key="group.title" class="space-y-3">
      <h2 class="text-sm font-semibold uppercase tracking-wide text-muted-foreground flex items-center gap-2">
        <Icon :icon="group.icon" class="size-4" />
        {{ group.title }}
      </h2>

      <div
        v-for="ep in group.endpoints"
        :key="ep.method + ep.path"
        class="bg-sidebar rounded-(--radius-xl) border overflow-hidden"
      >
        <button
          class="w-full flex items-start gap-3 px-4 py-3 text-left hover:bg-muted/30 transition-colors"
          @click="toggle(ep.method + ep.path)"
        >
          <Badge :class="methodClass(ep.method)" class="font-mono text-xs mt-0.5 shrink-0 w-16 justify-center">
            {{ ep.method }}
          </Badge>
          <span class="font-mono text-sm flex-1">{{ ep.path }}</span>
          <span class="text-xs text-muted-foreground mt-0.5 flex-1 hidden sm:block">{{ ep.summary }}</span>
          <Icon
            :icon="expanded.has(ep.method + ep.path) ? 'majesticons:chevron-up' : 'majesticons:chevron-down'"
            class="size-4 text-muted-foreground shrink-0 mt-0.5"
          />
        </button>

        <div v-if="expanded.has(ep.method + ep.path)" class="border-t px-4 py-4 space-y-4 text-sm">
          <p class="text-muted-foreground">{{ ep.description || ep.summary }}</p>

          <div v-if="ep.request" class="space-y-1">
            <div class="text-xs font-semibold uppercase tracking-wide text-muted-foreground">Request body</div>
            <pre class="bg-muted rounded p-3 text-xs overflow-x-auto">{{ ep.request }}</pre>
          </div>

          <div v-if="ep.response" class="space-y-1">
            <div class="text-xs font-semibold uppercase tracking-wide text-muted-foreground">Response</div>
            <pre class="bg-muted rounded p-3 text-xs overflow-x-auto">{{ ep.response }}</pre>
          </div>

          <div v-if="ep.notes" class="text-xs text-muted-foreground italic">{{ ep.notes }}</div>
        </div>
      </div>
    </section>
  </div>
</template>

<script setup>
import { Icon } from '@iconify/vue'

useHead({ title: 'API Documentation' })
const { setBreadcrumb } = useBreadcrumb()
setBreadcrumb([{ label: 'Documentation' }])

const expanded = ref(new Set())
function toggle(key) {
  if (expanded.value.has(key)) expanded.value.delete(key)
  else expanded.value.add(key)
  expanded.value = new Set(expanded.value)
}

function methodClass(method) {
  return {
    GET: 'bg-blue-100 text-blue-700 dark:bg-blue-900/40 dark:text-blue-300',
    POST: 'bg-green-100 text-green-700 dark:bg-green-900/40 dark:text-green-300',
    DELETE: 'bg-red-100 text-red-700 dark:bg-red-900/40 dark:text-red-300',
  }[method] || ''
}

const groups = [
  {
    title: 'Authentication',
    icon: 'majesticons:lock',
    endpoints: [
      {
        method: 'GET',
        path: '/api/auth/status',
        summary: 'Check whether authentication is required',
        description: 'Returns whether at least one API key exists. This endpoint is always public.',
        response: `{ "auth_required": boolean }`,
        notes: 'Always accessible, even without a valid API key.',
      },
    ],
  },
  {
    title: 'API Keys',
    icon: 'majesticons:key',
    endpoints: [
      {
        method: 'GET',
        path: '/api/api-keys',
        summary: 'List all API keys',
        response: `{
  "keys": [
    {
      "id": "uuid",
      "name": "string",
      "created_at": "ISO 8601",
      "last_used_at": "ISO 8601 | null"
    }
  ]
}`,
        notes: 'Key hashes are never returned.',
      },
      {
        method: 'POST',
        path: '/api/api-keys',
        summary: 'Create a new API key',
        request: `{ "name": "string" }`,
        response: `{
  "id": "uuid",
  "name": "string",
  "key": "tantex_<random>"   // shown once only
}`,
        notes: 'The raw key is returned exactly once. Store it immediately.',
      },
      {
        method: 'DELETE',
        path: '/api/api-keys/:id',
        summary: 'Revoke an API key',
        response: `{ "success": true }`,
      },
    ],
  },
  {
    title: 'Indexes',
    icon: 'majesticons:database',
    endpoints: [
      {
        method: 'GET',
        path: '/api/indexes',
        summary: 'List all indexes',
        response: `{
  "indexes": [
    {
      "name": "string",
      "doc_count": number,
      "num_segments": number,
      "pending_docs": number
    }
  ]
}`,
      },
      {
        method: 'POST',
        path: '/api/indexes',
        summary: 'Create a new index',
        request: `{
  "name": "string",
  "schema": [
    {
      "name": "string",
      "type": "text | u64 | i64 | f64 | date | bool | bytes | json",
      "stored": boolean,
      "indexed": boolean,
      "fast": boolean,
      "tokenizer": "default | raw | en_stem | ..."
    }
  ]
}`,
        response: `{ "success": true, "field_ids": [number] }`,
      },
      {
        method: 'GET',
        path: '/api/indexes/:name',
        summary: 'Get index details',
        response: `{
  "name": "string",
  "schema": [...],
  "doc_count": number,
  "num_segments": number,
  "pending_docs": number
}`,
      },
      {
        method: 'DELETE',
        path: '/api/indexes/:name',
        summary: 'Delete an index and all its data',
        response: `{ "success": true }`,
      },
      {
        method: 'POST',
        path: '/api/indexes/:name/commit',
        summary: 'Force a commit (flush pending docs)',
        response: `{ "success": true }`,
      },
      {
        method: 'GET',
        path: '/api/indexes/:name/segments',
        summary: 'List segments for an index',
        response: `{
  "segments": [
    {
      "id": "string",
      "doc_count": number,
      "deleted_docs": number,
      "size_bytes": number
    }
  ]
}`,
      },
      {
        method: 'POST',
        path: '/api/indexes/:name/search',
        summary: 'Search an index',
        request: `{
  "query": "string",   // tantivy query syntax
  "limit": number,     // default 10
  "offset": number     // default 0
}`,
        response: `{
  "total_hits": number,
  "hits": [
    { "score": number, "doc": { ...stored fields } }
  ]
}`,
        notes: 'Query uses tantivy syntax: field:value, AND/OR, phrases, ranges.',
      },
    ],
  },
  {
    title: 'Metrics',
    icon: 'majesticons:pulse',
    endpoints: [
      {
        method: 'GET',
        path: '/api/metrics',
        summary: 'Snapshot of current metrics',
        response: `{
  "totalDocs": number,
  "totalIndexes": number,
  "totalSegments": number,
  "totalPendingDocs": number,
  "indexes": [...]
}`,
      },
      {
        method: 'GET',
        path: '/api/metrics/stream',
        summary: 'Server-Sent Events stream of live metrics',
        response: `// text/event-stream, one event per second
data: {
  "type": "metrics",
  "status": "online",
  "totalDocs": number,
  "totalIndexes": number,
  "totalSegments": number,
  "totalPendingDocs": number,
  "indexes": [...]
}`,
        notes: 'Connect via EventSource. Emits every ~1 second.',
      },
    ],
  },
  {
    title: 'Configuration',
    icon: 'majesticons:settings-cog',
    endpoints: [
      {
        method: 'GET',
        path: '/api/config',
        summary: 'Get current server configuration',
        response: `{
  "socket_path": "string",
  "data_dir": "string",
  "http_port": number,
  "shm_buffer_size": number,
  "writer_heap_size": number,
  "auto_commit_doc_count": number,
  "auto_commit_interval_secs": number,
  "merge_target_docs": number,
  "max_merge_factor": number,
  "min_num_segments": number,
  "num_indexing_threads": number,
  "index_threads_pct": number,
  "hard_commit_multiplier": number
}`,
      },
      {
        method: 'POST',
        path: '/api/config',
        summary: 'Update tunable configuration values at runtime',
        request: `{
  // all fields optional
  "shm_buffer_size": number,
  "writer_heap_size": number,
  "auto_commit_doc_count": number,
  "auto_commit_interval_secs": number,
  "merge_target_docs": number,
  "max_merge_factor": number,
  "min_num_segments": number,
  "num_indexing_threads": number,
  "index_threads_pct": number,
  "hard_commit_multiplier": number
}`,
        response: `{ "success": true }`,
        notes: 'Socket path and data dir cannot be changed at runtime.',
      },
    ],
  },
]
</script>
