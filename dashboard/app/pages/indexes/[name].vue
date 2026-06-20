<template>
  <div class="p-6 space-y-6">
    <div v-if="pending" class="text-sm text-muted-foreground">Loading\u2026</div>

    <template v-else-if="indexData">
      <div
        v-if="feedback"
        class="rounded-md border px-4 py-3 text-sm"
        :class="feedbackType === 'error'
          ? 'border-destructive/40 bg-destructive/10 text-destructive'
          : 'border-green-500/40 bg-green-500/10 text-green-700 dark:text-green-400'"
      >
        {{ feedback }}
      </div>

      <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
        <StatCard
          title="Documents"
          :value="(indexData.doc_count ?? 0).toLocaleString()"
          :sub="`${(indexData.pending_docs ?? 0).toLocaleString()} en attente`"
        />
        <StatCard
          title="Stockage"
          :value="formatBytes(totalDiskSize)"
          sub="compressé"
        />
        <StatCard
          title="Données brutes"
          :value="formatBytes(indexData.raw_bytes_ingested ?? 0)"
          sub="non compressé"
        />
        <StatCard
          title="Compression"
          :value="compressionAlgo"
          :sub="compressionLevel ? `niveau ${compressionLevel}` : ''"
        />
      </div>

      <Tabs default-value="segments">
        <TabsList class="gap-1 bg-sidebar">
          <TabsTrigger value="segments">
            <Icon icon="majesticons:collection" class="size-4 mr-1" />
            Segments
          </TabsTrigger>
          <TabsTrigger value="settings">
            <Icon icon="majesticons:cog" class="size-4 mr-1" />
            Settings
          </TabsTrigger>
        </TabsList>

        <TabsContent value="segments" class="space-y-4 mt-4">
          <div class="flex items-center justify-between">
            <p class="text-sm text-muted-foreground">
              {{ segments.length }} segment{{ segments.length !== 1 ? 's' : '' }}
            </p>
            <Button variant="secondary" size="sm" :disabled="committing" @click="handleCommit">
              <Icon icon="majesticons:save" class="size-4 mr-2" />
              {{ committing ? 'Committing\u2026' : 'Commit & merge' }}
            </Button>
          </div>

          <template v-if="segments.length">
            <DataTable :columns="segmentColumns" :data="segments" />
            <p class="text-xs text-muted-foreground">
              {{ formatBytes(segments.reduce((a, s) => a + s.size_bytes, 0)) }} total on disk
            </p>
          </template>

          <p v-else class="text-sm text-muted-foreground text-center py-8">
            No segments yet. Ingest documents then commit.
          </p>

          <p v-if="indexData.pending_docs > 0" class="text-xs text-muted-foreground">
            <Icon icon="majesticons:clock" class="size-3.5 inline mr-1 align-middle" />
            {{ (indexData.pending_docs ?? 0).toLocaleString() }} docs buffered, not yet committed
          </p>
        </TabsContent>


        <TabsContent value="settings" class="space-y-6 mt-4">
          <!-- Merge policy -->
          <Card class="bg-sidebar">
            <CardHeader class="pb-3">
              <CardTitle class="text-sm font-semibold">Merge policy</CardTitle>
            </CardHeader>
            <CardContent class="space-y-3">
              <div class="grid grid-cols-2 gap-3">
                <div class="space-y-1">
                  <label class="text-xs text-muted-foreground">Target docs / segment</label>
                  <Input v-model.number="mergeSettings.merge_target_docs" type="number" min="1" />
                </div>
                <div class="space-y-1">
                  <label class="text-xs text-muted-foreground">Max merge factor</label>
                  <Input v-model.number="mergeSettings.max_merge_factor" type="number" min="2" />
                </div>
              </div>
              <Button size="sm" :disabled="savingMerge" @click="handleSaveMerge">
                {{ savingMerge ? 'Saving…' : 'Apply' }}
              </Button>
            </CardContent>
          </Card>

          <!-- Delete by query -->
          <Card class="border-orange-500/30 bg-sidebar">
            <CardHeader class="pb-3">
              <CardTitle class="text-sm font-semibold text-orange-600 dark:text-orange-400">Delete by query</CardTitle>
            </CardHeader>
            <CardContent class="space-y-3">
              <p class="text-xs text-muted-foreground">Delete all documents matching a tantivy query. A commit is performed immediately after.</p>
              <Input v-model="deleteQuery" placeholder="title:example OR body:test" />
              <AlertDialog>
                <AlertDialogTrigger as-child>
                  <Button variant="outline" size="sm" :disabled="!deleteQuery || deleting" class="border-orange-500/40 text-orange-600 dark:text-orange-400 hover:bg-orange-500/10">
                    <Icon icon="majesticons:minus-circle" class="size-4 mr-2" />
                    {{ deleting ? 'Deleting…' : 'Delete matching docs' }}
                  </Button>
                </AlertDialogTrigger>
                <AlertDialogContent>
                  <AlertDialogHeader>
                    <AlertDialogTitle>Delete matching documents?</AlertDialogTitle>
                    <AlertDialogDescription>
                      All documents matching <code class="text-xs bg-muted px-1 rounded">{{ deleteQuery }}</code> will be permanently deleted and committed. This cannot be undone.
                    </AlertDialogDescription>
                  </AlertDialogHeader>
                  <AlertDialogFooter>
                    <AlertDialogCancel>Cancel</AlertDialogCancel>
                    <AlertDialogAction
                      class="bg-orange-600 text-white hover:bg-orange-700"
                      @click="handleDeleteByQuery"
                    >
                      Delete
                    </AlertDialogAction>
                  </AlertDialogFooter>
                </AlertDialogContent>
              </AlertDialog>
            </CardContent>
          </Card>

          <!-- Danger zone -->
          <Card class="border-destructive/40 bg-sidebar">
            <CardHeader class="pb-3">
              <CardTitle class="text-sm font-semibold text-destructive">Danger zone</CardTitle>
            </CardHeader>
            <CardContent>
              <div class="flex items-center justify-between">
                <div>
                  <p class="text-sm font-medium">Delete this index</p>
                  <p class="text-xs text-muted-foreground mt-0.5">Permanently removes all data. Cannot be undone.</p>
                </div>
                <AlertDialog>
                  <AlertDialogTrigger as-child>
                    <Button variant="destructive" size="sm">
                      <Icon icon="majesticons:trash" class="size-4 mr-2" />
                      Delete index
                    </Button>
                  </AlertDialogTrigger>
                  <AlertDialogContent>
                    <AlertDialogHeader>
                      <AlertDialogTitle>Delete "{{ indexData.name }}"?</AlertDialogTitle>
                      <AlertDialogDescription>
                        This permanently deletes the index and all its {{ (indexData.doc_count ?? 0).toLocaleString() }} documents. This action cannot be undone.
                      </AlertDialogDescription>
                    </AlertDialogHeader>
                    <AlertDialogFooter>
                      <AlertDialogCancel>Cancel</AlertDialogCancel>
                      <AlertDialogAction
                        class="bg-destructive text-destructive-foreground hover:bg-destructive/90"
                        @click="handleDelete"
                      >
                        Delete
                      </AlertDialogAction>
                    </AlertDialogFooter>
                  </AlertDialogContent>
                </AlertDialog>
              </div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </template>

    <div v-else class="text-sm text-destructive">
      Index not found or server unreachable.
    </div>
  </div>

  <Teleport to="#sidebar-panel">
    <div class="space-y-5">
      <!-- Compression -->
      <div>
        <h3 class="text-sm font-semibold mb-3">Docstore</h3>
        <div class="space-y-1.5">
          <div class="flex items-center justify-between text-sm">
            <span class="text-sidebar-foreground/60 text-xs">Compression</span>
            <div class="flex items-center gap-1.5">
              <Badge variant="outline" class="text-[10px] px-1.5 py-0 h-4 font-mono">
                {{ compressionAlgo }}
              </Badge>
              <Badge
                v-if="compressionLevel !== null"
                variant="outline"
                class="text-[10px] px-1.5 py-0 h-4 font-mono text-blue-600 dark:text-blue-400 border-blue-500/30"
              >
                level {{ compressionLevel }}
              </Badge>
            </div>
          </div>
          <div class="flex items-center justify-between text-sm">
            <span class="text-sidebar-foreground/60 text-xs">Block size</span>
            <span class="text-xs font-mono">{{ fmtBlockSize }}</span>
          </div>
        </div>
      </div>

      <Separator />

      <!-- Schema -->
      <div>
      <h3 class="text-sm font-semibold mb-3">Schema</h3>
      <div class="space-y-2">
        <div
          v-for="f in indexData?.schema?.fields ?? []"
          :key="f.name"
          class="border border-sidebar-border rounded-md px-2.5 py-2 text-sm"
        >
          <div class="flex items-center justify-between gap-1">
            <span class="font-medium truncate">{{ f.name }}</span>
            <Badge variant="outline" class="text-[10px] px-1 py-0 h-4 shrink-0">{{ f.type }}</Badge>
          </div>
          <div class="flex gap-2 mt-1">
            <div class="flex gap-2">
              <span v-if="f.stored" class="text-[10px] text-sidebar-foreground/60">stored</span>
              <span v-if="f.indexed" class="text-[10px] text-sidebar-foreground/60">indexed</span>
              <span v-if="f.fast" class="text-[10px] text-sidebar-foreground/60">fast</span>
            </div>
            <span v-if="f.type === 'text' || f.type === 'json' || f.type === 'array<text>' || f.type === 'array<json>'" class="ml-auto text-[10px] text-sidebar-foreground/40">{{ f.tokenizer }}</span>
          </div>
        </div>
        <p v-if="!(indexData?.schema?.fields?.length)" class="text-sm text-sidebar-foreground/60">No fields defined.</p>
      </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup>
import { Icon } from '@iconify/vue'
import { h } from 'vue'

const route = useRoute()
const router = useRouter()
const name = computed(() => route.params.name)

useHead({ title: computed(() => `${name.value} — Index`) })
const { getIndex, commitIndex, deleteIndex, getSegments, deleteByQuery, setIndexSettings } = useTantex()
const { setBreadcrumb } = useBreadcrumb()
const { lastEvent } = useStreamEvents()

const indexData = ref(null)
const pending = ref(true)
const committing = ref(false)
const feedback = ref('')
const feedbackType = ref('success')

// Delete by query
const deleteQuery = ref('')
const deleting = ref(false)

// Merge settings
const mergeSettings = ref({ merge_target_docs: 20_000_000, max_merge_factor: 10 })
const savingMerge = ref(false)

const totalDiskSize = computed(() =>
  segments.value.reduce((a, s) => a + (s.size_bytes ?? 0), 0)
)

const compressionAlgo = computed(() => {
  const raw = indexData.value?.schema?.compression
  if (!raw || raw === 'none') return raw ?? 'lz4'
  return raw.split(':')[0]
})

const compressionLevel = computed(() => {
  const raw = indexData.value?.schema?.compression
  if (!raw) return null
  const parts = raw.split(':')
  return parts.length > 1 ? Number(parts[1]) : null
})

const fmtBlockSize = computed(() => {
  const bs = indexData.value?.schema?.block_size ?? 16384
  if (bs >= 1024 * 1024) return `${(bs / 1024 / 1024).toFixed(0)} MB`
  if (bs >= 1024) return `${(bs / 1024).toFixed(0)} KB`
  return `${bs} B`
})

const segments = ref([])

const segmentColumns = [
  { id: 'segment_id', header: () => 'Segment ID', accessorKey: 'segment_id', cell: ({ getValue }) => h('span', { class: 'font-mono text-xs' }, getValue()) },
  { id: 'num_docs', header: () => 'Documents', accessorKey: 'num_docs', cell: ({ getValue }) => h('span', { class: 'tabular-nums' }, (getValue() ?? 0).toLocaleString()) },
  { id: 'num_deleted_docs', header: () => 'Deleted', accessorKey: 'num_deleted_docs', cell: ({ getValue }) => h('span', { class: 'tabular-nums' }, (getValue() ?? 0).toLocaleString()) },
  { id: 'size', header: () => 'Size', accessorKey: 'size_bytes', cell: ({ getValue }) => h('span', { class: 'tabular-nums text-muted-foreground' }, formatBytes(getValue())) },
]

function formatBytes(bytes) {
  if (!bytes) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return (bytes / Math.pow(k, i)).toFixed(1) + ' ' + sizes[i]
}

async function loadSegments() {
  try {
    const res = await getSegments(name.value)
    segments.value = res.segments ?? []
  } catch (e) {
    console.warn('Failed to load segments:', e)
    segments.value = []
  }
}

onMounted(() => {
  useState('sidebar-visible').value = true
})

onUnmounted(() => {
  useState('sidebar-visible').value = false
})

async function load() {
  pending.value = true
  try {
    const [idx, cfg] = await Promise.all([getIndex(name.value), $fetch('/api/config')])
    indexData.value = idx
    mergeSettings.value = {
      merge_target_docs: cfg.merge_target_docs ?? 20_000_000,
      max_merge_factor: cfg.max_merge_factor ?? 10,
    }
    setBreadcrumb([
      { label: 'Indexes', href: '/indexes' },
      { label: indexData.value.name },
    ])
    await loadSegments()
  } catch {
    indexData.value = null
  } finally {
    pending.value = false
  }
}

async function handleCommit() {
  committing.value = true
  feedback.value = ''
  try {
    await commitIndex(name.value)
    feedback.value = 'Committed successfully.'
    feedbackType.value = 'success'
    await load()
    await loadSegments()
  } catch (e) {
    feedback.value = e.message ?? 'Commit failed'
    feedbackType.value = 'error'
  } finally {
    committing.value = false
  }
}

async function handleDelete() {
  try {
    await deleteIndex(name.value)
    router.push('/indexes')
  } catch (e) {
    feedback.value = e.message ?? 'Delete failed'
    feedbackType.value = 'error'
  }
}

async function handleDeleteByQuery() {
  if (!deleteQuery.value) return
  deleting.value = true
  feedback.value = ''
  try {
    const res = await deleteByQuery(name.value, deleteQuery.value)
    feedback.value = `${(res.deleted ?? 0).toLocaleString()} document(s) deleted and committed.`
    feedbackType.value = 'success'
    deleteQuery.value = ''
    await load()
    await loadSegments()
  } catch (e) {
    feedback.value = e.data?.message ?? e.message ?? 'Delete failed'
    feedbackType.value = 'error'
  } finally {
    deleting.value = false
  }
}

async function handleSaveMerge() {
  savingMerge.value = true
  feedback.value = ''
  try {
    await setIndexSettings(name.value, mergeSettings.value)
    feedback.value = 'Merge policy updated.'
    feedbackType.value = 'success'
  } catch (e) {
    feedback.value = e.data?.message ?? e.message ?? 'Failed to update merge policy'
    feedbackType.value = 'error'
  } finally {
    savingMerge.value = false
  }
}

watch(lastEvent, (event) => {
  if (!event || !indexData.value) return
  if (event.type === 'segments_updated' && event.index === name.value) {
    loadSegments()
  }
  if (event.type === 'metrics' && event.status === 'online') {
    const idx = event.indexes?.find((i) => i.name === name.value)
    if (idx) {
      indexData.value.doc_count = idx.doc_count
      indexData.value.num_segments = idx.num_segments
      indexData.value.pending_docs = idx.pending_docs ?? 0
    }
  }
})

await load()
</script>
