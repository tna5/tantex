<script setup>
import { Icon } from '@iconify/vue'

defineProps({
  icon: { type: String, default: 'majesticons:inbox' },
  title: { type: String, required: true },
  description: { type: String, default: '' },
  actionLabel: { type: String, default: '' },
})

const emit = defineEmits(['action'])
</script>

<template>
  <div class="flex flex-col items-center gap-4 py-16 text-center">
    <div class="size-14 rounded-full bg-gradient-to-b from-muted to-muted/50 ring-1 ring-border/40 flex items-center justify-center">
      <Icon :icon="icon" class="size-7 text-muted-foreground/50" />
    </div>
    <div class="max-w-64">
      <p class="text-sm font-medium">{{ title }}</p>
      <p v-if="description" class="text-xs text-muted-foreground mt-1 leading-relaxed">{{ description }}</p>
    </div>
    <Button
      v-if="actionLabel"
      variant="secondary"
      size="sm"
      @click="emit('action')"
    >
      <Icon icon="majesticons:plus" class="size-4 mr-1.5" />
      {{ actionLabel }}
    </Button>
  </div>
</template>
