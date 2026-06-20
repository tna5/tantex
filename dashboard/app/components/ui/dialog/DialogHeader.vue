<script setup>
import { cn } from "@/lib/utils"

const props = defineProps({ class: null })
</script>

<template>
  <div
    data-slot="dialog-header"
    :class="cn('gap-2 flex flex-col', props.class)"
  >
    <slot />
  </div>
</template>
