<template>
  <div class="p-6 space-y-6">
    <template v-if="loading">
      <div class="bg-sidebar rounded-(--radius-xl) border p-4">
        <div class="flex items-center gap-3">
          <Skeleton class="size-3 rounded-full" />
          <div class="space-y-2">
            <Skeleton class="h-5 w-40" />
            <Skeleton class="h-3 w-64" />
          </div>
        </div>
      </div>

      <div class="grid grid-cols-2 lg:grid-cols-5 gap-4">
        <div v-for="n in 5" :key="n" class="bg-sidebar rounded-(--radius-xl) border p-4 space-y-3">
          <Skeleton class="h-3 w-24" />
          <Skeleton class="h-7 w-20" />
          <Skeleton class="h-3 w-32" />
        </div>
      </div>

      <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
        <div v-for="n in 2" :key="n" class="bg-sidebar rounded-(--radius-xl) border p-4 space-y-3">
          <div class="flex items-center justify-between">
            <Skeleton class="h-3 w-20" />
            <Skeleton class="h-5 w-12" />
          </div>
          <Skeleton class="h-8 w-24" />
          <div class="space-y-1">
            <Skeleton v-for="i in 5" :key="i" class="h-2 w-full" />
          </div>
        </div>
      </div>

      <div class="space-y-3">
        <Skeleton class="h-4 w-20" />
        <div class="rounded-lg border overflow-hidden">
          <div class="p-4 space-y-3">
            <Skeleton v-for="n in 4" :key="n" class="h-6 w-full" />
          </div>
        </div>
      </div>
    </template>
    <template v-else>
      <div class="bg-sidebar rounded-(--radius-xl) border p-4">
        <div class="flex items-center gap-4">
          <div class="flex items-center gap-3">
            <div class="relative flex size-3">
              <span
                v-if="health.key === 'ACTIF'"
                class="absolute inline-flex h-full w-full rounded-full opacity-75 animate-ping"
                :class="health.pingClass"
              />
              <span class="relative inline-flex rounded-full size-3" :class="health.dotClass" />
            </div>
            <div>
              <div class="flex items-center gap-2">
                <span class="text-lg font-bold tracking-tight">{{ health.label }}</span>
                <Badge v-if="health.key !== 'HORS LIGNE'" :class="health.badgeClass">{{ health.key }}</Badge>
              </div>
              <p class="text-xs text-muted-foreground mt-0.5">{{ health.description }}</p>
            </div>
          </div>
          <div class="ml-auto text-xs text-muted-foreground tabular-nums">
            Mis à jour il y a <span class="font-medium">{{ lastUpdateAgo }}s</span>
          </div>
        </div>
      </div>

      <div class="grid grid-cols-2 lg:grid-cols-5 gap-4">
        <StatCard
          title="Total Documents"
          :value="fmt(totalDocs)"
          :sub="`${fmt(totalIndexes)} index · ${totalSegments} segments`"
        />
        <StatCard
          title="RAM utilisée"
          :value="fmt(ramUsedMb)"
          unit="MB"
          :sub="`sur ${fmt(ramTotalMb)} MB total`"
        />
        <StatCard
          title="Docs en attente"
          :value="fmt(totalPendingDocs)"
          :sub="totalPendingDocs > 0 ? 'buffered, non commités' : 'aucun pending'"
        />
        <StatCard
          title="Merges en cours"
          :value="String(totalMergesInProgress)"
          :sub="totalMergesInProgress > 0 ? 'fusion de segments active' : 'aucun merge actif'"
        />
        <StatCard
          title="Search Rate"
          :value="fmtDec(totalSearchRate)"
          unit="req/s"
          :sub="totalSearchRate > 0 ? 'requêtes actives' : 'aucune recherche'"
        />
      </div>

      <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
        <ClientOnly>
          <MetricCard
            title="Ingest Rate"
            :value="fmt(ingestRate)"
            unit="docs/s"
            :series="history.ingestRate.value"
            color="green"
          />
          <MetricCard
            title="Docs en attente"
            :value="fmt(totalPendingDocs)"
            badge="buffering"
            badge-variant="amber"
            :series="history.pendingDocs.value"
            color="orange"
          />
        </ClientOnly>
      </div>

      <div v-if="indexes.length" class="space-y-3">
        <h2 class="text-sm font-semibold text-muted-foreground uppercase tracking-wide">Par index</h2>
        <div class="rounded-lg border overflow-hidden">
          <table class="w-full text-sm">
            <thead class="bg-muted/50">
              <tr>
                <th class="text-left px-4 py-2.5 font-medium text-muted-foreground">Index</th>
                <th class="text-right px-4 py-2.5 font-medium text-muted-foreground">Docs commités</th>
                <th class="text-right px-4 py-2.5 font-medium text-muted-foreground">Pending</th>
                <th class="text-right px-4 py-2.5 font-medium text-muted-foreground">Total ingéré</th>
                <th class="text-right px-4 py-2.5 font-medium text-muted-foreground">Ingest/s</th>
                <th class="text-right px-4 py-2.5 font-medium text-muted-foreground">Search/s</th>
                <th class="text-right px-4 py-2.5 font-medium text-muted-foreground">Latence</th>
                <th class="text-right px-4 py-2.5 font-medium text-muted-foreground">Segments</th>
                <th class="text-right px-4 py-2.5 font-medium text-muted-foreground">Merges</th>
                <th class="text-right px-4 py-2.5 font-medium text-muted-foreground">En cours</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-border">
              <tr v-for="idx in indexes" :key="idx.name" class="hover:bg-muted/30 transition-colors">
                <td class="px-4 py-2.5">
                  <NuxtLink :to="`/indexes/${idx.name}`" class="font-medium hover:underline">{{ idx.name }}</NuxtLink>
                </td>
                <td class="px-4 py-2.5 text-right tabular-nums">{{ fmt(idx.doc_count) }}</td>
                <td class="px-4 py-2.5 text-right tabular-nums" :class="idx.pending_docs > 0 ? 'text-amber-500' : 'text-muted-foreground'">
                  {{ idx.pending_docs > 0 ? fmt(idx.pending_docs) : '—' }}
                </td>
                <td class="px-4 py-2.5 text-right tabular-nums text-muted-foreground">{{ fmt(idx.total_docs_ingested ?? 0) }}</td>
                <td class="px-4 py-2.5 text-right tabular-nums" :class="idx.rate > 0 ? 'text-green-500 font-medium' : 'text-muted-foreground'">
                  {{ idx.rate > 0 ? fmt(idx.rate) : '—' }}
                </td>
                <td class="px-4 py-2.5 text-right tabular-nums" :class="idx.search_rate > 0 ? 'text-violet-500 font-medium' : 'text-muted-foreground'">
                  {{ idx.search_rate > 0 ? fmtDec(idx.search_rate) : '—' }}
                </td>
                <td class="px-4 py-2.5 text-right tabular-nums" :class="idx.avg_search_latency_ms > 100 ? 'text-red-500' : 'text-muted-foreground'">
                  {{ idx.avg_search_latency_ms > 0 ? fmtDec(idx.avg_search_latency_ms) + ' ms' : '—' }}
                </td>
                <td class="px-4 py-2.5 text-right tabular-nums">{{ idx.num_segments }}</td>
                <td class="px-4 py-2.5 text-right tabular-nums text-muted-foreground">{{ idx.merge_count ?? 0 }}</td>
                <td class="px-4 py-2.5 text-right tabular-nums">
                  <span v-if="idx.merge_in_progress" class="inline-flex items-center gap-1 text-amber-500 font-medium">
                    <span class="size-1.5 rounded-full bg-amber-500 animate-pulse inline-block" />
                    actif
                  </span>
                  <span v-else class="text-muted-foreground">—</span>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </template>
  </div>
</template>

<script setup>
useHead({ title: 'Metrics' })

const { status, totalDocs, totalIndexes, totalSegments, ingestRate, totalPendingDocs, totalMergesInProgress, ramUsedMb, ramTotalMb, indexes } = useMetricsStream()
const history = useMetricsHistory()
const { setBreadcrumb } = useBreadcrumb()

const loading = computed(() => status.value === 'connecting')

setBreadcrumb([{ label: 'Metrics' }])

const lastUpdateAgo = ref(0)
let lastUpdateTime = Date.now()

const { lastEvent } = useStreamEvents()
watch(lastEvent, (e) => {
  if (e && e.type === 'metrics' && e.status === 'online') {
    lastUpdateTime = Date.now()
    lastUpdateAgo.value = 0
  }
})

let ticker = null
onMounted(() => {
  ticker = setInterval(() => {
    lastUpdateAgo.value = Math.floor((Date.now() - lastUpdateTime) / 1000)
  }, 1000)
})
onUnmounted(() => { if (ticker) clearInterval(ticker) })

const totalSearchRate = computed(() =>
  indexes.value.reduce((s, i) => s + (i.search_rate ?? 0), 0)
)

const avgSearchLatencyMs = computed(() => {
  const relevant = indexes.value.filter((i) => i.avg_search_latency_ms > 0)
  if (!relevant.length) return 0
  return relevant.reduce((s, i) => s + i.avg_search_latency_ms, 0) / relevant.length
})

const health = computed(() => {
  if (status.value !== 'online') {
    return { key: 'HORS LIGNE', label: 'Serveur hors ligne', description: 'Le serveur tantex est inaccessible.', dotClass: 'bg-red-500', pingClass: '', badgeClass: 'bg-zinc-100 text-zinc-700 dark:bg-zinc-800 dark:text-zinc-300' }
  }
  const lat = avgSearchLatencyMs.value
  const segs = totalSegments.value
  if (lat > 500 || totalPendingDocs.value > 60_000_000) {
    return { key: 'SATUR\u00C9', label: 'Noeud satur\u00E9', description: `Latence \u00E9lev\u00E9e ou trop de docs en attente (${fmt(totalPendingDocs.value)}).`, dotClass: 'bg-red-500', pingClass: 'bg-red-400', badgeClass: 'bg-red-100 text-red-700 dark:bg-red-900/40 dark:text-red-300' }
  }
  if (lat > 100 || segs > 80) {
    return { key: 'D\u00C9GRAD\u00C9', label: 'Performance d\u00E9grad\u00E9e', description: `Latence ou fragmentation \u00E9lev\u00E9e (${segs} segments).`, dotClass: 'bg-amber-500', pingClass: 'bg-amber-400', badgeClass: 'bg-amber-100 text-amber-700 dark:bg-amber-900/40 dark:text-amber-300' }
  }
  if (ingestRate.value > 0 || totalSearchRate.value > 0) {
    return { key: 'ACTIF', label: 'Noeud actif', description: 'Ingestion et/ou recherche en cours.', dotClass: 'bg-green-500', pingClass: 'bg-green-400', badgeClass: 'bg-green-100 text-green-700 dark:bg-green-900/40 dark:text-green-300' }
  }
  return { key: 'PARFAIT', label: 'Noeud en bonne sant\u00E9', description: 'Toutes les m\u00E9triques sont dans les seuils normaux.', dotClass: 'bg-green-500', pingClass: '', badgeClass: 'bg-green-100 text-green-700 dark:bg-green-900/40 dark:text-green-300' }
})

function fmt(n) { return n.toLocaleString() }
function fmtDec(n) { return n.toLocaleString(undefined, { maximumFractionDigits: 2 }) }
</script>
