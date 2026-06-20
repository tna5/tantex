<script setup>
import { Primitive } from "reka-ui"
import { cn } from "@/lib/utils"

const props = defineProps({ as: { type: String, default: 'a' }, class: null })
</script>

<template>
  <Primitive
    data-slot="breadcrumb-link"
    :as="as"
    :as-child="asChild"
    :class="cn('hover:text-foreground transition-colors', props.class)"
  >
    <slot />
  </Primitive>
</template>
