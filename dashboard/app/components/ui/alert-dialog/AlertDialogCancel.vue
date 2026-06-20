<script setup>
import { reactiveOmit } from "@vueuse/core"
import { AlertDialogCancel } from "reka-ui"
import { cn } from "@/lib/utils"
import { buttonVariants } from '@/components/ui/button'

const props = defineProps({ class: null, variant: { type: String, default: 'outline' }, size: { type: String, default: 'default' } })

const delegatedProps = reactiveOmit(props, "class", "variant", "size")
</script>

<template>
  <AlertDialogCancel
    data-slot="alert-dialog-cancel"
    v-bind="delegatedProps"
    :class="cn(
      '',
      buttonVariants({ variant, size }),
      props.class,
    )"
  >
    <slot />
  </AlertDialogCancel>
</template>
