<script setup>
import { cn } from "@/lib/utils"

const props = defineProps({ class: null })
</script>

<template>
  <tr
    data-slot="table-row"
    :class="cn('hover:bg-muted/50 data-[state=selected]:bg-muted transition-colors has-aria-expanded:bg-muted/50', props.class)"
  >
    <slot />
  </tr>
</template>
