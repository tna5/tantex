<script setup>
import { cn } from "@/lib/utils"

const props = defineProps({ class: null })
</script>

<template>
  <div
    data-slot="sidebar-footer"
    data-sidebar="footer"
    :class="cn('gap-2 p-2 flex flex-col', props.class)"
  >
    <slot />
  </div>
</template>
