<script setup>
import { cn } from "@/lib/utils"

const props = defineProps({ class: null })
</script>

<template>
  <li
    data-slot="breadcrumb-item"
    :class="cn('gap-1 inline-flex items-center', props.class)"
  >
    <slot />
  </li>
</template>
