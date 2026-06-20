<template>
  <div class="bg-sidebar rounded-(--radius-xl) border p-3 overflow-hidden font-sans">
    <p class="text-xs font-medium text-muted-foreground uppercase tracking-wider mb-2">{{ title }}</p>
    <div class="flex items-baseline gap-1.5">
      <span class="text-lg font-bold text-sidebar-foreground tracking-tight leading-none tabular-nums">{{ value }}</span>
      <span v-if="unit" class="text-sm text-muted-foreground"> {{ unit }}</span>
    </div>
    <p v-if="sub" class="text-xs text-muted-foreground/60 mt-1.5">{{ sub }}</p>
  </div>
</template>

<script setup>
defineProps({
  title: { type: String, required: true },
  value: { type: String, required: true },
  unit: { type: String, default: '' },
  sub: { type: String, default: '' },
})
</script>
