<template>
  <div class="p-6 space-y-6">
    <Teleport to="#header-actions">
      <Button variant="secondary" size="sm" @click="createOpen = true">
        <Icon icon="majesticons:plus" class="size-4 mr-2" />
        New Index
      </Button>
    </Teleport>

    <Dialog v-model:open="createOpen">

      <DialogContent class="sm:max-w-3xl max-h-[85vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>Create Index</DialogTitle>
          <DialogDescription>Define the schema for your new index.</DialogDescription>
        </DialogHeader>

        <div class="space-y-5 py-1">
          <!-- Index name -->
          <div class="space-y-1.5">
            <label class="text-sm font-medium">Index name</label>
            <Input
              v-model="form.name"
              placeholder="my-index"
              class="max-w-xs"
              @keydown.enter="submitCreate"
            />
          </div>

          <!-- Schema fields -->
          <div class="space-y-2">
            <div class="flex items-center justify-between">
              <label class="text-sm font-medium">Fields</label>
              <Button variant="outline" size="sm" @click="addField">
                <Icon icon="majesticons:plus" class="size-3 mr-1" />
                Add field
              </Button>
            </div>

            <!-- Column headers -->
            <div class="grid grid-cols-[minmax(0,1fr)_11rem_2.25rem_2.25rem_2.25rem_minmax(0,0.5fr)_1.75rem] gap-x-2 text-xs font-medium text-muted-foreground px-2 pb-0.5">
              <span>Name</span>
              <span>Type</span>
              <Tooltip>
                <TooltipTrigger class="flex justify-center w-full cursor-default">
                  <Icon icon="majesticons:save" class="size-3.5" />
                </TooltipTrigger>
                <TooltipContent>Stored (retrievable in results)</TooltipContent>
              </Tooltip>
              <Tooltip>
                <TooltipTrigger class="flex justify-center w-full cursor-default">
                  <Icon icon="majesticons:search" class="size-3.5" />
                </TooltipTrigger>
                <TooltipContent>Indexed (searchable / filterable)</TooltipContent>
              </Tooltip>
              <Tooltip>
                <TooltipTrigger class="flex justify-center w-full cursor-default">
                  <Icon icon="majesticons:flash" class="size-3.5" />
                </TooltipTrigger>
                <TooltipContent>Fast (columnar storage for sorting & aggregation)</TooltipContent>
              </Tooltip>
              <span>Tokenizer</span>
              <span />
            </div>

            <!-- Field rows -->
            <div v-if="form.fields.length" class="space-y-1.5">
              <div
                v-for="(field, i) in form.fields"
                :key="i"
                class="grid grid-cols-[minmax(0,1fr)_11rem_2.25rem_2.25rem_2.25rem_minmax(0,0.5fr)_1.75rem] gap-x-2 items-center bg-muted/40 rounded-lg px-2 py-1.5"
              >
                <Input v-model="field.name" placeholder="field_name" class="h-8 text-sm" />
                <Select v-model="field.type">
                  <SelectTrigger class="h-8 text-sm">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectGroup>
                      <SelectLabel class="text-[10px] uppercase tracking-wider">Scalar</SelectLabel>
                      <SelectItem v-for="t in scalarTypes" :key="t" :value="t">{{ t }}</SelectItem>
                    </SelectGroup>
                    <SelectSeparator />
                    <SelectGroup>
                      <SelectLabel class="text-[10px] uppercase tracking-wider">Array</SelectLabel>
                      <SelectItem v-for="t in arrayTypes" :key="t" :value="t">{{ t }}</SelectItem>
                    </SelectGroup>
                  </SelectContent>
                </Select>
                <div class="flex justify-center">
                  <Checkbox v-model:checked="field.stored" />
                </div>
                <div class="flex justify-center">
                  <Checkbox v-model:checked="field.indexed" />
                </div>
                <div class="flex justify-center">
                  <Checkbox v-model:checked="field.fast" />
                </div>
                <Input
                  v-if="field.type === 'text' || field.type === 'json' || field.type === 'array<text>' || field.type === 'array<json>'"
                  v-model="field.tokenizer"
                  placeholder="default"
                  class="h-8 text-sm"
                />
                <span v-else />
                <Button
                  variant="ghost"
                  size="sm"
                  class="h-7 w-7 p-0 text-muted-foreground hover:text-destructive"
                  @click="removeField(i)"
                >
                  <Icon icon="majesticons:close" class="size-4" />
                </Button>
              </div>
            </div>

            <div v-else class="flex flex-col items-center gap-2 py-8 text-center rounded-lg border border-dashed border-border">
              <Icon icon="majesticons:table" class="size-5 text-muted-foreground/60" />
              <p class="text-xs text-muted-foreground">No fields yet — add at least one field.</p>
            </div>
          </div>

          <!-- Storage -->
          <div class="space-y-3 pt-1 border-t">
            <label class="text-sm font-medium">Storage</label>
            <div class="grid grid-cols-2 gap-3">
              <div class="space-y-1.5">
                <label class="text-xs text-muted-foreground">Compression</label>
                <Select v-model="form.compression">
                  <SelectTrigger class="h-8 text-sm">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="lz4">lz4 — fast (default)</SelectItem>
                    <SelectItem value="zstd">zstd — better ratio</SelectItem>
                    <SelectItem value="zstd:3">zstd:3 — balanced</SelectItem>
                    <SelectItem value="zstd:9">zstd:9 — high ratio</SelectItem>
                    <SelectItem value="none">none</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div class="space-y-1.5">
                <label class="text-xs text-muted-foreground">Block size (bytes)</label>
                <Input v-model.number="form.block_size" type="number" min="1024" step="1024" class="h-8 text-sm" />
              </div>
            </div>
            <p class="text-xs text-muted-foreground">Compression applies to the doc store (stored fields). Block size is bytes per compressed block (default 16384).</p>
          </div>

          <!-- Create error -->
          <div v-if="createError" class="flex items-center gap-2 rounded-lg bg-destructive/10 border border-destructive/20 px-3 py-2 text-sm text-destructive">
            <Icon icon="majesticons:alert-circle" class="size-4 shrink-0" />
            {{ createError }}
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" @click="createOpen = false">Cancel</Button>
          <Button
            variant="secondary"
            :disabled="creating || !form.name.trim() || !form.fields.length"
            @click="submitCreate"
          >
            <Icon v-if="creating" icon="majesticons:loader" class="size-4 mr-1.5 animate-spin" />
            {{ creating ? 'Creating…' : 'Create' }}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>

    <!-- Page-level error (load / delete) -->
    <div
      v-if="error"
      class="flex items-center gap-2 rounded-lg bg-destructive/10 border border-destructive/20 px-4 py-3 text-sm text-destructive"
    >
      <Icon icon="majesticons:alert-circle" class="size-4 shrink-0" />
      {{ error }}
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-[1fr_280px] gap-6">
      <DataTable v-if="indexes.length" :columns="columns" :data="indexes" />

      <div v-else-if="!pending" class="bg-sidebar rounded-(--radius-xl) border overflow-hidden">
        <EmptyState
          icon="majesticons:table"
          title="No indexes yet"
          description="Create an index to start ingesting and searching documents."
          action-label="New Index"
          @action="createOpen = true"
        />
      </div>

      <div class="bg-sidebar rounded-(--radius-xl) border p-5 space-y-2 self-start">
        <div class="flex items-center gap-2">
          <Icon icon="majesticons:information-circle" class="size-4 text-muted-foreground" />
          <h2 class="text-sm font-semibold text-muted-foreground uppercase tracking-wide">About Indexes</h2>
        </div>
        <p class="text-xs text-muted-foreground leading-relaxed">
          An index stores your documents in a schema-based search engine. Each index has a set of fields with defined types, stored and indexed options, and a tokenizer for text fields.
        </p>
        <NuxtLink to="/docs" class="inline-flex items-center gap-1 text-xs font-medium text-primary hover:underline">
          View documentation
          <Icon icon="majesticons:arrow-right" class="size-3" />
        </NuxtLink>
      </div>
    </div>

    <AlertDialog :open="deleteOpen" @update:open="deleteOpen = $event">
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Delete "{{ deleteTarget }}"?</AlertDialogTitle>
          <AlertDialogDescription>
            This permanently deletes the index and all its data. This action cannot be undone.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>Cancel</AlertDialogCancel>
          <AlertDialogAction
            class="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            @click="handleDelete(deleteTarget)"
          >
            Delete
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  </div>
</template>

<script setup>
import { h } from 'vue'
import { Icon } from '@iconify/vue'
import { Button } from '@/components/ui/button'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'

useHead({ title: 'Indexes' })

const { listIndexes, createIndex, deleteIndex } = useTantex()
const { setBreadcrumb } = useBreadcrumb()

setBreadcrumb([{ label: 'Indexes' }])

const indexes = ref([])

const columns = [
  {
    id: 'name',
    header: () => 'Name',
    accessorKey: 'name',
    cell: ({ getValue, row }) =>
      h('a', {
        href: `/indexes/${row.original.name}`,
        class: 'font-medium hover:underline cursor-pointer',
      }, getValue()),
  },
  { id: 'doc_count', header: () => 'Documents', accessorKey: 'doc_count', cell: ({ getValue }) => (getValue() ?? 0).toLocaleString() },
  { id: 'num_segments', header: () => 'Segments', accessorKey: 'num_segments', cell: ({ getValue }) => (getValue() ?? 0).toLocaleString() },
  {
    id: 'actions',
    header: () => '',
    cell: ({ row }) =>
      h('div', { class: 'text-right' },
        h(Tooltip, {}, [
          h(TooltipTrigger, { asChild: true }, [
            h(Button, {
              variant: 'ghost',
              size: 'sm',
              class: 'text-destructive hover:text-destructive',
              onClick: (e) => {
                e.stopPropagation()
                confirmDelete(row.original.name)
              },
            }, [
              h(Icon, { icon: 'majesticons:trash', class: 'size-4' }),
            ]),
          ]),
          h(TooltipContent, {}, 'Delete index'),
        ]),
      ),
  },
]

const deleteTarget = ref('')
const deleteOpen = ref(false)

function confirmDelete(name) {
  deleteTarget.value = name
  deleteOpen.value = true
}

const scalarTypes = ['text', 'u64', 'i64', 'f64', 'date', 'bool', 'bytes', 'json', 'ip']
const arrayTypes = ['array<text>', 'array<u64>', 'array<i64>', 'array<f64>', 'array<date>', 'array<bool>', 'array<ip>']

const pending = ref(true)
const error = ref('')
const createOpen = ref(false)
const createError = ref('')
const creating = ref(false)

const defaultField = () => ({
  name: '',
  type: 'text',
  stored: true,
  indexed: true,
  fast: false,
  tokenizer: 'default',
})

const form = reactive({
  name: '',
  fields: [defaultField()],
  compression: 'lz4',
  block_size: 16384,
})

function resetForm() {
  form.name = ''
  form.fields = [defaultField()]
  form.compression = 'lz4'
  form.block_size = 16384
  createError.value = ''
}

watch(createOpen, (open) => {
  if (!open) resetForm()
})

async function load() {
  pending.value = true
  error.value = ''
  try {
    const res = await listIndexes()
    indexes.value = res.indexes ?? []
  } catch (e) {
    error.value = e.message ?? 'Failed to load indexes'
  } finally {
    pending.value = false
  }
}

function addField() {
  form.fields.push(defaultField())
}

function removeField(i) {
  form.fields.splice(i, 1)
}

async function submitCreate() {
  if (!form.name.trim() || !form.fields.length || creating.value) return
  creating.value = true
  createError.value = ''
  try {
    await createIndex(form.name, {
      fields: form.fields.map(f => ({ ...f })),
      compression: form.compression,
      block_size: form.block_size,
    })
    createOpen.value = false
    await load()
  } catch (e) {
    createError.value = e.message ?? 'Failed to create index'
  } finally {
    creating.value = false
  }
}

async function handleDelete(name) {
  error.value = ''
  try {
    await deleteIndex(name)
    await load()
  } catch (e) {
    error.value = e.message ?? 'Failed to delete index'
  }
}

await load()
</script>
