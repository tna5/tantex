<template>
  <div class="flex items-center gap-2">
    <Input v-model.number="amount" type="number" min="1" class="w-28" />
    <Select v-model="unit">
      <SelectTrigger class="h-9 w-20 text-sm">
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        <SelectItem value="B">B</SelectItem>
        <SelectItem value="KB">KB</SelectItem>
        <SelectItem value="MB">MB</SelectItem>
        <SelectItem value="GB">GB</SelectItem>
      </SelectContent>
    </Select>
  </div>
</template>

<script setup>
const props = defineProps({ modelValue: { type: Number, required: true } })
const emit = defineEmits(['update:modelValue'])

const FACTORS = { B: 1, KB: 1_000, MB: 1_000_000, GB: 1_000_000_000 }

function pickUnit(bytes) {
  if (bytes >= 1_000_000_000 && bytes % 1_000_000_000 === 0) return 'GB'
  if (bytes >= 1_000_000 && bytes % 1_000_000 === 0) return 'MB'
  if (bytes >= 1_000 && bytes % 1_000 === 0) return 'KB'
  if (bytes >= 1_000_000_000) return 'GB'
  if (bytes >= 1_000_000) return 'MB'
  if (bytes >= 1_000) return 'KB'
  return 'B'
}

const unit = ref(pickUnit(props.modelValue))
const amount = ref(props.modelValue / FACTORS[unit.value])

watch([amount, unit], () => {
  const next = Math.round(amount.value * FACTORS[unit.value])
  if (next !== props.modelValue) emit('update:modelValue', next)
})

watch(() => props.modelValue, (next) => {
  const computed = Math.round(amount.value * FACTORS[unit.value])
  if (next !== computed) {
    const u = pickUnit(next)
    unit.value = u
    amount.value = next / FACTORS[u]
  }
})
</script>
