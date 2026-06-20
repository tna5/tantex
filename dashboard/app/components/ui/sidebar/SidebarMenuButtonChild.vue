<script setup>
import { Primitive } from "reka-ui"
import { cn } from "@/lib/utils"
import { sidebarMenuButtonVariants } from "."

const props = defineProps({ isActive: null, class: null, as: { type: String, default: "button" }, variant: { type: String, default: "default" }, size: { type: String, default: "default" } })
</script>

<template>
  <Primitive
    data-slot="sidebar-menu-button"
    data-sidebar="menu-button"
    :data-size="size"
    :data-active="isActive || undefined"
    :class="cn(sidebarMenuButtonVariants({ variant, size }), props.class)"
    :as="as"
    :as-child="asChild"
    v-bind="$attrs"
  >
    <slot />
  </Primitive>
</template>
