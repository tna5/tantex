<script setup>
import { SelectLabel } from "reka-ui"
import { cn } from "@/lib/utils"

const props = defineProps({ class: null })
</script>

<template>
  <SelectLabel
    data-slot="select-label"
    :class="cn('text-muted-foreground px-1.5 py-1 text-xs', props.class)"
  >
    <slot />
  </SelectLabel>
</template>
