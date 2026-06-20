<script setup>
import { IconChevronUp } from '@tabler/icons-vue';

import { reactiveOmit } from "@vueuse/core"
import { SelectScrollUpButton, useForwardProps } from "reka-ui"
import { cn } from "@/lib/utils"

const props = defineProps({ class: {} })

const delegatedProps = reactiveOmit(props, "class")

const forwardedProps = useForwardProps(delegatedProps)
</script>

<template>
  <SelectScrollUpButton
    data-slot="select-scroll-up-button"
    v-bind="forwardedProps"
    :class="cn('bg-popover z-10 flex cursor-default items-center justify-center py-1 [&_svg:not([class*=size-])]:size-4', props.class)"
  >
    <slot>
      <IconChevronUp />
    </slot>
  </SelectScrollUpButton>
</template>
