<template>
  <div class="p-6 space-y-6">
    <div
      v-if="status === 'offline'"
      class="rounded-md border border-destructive/40 bg-destructive/10 px-4 py-3 text-sm text-destructive"
    >
      tantex server is unreachable. Start it with cargo run.
    </div>

    <template v-if="loading">
      <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
        <div v-for="n in 4" :key="n" class="bg-sidebar rounded-(--radius-xl) border p-5">
          <Skeleton class="h-3 w-28 mb-4" />
          <Skeleton class="h-9 w-24" />
        </div>
      </div>
    </template>
    <template v-else>
      <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
        <Card class="bg-sidebar border">
          <CardHeader class="pb-1">
            <CardTitle class="text-xs font-medium text-muted-foreground uppercase tracking-wide">
              Total Documents
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p class="text-3xl font-bold tabular-nums">{{ formatNum(totalDocs) }}</p>
          </CardContent>
        </Card>

        <Card class="bg-sidebar border">
          <CardHeader class="pb-1">
            <CardTitle class="text-xs font-medium text-muted-foreground uppercase tracking-wide">
              Indexes
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p class="text-3xl font-bold tabular-nums">{{ totalIndexes }}</p>
          </CardContent>
        </Card>

        <Card class="bg-sidebar border">
          <CardHeader class="pb-1">
            <CardTitle class="text-xs font-medium text-muted-foreground uppercase tracking-wide">
              Total Segments
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p class="text-3xl font-bold tabular-nums">{{ totalSegments }}</p>
          </CardContent>
        </Card>

        <Card :class="'bg-sidebar border ' + (ingestRate > 0 ? 'border-green-500/40 bg-green-500/5' : '')">
          <CardHeader class="pb-1">
            <CardTitle class="text-xs font-medium text-muted-foreground uppercase tracking-wide">
              Ingest Rate
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p
              class="text-3xl font-bold tabular-nums"
              :class="ingestRate > 0 ? 'text-green-500' : ''"
            >
              {{ formatNum(ingestRate) }}
            </p>
            <p class="text-xs text-muted-foreground mt-1">docs / sec</p>
          </CardContent>
        </Card>
      </div>
    </template>

    <template v-if="loading">
      <h2 class="text-sm font-semibold text-muted-foreground mb-3 uppercase tracking-wide">Per-index</h2>
      <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-3">
        <div v-for="n in 4" :key="n" class="bg-sidebar rounded-(--radius-xl) border p-4">
          <div class="flex items-center justify-between mb-3">
            <Skeleton class="h-4 w-28" />
            <Skeleton class="h-5 w-16 rounded-md" />
          </div>
          <div class="space-y-2">
            <Skeleton class="h-3.5 w-full" />
            <Skeleton class="h-3.5 w-20" />
          </div>
        </div>
      </div>
    </template>
    <template v-else>
      <div v-if="indexes.length">
        <h2 class="text-sm font-semibold text-muted-foreground mb-3 uppercase tracking-wide">Per-index</h2>
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-3">
          <NuxtLink
            v-for="idx in indexes"
            :key="idx.name"
            :to="`/indexes/${idx.name}`"
            class="block"
          >
            <Card class="bg-sidebar hover:border-primary/50 transition-colors cursor-pointer">
              <CardHeader class="pb-2">
                <div class="flex items-center justify-between">
                  <CardTitle class="text-sm font-semibold truncate">{{ idx.name }}</CardTitle>
                  <Badge
                    v-if="idx.rate > 0"
                    variant="secondary"
                    class="text-green-600 bg-green-100 dark:bg-green-900/30 dark:text-green-400 shrink-0"
                  >
                    +{{ formatNum(idx.rate) }}/s
                  </Badge>
                </div>
              </CardHeader>
              <CardContent class="space-y-1">
                <div class="flex justify-between text-xs text-muted-foreground">
                  <span>Documents</span>
                  <span class="font-mono font-medium text-foreground">{{ formatNum(idx.total_docs ?? idx.doc_count) }}</span>
                </div>
                <div class="flex justify-between text-xs text-muted-foreground">
                  <span>Segments</span>
                  <span class="font-mono font-medium text-foreground">{{ idx.num_segments }}</span>
                </div>
              </CardContent>
            </Card>
          </NuxtLink>
        </div>
      </div>

      <div v-else-if="status === 'online'" class="text-sm text-muted-foreground">
        No indexes yet. <NuxtLink to="/indexes" class="underline">Create one</NuxtLink>.
      </div>
    </template>
  </div>
</template>

<script setup>
useHead({ title: 'Dashboard' })

const { status, totalDocs, totalIndexes, totalSegments, ingestRate, indexes } = useMetricsStream()
const { setBreadcrumb } = useBreadcrumb()

const loading = computed(() => status.value === 'connecting')

setBreadcrumb([{ label: 'Dashboard' }])

function formatNum(n) {
  return n.toLocaleString()
}
</script>
