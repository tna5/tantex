<script setup>
import { cn } from "@/lib/utils"

const props = defineProps({ class: null, size: { type: String, default: 'default' } })
</script>

<template>
  <div
    data-slot="card"
    :data-size="size"
    :class="cn('bg-sidebar text-card-foreground gap-4 overflow-hidden rounded-(--radius-xl) border py-4 text-sm has-data-[slot=card-footer]:pb-0 has-[>img:first-child]:pt-0 data-[size=sm]:gap-3 data-[size=sm]:py-3 data-[size=sm]:has-data-[slot=card-footer]:pb-0 group/card flex flex-col', props.class)"
  >
    <slot />
  </div>
</template>
