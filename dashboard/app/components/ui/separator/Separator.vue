<script setup>
import { reactiveOmit } from "@vueuse/core"
import { Separator } from "reka-ui"
import { cn } from "@/lib/utils"

const props = defineProps({ class: null, orientation: { type: String, default: 'horizontal' }, decorative: { type: Boolean, default: true } })

const delegatedProps = reactiveOmit(props, "class")
</script>

<template>
  <Separator
    data-slot="separator"
    v-bind="delegatedProps"
    :class="
      cn(
        'shrink-0 bg-border data-[orientation=horizontal]:h-px data-[orientation=horizontal]:w-full data-[orientation=vertical]:w-px data-[orientation=vertical]:self-stretch',
        props.class,
      )
    "
  />
</template>
