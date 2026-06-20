<template>
  <div
    class="rounded-xl border bg-card px-4 py-3 flex items-center gap-3 transition-colors"
    :class="accentBorder"
  >
    <div
      v-if="icon"
      class="size-9 rounded-lg flex items-center justify-center shrink-0"
      :class="accentBg"
    >
      <Icon :icon="icon" class="size-4" :class="accentIcon" />
    </div>
    <div class="min-w-0 flex-1">
      <div class="text-[11px] uppercase tracking-wide text-muted-foreground">{{ label }}</div>
      <div
        class="text-lg font-semibold leading-tight truncate"
        :class="[mono ? 'tabular-nums font-mono' : '', accentValue]"
        :title="String(value)"
      >
        {{ value }}
      </div>
      <div v-if="sub" class="text-[11px] text-muted-foreground tabular-nums mt-0.5 truncate" :title="sub">{{ sub }}</div>
    </div>
  </div>
</template>

<script setup>
import { Icon } from '@iconify/vue'

const props = defineProps({
  label: String,
  value: [String, Number],
  sub: String,
  icon: String,
  accent: String,
  mono: Boolean,
})

const a = computed(() => props.accent || 'default')

const accentBg = computed(() => ({
  green: 'bg-green-500/10',
  blue:  'bg-blue-500/10',
  amber: 'bg-amber-500/10',
  red:   'bg-red-500/10',
  default: 'bg-muted',
}[a.value] ?? 'bg-muted'))

const accentIcon = computed(() => ({
  green: 'text-green-600 dark:text-green-400',
  blue:  'text-blue-600 dark:text-blue-400',
  amber: 'text-amber-600 dark:text-amber-400',
  red:   'text-red-600 dark:text-red-400',
  default: 'text-muted-foreground',
}[a.value] ?? 'text-muted-foreground'))

const accentValue = computed(() => ({
  green: 'text-green-700 dark:text-green-300',
  blue:  'text-blue-700 dark:text-blue-300',
  amber: 'text-amber-700 dark:text-amber-300',
  red:   'text-red-700 dark:text-red-300',
  default: 'text-foreground',
}[a.value] ?? 'text-foreground'))

const accentBorder = computed(() => ({
  green: 'border-green-500/30',
  blue:  'border-blue-500/30',
  amber: 'border-amber-500/30',
  red:   'border-red-500/30',
  default: '',
}[a.value] ?? ''))
</script>
