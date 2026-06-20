<template>
    <div class="p-6 space-y-6">

        <div v-if="loading" class="text-muted-foreground text-sm flex items-center gap-2 py-8">
            <Icon icon="majesticons:loader" class="size-4 animate-spin" />
            Loading server config…
        </div>

        <template v-else>
            <!-- Preset selector -->
            <div class="space-y-3">
                <h2 class="text-sm font-semibold text-muted-foreground uppercase tracking-wide">Preset</h2>
                <div class="grid grid-cols-3 gap-3">
                    <button type="button"
                        class="flex flex-col gap-2.5 rounded-(--radius-xl) bg-sidebar p-4 text-left transition-all duration-150 ring-1"
                        :class="activePreset === 'max-ingest' ? 'ring-foreground/30' : 'ring-transparent hover:ring-foreground/10'"
                        @click="applyPreset('max-ingest')">
                        <div class="flex items-center gap-2">
                            <Icon icon="majesticons:flash" class="size-4 text-muted-foreground" />
                            <span class="text-sm font-medium">Max ingest</span>
                        </div>
                        <p class="text-xs text-muted-foreground">Throughput first, long commits</p>
                        <div class="flex flex-wrap gap-1 mt-0.5">
                            <span class="text-[10px] px-1.5 py-0.5 rounded bg-muted font-mono text-muted-foreground">4 GB heap</span>
                            <span class="text-[10px] px-1.5 py-0.5 rounded bg-muted font-mono text-muted-foreground">10M commit</span>
                        </div>
                    </button>

                    <button type="button"
                        class="flex flex-col gap-2.5 rounded-(--radius-xl) bg-sidebar p-4 text-left transition-all duration-150 ring-1"
                        :class="activePreset === 'balanced' ? 'ring-foreground/30' : 'ring-transparent hover:ring-foreground/10'"
                        @click="applyPreset('balanced')">
                        <div class="flex items-center gap-2">
                            <Icon icon="majesticons:scale" class="size-4 text-muted-foreground" />
                            <span class="text-sm font-medium">Balanced</span>
                        </div>
                        <p class="text-xs text-muted-foreground">Speed and visibility</p>
                        <div class="flex flex-wrap gap-1 mt-0.5">
                            <span class="text-[10px] px-1.5 py-0.5 rounded bg-muted font-mono text-muted-foreground">2 GB heap</span>
                            <span class="text-[10px] px-1.5 py-0.5 rounded bg-muted font-mono text-muted-foreground">1M commit</span>
                        </div>
                    </button>

                    <button type="button"
                        class="flex flex-col gap-2.5 rounded-(--radius-xl) bg-sidebar p-4 text-left transition-all duration-150 ring-1"
                        :class="activePreset === 'low-latency' ? 'ring-foreground/30' : 'ring-transparent hover:ring-foreground/10'"
                        @click="applyPreset('low-latency')">
                        <div class="flex items-center gap-2">
                            <Icon icon="majesticons:clock" class="size-4 text-muted-foreground" />
                            <span class="text-sm font-medium">Low latency</span>
                        </div>
                        <p class="text-xs text-muted-foreground">Fast visibility, small commits</p>
                        <div class="flex flex-wrap gap-1 mt-0.5">
                            <span class="text-[10px] px-1.5 py-0.5 rounded bg-muted font-mono text-muted-foreground">500 MB heap</span>
                            <span class="text-[10px] px-1.5 py-0.5 rounded bg-muted font-mono text-muted-foreground">100K commit</span>
                        </div>
                    </button>
                </div>
            </div>

            <!-- Server info -->
            <div class="bg-sidebar rounded-(--radius-xl) border p-5 space-y-4">
                <div class="flex items-center justify-between">
                    <div class="flex items-center gap-2">
                        <Icon icon="majesticons:server" class="size-4 text-muted-foreground" />
                        <h2 class="text-sm font-semibold text-muted-foreground uppercase tracking-wide">Server</h2>
                    </div>
                    <span class="text-[10px] font-medium px-2 py-0.5 rounded-full bg-muted text-muted-foreground uppercase tracking-wide">read-only</span>
                </div>
                <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
                    <div class="space-y-1">
                        <p class="text-xs text-muted-foreground">Socket path</p>
                        <code class="block bg-muted rounded-lg px-3 py-2 text-xs font-mono truncate">{{ form.socket_path }}</code>
                    </div>
                    <div class="space-y-1">
                        <p class="text-xs text-muted-foreground">Data directory</p>
                        <code class="block bg-muted rounded-lg px-3 py-2 text-xs font-mono truncate">{{ form.data_dir }}</code>
                    </div>
                </div>
            </div>

            <!-- Ingest performance -->
            <div class="bg-sidebar rounded-(--radius-xl) border p-5 space-y-0">
                <div class="flex items-center gap-2 pb-4 mb-1 border-b-[1px] border-sidebar-border">
                    <Icon icon="majesticons:rocket" class="size-4 text-muted-foreground" />
                    <div>
                        <h2 class="text-sm font-semibold text-muted-foreground uppercase tracking-wide">Ingest performance</h2>
                        <p class="text-xs text-muted-foreground/70 mt-0.5">How fast new documents flow from socket → tantivy</p>
                    </div>
                </div>
                <SettingField label="Indexing thread budget" :value="form.num_indexing_threads" :default-value="8"
                    unit="threads"
                    why="Total parallelism (split between tokenization and JSON parsing); 8 is the sweet spot on M2 (4P+4E) — beyond that, threads compete for cores and throughput drops.">
                    <Input v-model.number="form.num_indexing_threads" type="number" min="2" max="64" class="w-28" />
                </SettingField>

                <SettingField label="Index/parse thread split" :value="`${form.index_threads_pct}%`" :default-value="63"
                    unit="% allocated to tantivy index threads"
                    why="Share of the budget given to tantivy index threads vs the rayon parse pool; 63% gives 5/3 at 8 threads, the split benchmarked as fastest on M2.">
                    <div class="flex items-center gap-3">
                        <Input v-model.number="form.index_threads_pct" type="number" min="10" max="90" class="w-24" />
                        <span class="text-xs text-muted-foreground font-mono">{{ derivedSplit }}</span>
                    </div>
                </SettingField>

                <SettingField label="Writer heap" :value="fmtBytes(form.writer_heap_size)"
                    :default-value="fmtBytes(4_000_000_000)" unit="in-RAM buffer before disk flush"
                    why="Bigger heap = fewer disk flushes during ingest; 4 GB lets ~8M docs accumulate before tantivy spills a segment, ideal for sustained writes.">
                    <ByteInput v-model="form.writer_heap_size" />
                </SettingField>

                <SettingField label="SHM buffer" :value="fmtBytes(form.shm_buffer_size)"
                    :default-value="fmtBytes(268_435_456)" unit="per client session"
                    why="Shared-memory window between client and server for zero-copy NDJSON ingest; 256 MB fits any reasonable batch and avoids re-mapping cost.">
                    <ByteInput v-model="form.shm_buffer_size" />
                </SettingField>
            </div>

            <!-- Commit policy -->
            <div class="bg-sidebar rounded-(--radius-xl) border p-5 space-y-0">
                <div class="flex items-center gap-2 pb-4 mb-1 border-b-[1px] border-sidebar-border">
                    <Icon icon="majesticons:save" class="size-4 text-muted-foreground" />
                    <div>
                        <h2 class="text-sm font-semibold text-muted-foreground uppercase tracking-wide">Commit policy</h2>
                        <p class="text-xs text-muted-foreground/70 mt-0.5">When pending docs become visible to searches</p>
                    </div>
                </div>
                <SettingField label="Soft commit threshold" :value="fmtNum(form.auto_commit_doc_count)"
                    :default-value="fmtNum(10_000_000)" unit="docs buffered since last commit"
                    why="Commits trigger when this many docs are buffered AND the writer is idle; 10M maximises throughput by amortising flush cost over many docs.">
                    <Input v-model.number="form.auto_commit_doc_count" type="number" min="1000" class="w-36" />
                </SettingField>

                <SettingField label="Hard commit multiplier" :value="`${form.hard_commit_multiplier}×`"
                    :default-value="4"
                    :unit="`= ${fmtNum(form.auto_commit_doc_count * form.hard_commit_multiplier)} docs hard limit`"
                    why="Forces a commit under continuous load when the idle-based soft trigger never fires; 4× keeps backlog bounded without choking peak throughput.">
                    <Input v-model.number="form.hard_commit_multiplier" type="number" min="1" max="20" class="w-24" />
                </SettingField>

                <SettingField label="Timer commit" :value="`${form.auto_commit_interval_secs}s`" :default-value="30"
                    unit="idle seconds before flush"
                    why="Safety net so docs don't sit forever when ingest stops; 30 s is short enough for near-realtime visibility, long enough to never fire mid-batch.">
                    <Input v-model.number="form.auto_commit_interval_secs" type="number" min="1" class="w-28" />
                </SettingField>
            </div>

            <!-- Merge policy -->
            <div class="bg-sidebar rounded-(--radius-xl) border p-5 space-y-0">
                <div class="flex items-center gap-2 pb-4 mb-1 border-b-[1px] border-sidebar-border">
                    <Icon icon="majesticons:layers" class="size-4 text-muted-foreground" />
                    <div>
                        <h2 class="text-sm font-semibold text-muted-foreground uppercase tracking-wide">Merge policy</h2>
                        <p class="text-xs text-muted-foreground/70 mt-0.5">How tantivy consolidates segments in the background</p>
                    </div>
                </div>
                <SettingField label="Merge target docs" :value="fmtNum(form.merge_target_docs)"
                    :default-value="fmtNum(20_000_000)" unit="docs per consolidated segment"
                    why="Target size for consolidated segments; 20M keeps search fast (few segments to scan) without triggering huge multi-GB merges that stall ingest.">
                    <Input v-model.number="form.merge_target_docs" type="number" min="100000" class="w-36" />
                </SettingField>

                <SettingField label="Max merge factor" :value="form.max_merge_factor" :default-value="10"
                    unit="segments combined per merge pass"
                    why="Cap on how many small segments are combined in one merge pass; 10 spreads merge work over time instead of running rare giant merges.">
                    <Input v-model.number="form.max_merge_factor" type="number" min="2" max="100" class="w-24" />
                </SettingField>

                <SettingField label="Min segments before merge" :value="form.min_num_segments" :default-value="2"
                    unit="segments before a merge is scheduled"
                    why="Below this many small segments, no merge is scheduled; 2 lets pairs merge as soon as they appear, keeping segment count low for search.">
                    <Input v-model.number="form.min_num_segments" type="number" min="2" max="50" class="w-24" />
                </SettingField>
            </div>

            <!-- Save bar -->
            <div :class="[
                'sticky bottom-4 z-10 flex items-center justify-between gap-3 px-5 py-3 rounded-(--radius-xl) shadow-md transition-all duration-200',
                dirty ? 'bg-sidebar' : saved ? 'bg-sidebar' : 'bg-sidebar'
            ]">
                <div class="text-xs flex items-center gap-1.5">
                    <template v-if="error">
                        <Icon icon="majesticons:alert-circle" class="size-3.5 text-destructive" />
                        <span class="text-destructive">{{ error }}</span>
                    </template>
                    <template v-else-if="saved">
                        <Icon icon="majesticons:check-circle" class="size-3.5 text-muted-foreground" />
                        <span class="text-muted-foreground">Saved.</span>
                    </template>
                    <template v-else-if="dirty">
                        <span class="size-1.5 rounded-full bg-muted-foreground/50 inline-block" />
                        <span class="text-muted-foreground">Unsaved changes</span>
                    </template>
                    <template v-else>
                        <span class="text-muted-foreground">All changes saved.</span>
                    </template>
                </div>
                <div class="flex gap-2">
                    <Button variant="ghost" size="sm" :disabled="!dirty || saving" @click="reset">Reset</Button>
                    <Button variant="secondary" size="sm" :disabled="saving || !dirty" @click="save">
                        <Icon v-if="saving" icon="majesticons:loader" class="size-3.5 mr-1.5 animate-spin" />
                        Save changes
                        <kbd class="ml-1.5 hidden sm:inline-flex items-center gap-0.5 rounded border px-1 py-0.5 text-[10px] font-mono opacity-60">⌘S</kbd>
                    </Button>
                </div>
            </div>
        </template>
    </div>
</template>

<script setup>
import { Icon } from '@iconify/vue'

useHead({ title: 'Settings' })

const { getConfig, setConfig } = useTantex()
const { setBreadcrumb } = useBreadcrumb()
setBreadcrumb([{ label: 'Settings' }])

const loading = ref(true)
const saving = ref(false)
const error = ref('')
const saved = ref(false)
const original = ref(null)
const form = ref({
    socket_path: '',
    data_dir: '',
    shm_buffer_size: 268_435_456,
    writer_heap_size: 4_000_000_000,
    auto_commit_doc_count: 10_000_000,
    auto_commit_interval_secs: 30,
    merge_target_docs: 20_000_000,
    max_merge_factor: 10,
    min_num_segments: 2,
    num_indexing_threads: 8,
    index_threads_pct: 62,
    hard_commit_multiplier: 4,
})

const dirty = computed(() => {
    if (!original.value) return false
    return JSON.stringify(form.value) !== JSON.stringify(original.value)
})

const derivedSplit = computed(() => {
    const total = Math.max(2, form.value.num_indexing_threads)
    const pct = Math.min(90, Math.max(10, form.value.index_threads_pct))
    const idx = Math.max(2, Math.floor((total * pct) / 100))
    const parse = Math.max(2, total - idx)
    return `${idx} index + ${parse} parse`
})

onMounted(async () => {
    try {
        const cfg = await getConfig()
        form.value = { ...cfg }
        original.value = { ...cfg }
    } catch (e) {
        error.value = e instanceof Error ? e.message : 'Failed to load config'
    } finally {
        loading.value = false
    }
})

let _keydown = null
onMounted(() => {
    _keydown = (e) => {
        if ((e.metaKey || e.ctrlKey) && e.key === 's') {
            e.preventDefault()
            if (dirty.value && !saving.value) save()
        }
    }
    window.addEventListener('keydown', _keydown)
})
onUnmounted(() => { if (_keydown) window.removeEventListener('keydown', _keydown) })

function reset() {
    if (original.value) form.value = { ...original.value }
    saved.value = false
    error.value = ''
}

const PRESETS = {
    'max-ingest': {
        writer_heap_size: 4_000_000_000,
        auto_commit_doc_count: 10_000_000,
        auto_commit_interval_secs: 60,
        hard_commit_multiplier: 4,
        num_indexing_threads: 8,
        index_threads_pct: 63,
        merge_target_docs: 20_000_000,
        max_merge_factor: 10,
    },
    'balanced': {
        writer_heap_size: 2_000_000_000,
        auto_commit_doc_count: 1_000_000,
        auto_commit_interval_secs: 30,
        hard_commit_multiplier: 4,
        num_indexing_threads: 8,
        index_threads_pct: 63,
        merge_target_docs: 10_000_000,
        max_merge_factor: 10,
    },
    'low-latency': {
        writer_heap_size: 500_000_000,
        auto_commit_doc_count: 100_000,
        auto_commit_interval_secs: 5,
        hard_commit_multiplier: 4,
        num_indexing_threads: 8,
        index_threads_pct: 63,
        merge_target_docs: 5_000_000,
        max_merge_factor: 6,
    },
}

const activePreset = computed(() => {
    for (const [name, preset] of Object.entries(PRESETS)) {
        if (Object.entries(preset).every(([k, v]) => form.value[k] === v)) return name
    }
    return null
})

function applyPreset(name) {
    const p = PRESETS[name]
    if (!p) return
    form.value = { ...form.value, ...p }
}

async function save() {
    saving.value = true
    error.value = ''
    saved.value = false
    try {
        await setConfig({
            shm_buffer_size: form.value.shm_buffer_size,
            writer_heap_size: form.value.writer_heap_size,
            auto_commit_doc_count: form.value.auto_commit_doc_count,
            auto_commit_interval_secs: form.value.auto_commit_interval_secs,
            merge_target_docs: form.value.merge_target_docs,
            max_merge_factor: form.value.max_merge_factor,
            min_num_segments: form.value.min_num_segments,
            num_indexing_threads: form.value.num_indexing_threads,
            index_threads_pct: form.value.index_threads_pct,
            hard_commit_multiplier: form.value.hard_commit_multiplier,
        })
        original.value = { ...form.value }
        saved.value = true
        setTimeout(() => { saved.value = false }, 3000)
    } catch (e) {
        error.value = e instanceof Error ? e.message : 'Failed to save config'
    } finally {
        saving.value = false
    }
}

function fmtNum(n) {
    return n.toLocaleString()
}
function fmtBytes(n) {
    if (n >= 1_000_000_000) return `${(n / 1_000_000_000).toFixed(2)} GB`
    if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)} MB`
    if (n >= 1_000) return `${(n / 1_000).toFixed(1)} KB`
    return `${n} B`
}
</script>
