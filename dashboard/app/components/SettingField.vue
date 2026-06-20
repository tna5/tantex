<template>
  <div class="grid grid-cols-1 lg:grid-cols-[260px_minmax(0,1fr)] gap-x-8 gap-y-2 py-4 border-t-[1px] border-sidebar-border first:border-t-0 first:pt-0">
    <div class="space-y-1">
      <div class="flex items-center gap-1.5">
        <label class="text-sm font-medium">{{ label }}</label>
        <TooltipProvider v-if="why" :delay-duration="100">
          <Tooltip>
            <TooltipTrigger as-child>
              <button type="button" class="inline-flex items-center justify-center size-4 rounded-full text-muted-foreground/40 hover:text-muted-foreground transition-colors focus:outline-none">
                <Icon icon="majesticons:question-mark-circle" class="size-3.5" />
              </button>
            </TooltipTrigger>
            <TooltipContent class="max-w-64 text-xs leading-relaxed" side="right">
              {{ why }}
            </TooltipContent>
          </Tooltip>
        </TooltipProvider>
        <span v-if="isChanged" class="size-1.5 rounded-full bg-muted-foreground/40 ml-0.5" />
      </div>
      <p v-if="unit" class="text-xs text-muted-foreground">{{ unit }}</p>
    </div>

    <div class="flex items-start gap-2">
      <slot />
    </div>
  </div>
</template>

<script setup>
import { Icon } from '@iconify/vue'

const props = defineProps({
  label: String,
  value: [String, Number],
  defaultValue: [String, Number],
  unit: String,
  why: String,
})

const isChanged = computed(() => {
  if (props.defaultValue === undefined || props.defaultValue === null) return false
  return String(props.defaultValue) !== String(props.value)
})
</script>
