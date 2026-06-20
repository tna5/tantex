<script setup>
import { reactiveOmit } from "@vueuse/core"
import { DialogTitle } from "reka-ui"
import { cn } from "@/lib/utils"

const props = defineProps({ class: null })

const delegatedProps = reactiveOmit(props, "class")
</script>

<template>
  <DialogTitle
    data-slot="sheet-title"
    :class="cn('text-foreground text-base font-medium cn-font-heading', props.class)"
    v-bind="delegatedProps"
  >
    <slot />
  </DialogTitle>
</template>
