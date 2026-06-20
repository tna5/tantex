<template>
  <div class="p-6 space-y-6">
    <div class="grid grid-cols-1 lg:grid-cols-[1fr_280px] gap-6">
      <div class="bg-sidebar rounded-(--radius-xl) border overflow-hidden">
        <div class="p-4 space-y-3">
          <div class="flex items-center gap-2">
            <Select v-model="selectedIndex" :disabled="!indexes.length || !hasSearchableIndexes">
              <SelectTrigger class="h-9 w-44 shrink-0 text-sm">
                <SelectValue placeholder="Index\u2026" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem v-for="idx in indexes" :key="idx.name" :value="idx.name">
                  <span>{{ idx.name }}</span>
                  <span class="ml-2 text-xs text-muted-foreground tabular-nums">{{ fmtNum(idx.doc_count) }}</span>
                </SelectItem>
              </SelectContent>
            </Select>

            <div class="relative flex-1">
              <Icon icon="majesticons:search" class="absolute left-2.5 top-1/2 -translate-y-1/2 size-4 text-muted-foreground pointer-events-none" />
              <Input v-model="query" :disabled="!hasSearchableIndexes" placeholder="tantivy query &mdash; e.g. title:fast AND body:rust" class="h-9 pl-8 pr-3 font-mono text-sm w-full" @keydown.enter="runSearch" />
            </div>

            <Input v-model.number="limit" type="number" min="1" max="500" :disabled="!hasSearchableIndexes" class="h-9 w-20 text-sm tabular-nums shrink-0" title="Max results" />

            <Button variant="secondary" class="h-9 shrink-0" :disabled="!hasSearchableIndexes || !selectedIndex || !query.trim() || running" @click="runSearch">
              <Icon v-if="running" icon="majesticons:loader" class="size-4 mr-1.5 animate-spin" />
              <span v-else>Run</span>
            </Button>
          </div>

          <div v-if="searchErr" class="flex items-center gap-2 text-sm text-destructive">
            <Icon icon="majesticons:alert-circle" class="size-4 shrink-0" />
            {{ searchErr }}
          </div>

          <div v-if="lastResult" class="flex items-center gap-4 text-sm">
            <div class="flex items-center gap-1.5">
              <span class="text-muted-foreground">tantivy</span>
              <span class="font-mono font-semibold tabular-nums" :class="latencyClass(lastResult.elapsedUs)">{{ lastResult.elapsedUs }} &micro;s</span>
            </div>

            <span class="text-border">|</span>

            <div class="flex items-center gap-1.5">
              <span class="text-muted-foreground">hits</span>
              <span class="font-mono font-semibold tabular-nums">{{ fmtNum(lastResult.total) }}</span>
            </div>

            <span class="text-border">|</span>

            <div class="flex items-center gap-1.5">
              <span class="text-muted-foreground">showing</span>
              <span class="font-mono tabular-nums">{{ lastResult.hits.length }}</span>
            </div>

            <span class="text-border">|</span>

            <span class="text-muted-foreground font-mono truncate max-w-[280px]">{{ lastResult.index }}</span>
          </div>
        </div>

        <div v-if="!lastResult && !searchErr" class="flex items-center justify-center py-16">
          <EmptyState icon="majesticons:search" title="Type a query above and press Enter" description="Select an index, enter a tantivy query, and hit Run." />
        </div>

        <div v-else-if="lastResult && !lastResult.hits.length" class="flex items-center justify-center py-16">
          <EmptyState icon="majesticons:file-search" title="No documents matched" description="Try a different query or index." />
        </div>

        <div v-else-if="lastResult?.hits.length" class="divide-y divide-border/40">
          <div v-for="(hit, i) in lastResult.hits" :key="i" class="p-4 space-y-2">
            <div class="flex items-center justify-between gap-2">
              <span class="text-xs text-muted-foreground tabular-nums">#{{ i + 1 }}</span>
              <span class="text-xs font-mono text-amber-600 dark:text-amber-400 tabular-nums">score&nbsp;{{ hit.score.toFixed(6) }}</span>
            </div>
            <div class="divide-y divide-border/40 rounded-md border border-border/40">
              <div v-for="(val, key) in flatDoc(hit.doc)" :key="key" class="grid grid-cols-[140px_1fr] text-xs">
                <span class="px-3 py-1.5 font-mono text-muted-foreground truncate bg-muted/30">{{ key }}</span>
                <span class="px-3 py-1.5 font-mono break-all">{{ val }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div class="space-y-4">
        <div class="bg-sidebar rounded-(--radius-xl) border p-5 space-y-2 self-start">
          <div class="flex items-center gap-2">
            <Icon icon="majesticons:information-circle" class="size-4 text-muted-foreground" />
            <h2 class="text-sm font-semibold text-muted-foreground uppercase tracking-wide">Search Syntax</h2>
          </div>
          <div class="space-y-2 text-xs text-muted-foreground">
            <p>Basic <code class="bg-muted rounded px-1 py-0.5 font-mono">field:term</code></p>
            <p>Phrase <code class="bg-muted rounded px-1 py-0.5 font-mono">field:&quot;exact phrase&quot;</code></p>
            <p>AND / OR / NOT operators</p>
            <p>Range <code class="bg-muted rounded px-1 py-0.5 font-mono">field:[start TO end]</code></p>
            <NuxtLink to="/docs" class="inline-flex items-center gap-1 text-xs font-medium text-primary hover:underline mt-1">
              View full documentation
              <Icon icon="majesticons:arrow-right" class="size-3" />
            </NuxtLink>
          </div>
        </div>

        <div v-if="history.length" class="bg-sidebar rounded-(--radius-xl) border p-5 space-y-3 self-start">
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              <Icon icon="majesticons:clock" class="size-4 text-muted-foreground" />
              <h2 class="text-sm font-semibold text-muted-foreground uppercase tracking-wide">History</h2>
            </div>
            <Button variant="ghost" size="sm" class="text-xs text-muted-foreground h-auto px-1" @click="history = []">clear</Button>
          </div>
          <div class="space-y-2">
            <div v-for="(h, i) in history" :key="i" class="border border-border/40 rounded-md p-2.5 cursor-pointer hover:bg-muted/20 transition-colors space-y-1" @click="replay(h)">
              <div class="flex items-center justify-between gap-2">
                <span class="text-[11px] font-mono text-muted-foreground truncate">{{ h.index }}</span>
                <span class="text-[11px] font-mono tabular-nums shrink-0" :class="latencyClass(h.elapsedUs)">{{ h.elapsedUs }} &micro;s</span>
              </div>
              <div class="text-xs font-mono truncate" :title="h.query">{{ h.query }}</div>
              <div class="text-[11px] text-muted-foreground tabular-nums">{{ fmtNum(h.total) }} hits</div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { Icon } from '@iconify/vue';

const { listIndexes, searchIndex } = useTantex();
const { setBreadcrumb } = useBreadcrumb();
setBreadcrumb([{ label: 'Search' }]);

const indexes = ref([])
const indexErr = ref('')
const selectedIndex = ref('')
const hasSearchableIndexes = computed(() => indexes.value.some(idx => (idx.doc_count ?? 0) > 0))
const query = ref('')
const limit = ref(10)
const running = ref(false)
const searchErr = ref('')
const lastResult = ref(null)
const history = ref([])

onMounted(async () => {
    try {
        const res = await listIndexes()
        indexes.value = res.indexes ?? []
        if (indexes.value.length) selectedIndex.value = indexes.value[0].name
    } catch (e) {
        indexErr.value = e?.message ?? 'Failed to load indexes'
    }
})

async function runSearch() {
    if (!selectedIndex.value || !query.value.trim() || running.value) return
    running.value = true
    searchErr.value = ''
    try {
        const r = await searchIndex(selectedIndex.value, query.value.trim(), Number(limit.value) || 10)
        const elapsedUs = Number(r.elapsed_us ?? 0)
        const result = {
            index: selectedIndex.value,
            query: query.value.trim(),
            elapsedUs,
            total: r.total_hits ?? 0,
            hits: Array.isArray(r.hits) ? r.hits : [],
        }
        lastResult.value = result
        history.value = [
            { index: result.index, query: result.query, elapsedUs, total: result.total },
            ...history.value.filter(h => !(h.index === result.index && h.query === result.query)),
        ].slice(0, 20)
    } catch (e) {
        searchErr.value = e?.message ?? 'Search failed'
    } finally {
        running.value = false
    }
}

function replay(h) {
    selectedIndex.value = h.index
    query.value = h.query
    runSearch()
}

function fmtNum(n) {
    return Number(n ?? 0).toLocaleString()
}

function latencyClass(us) {
    if (!us) return 'text-muted-foreground'
    if (us < 1_000) return 'text-green-600 dark:text-green-400'
    if (us < 10_000) return 'text-blue-600 dark:text-blue-400'
    if (us < 100_000) return 'text-amber-600 dark:text-amber-400'
    return 'text-red-600 dark:text-red-400'
}

function flatDoc(doc) {
    const out = {}
    for (const [k, v] of Object.entries(doc ?? {})) {
        out[k] = typeof v === 'object' ? JSON.stringify(v) : String(v)
    }
    return out
}
</script>
