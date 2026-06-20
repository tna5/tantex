<script setup>
import {
  FlexRender,
  useVueTable,
  getCoreRowModel,
  getSortedRowModel,
} from '@tanstack/vue-table'
import { Icon } from '@iconify/vue'

const props = defineProps({
  columns: { type: Array, required: true },
  data: { type: Array, required: true },
})

const sorting = ref([])

const table = useVueTable({
  get data() { return props.data },
  get columns() { return props.columns },
  getCoreRowModel: getCoreRowModel(),
  getSortedRowModel: getSortedRowModel(),
  onSortingChange: (updaterOrValue) => {
    sorting.value = typeof updaterOrValue === 'function'
      ? updaterOrValue(sorting.value)
      : updaterOrValue
  },
  state: {
    get sorting() { return sorting.value },
  },
})
</script>

<template>
  <div class="overflow-hidden rounded-(--radius-xl) bg-sidebar border">
    <Table>
      <TableHeader class="bg-muted-foreground/[0.06]">
        <TableRow v-for="headerGroup in table.getHeaderGroups()" :key="headerGroup.id">
          <TableHead
            v-for="header in headerGroup.headers"
            :key="header.id"
            :class="{ 'cursor-pointer select-none': header.column.getCanSort() }"
            @click="header.column.getCanSort() && header.column.toggleSorting()"
          >
            <div class="flex items-center gap-1">
              <FlexRender
                v-if="!header.isPlaceholder"
                :render="header.column.columnDef.header"
                :props="header.getContext()"
              />
              <Icon
                v-if="header.column.getCanSort()"
                icon="majesticons:chevron-down"
                class="size-3 transition-transform"
                :class="{
                  'rotate-180': header.column.getIsSorted() === 'asc',
                  'opacity-0': !header.column.getIsSorted(),
                  'opacity-100': header.column.getIsSorted()
                }"
              />
            </div>
          </TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        <TableRow v-for="row in table.getRowModel().rows" :key="row.id" :data-state="row.getIsSelected() && 'selected'">
          <TableCell v-for="cell in row.getVisibleCells()" :key="cell.id">
            <FlexRender :render="cell.column.columnDef.cell" :props="cell.getContext()" />
          </TableCell>
        </TableRow>
        <TableRow v-if="!table.getRowModel().rows.length">
          <TableCell :col-span="columns.length" class="h-24 text-center text-muted-foreground">
            No results.
          </TableCell>
        </TableRow>
      </TableBody>
    </Table>
  </div>
</template>
