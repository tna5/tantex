<script setup>
import { IconDots } from '@tabler/icons-vue';

import { cn } from "@/lib/utils"

const props = defineProps({ class: null })
</script>

<template>
  <span
    data-slot="breadcrumb-ellipsis"
    role="presentation"
    aria-hidden="true"
    :class="cn('size-5 [&>svg]:size-4 flex items-center justify-center', props.class)"
  >
    <slot>
      <IconDots />
    </slot>
    <span class="sr-only">More</span>
  </span>
</template>
