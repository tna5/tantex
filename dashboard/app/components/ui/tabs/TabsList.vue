<script setup>
import { reactiveOmit } from "@vueuse/core"
import { TabsList } from "reka-ui"
import { cn } from "@/lib/utils"
import { tabsListVariants } from "."

const props = defineProps({ class: null, variant: { type: String, default: "default" } })

const delegatedProps = reactiveOmit(props, "class", "variant")
</script>

<template>
  <TabsList
    data-slot="tabs-list"
    :data-variant="variant"
    v-bind="delegatedProps"
    :class="cn(tabsListVariants({ variant }), props.class)"
  >
    <slot />
  </TabsList>
</template>
