<template>
  <div class="bg-sidebar rounded-(--radius-xl) border px-3 pt-3 pb-0 overflow-hidden">
    <div class="flex items-start justify-between mb-4">
      <div>
        <h2 class="text-lg font-bold leading-tight text-sidebar-foreground tracking-tight mb-2">
          {{ title }}
        </h2>
        <div class="flex items-center gap-2.5">
          <span class="text-sm font-normal text-muted-foreground tabular-nums">
            {{ value }}<span v-if="unit" class="text-sm text-muted-foreground"> {{ unit }}</span>
          </span>
          <span
            v-if="badge"
            class="text-xs font-semibold rounded-full px-2.5 py-0.5 leading-none"
            :class="badgeClass"
          >
            {{ badge }}
          </span>
        </div>
      </div>
    </div>

    <div class="w-[calc(100%+24px)] -ml-3">
      <svg
        class="block w-full h-[180px]"
        :viewBox="`0 0 ${W} ${H}`"
        preserveAspectRatio="none"
        xmlns="http://www.w3.org/2000/svg"
      >
        <defs>
          <linearGradient :id="`grad-${uid}`" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" :stop-color="palette.line" stop-opacity="0.5" />
            <stop offset="100%" :stop-color="palette.line" stop-opacity="0.02" />
          </linearGradient>
          <clipPath :id="`clip-${uid}`">
            <rect :width="W" :height="H" />
          </clipPath>
        </defs>
        <path
          :d="areaPath"
          :fill="`url(#grad-${uid})`"
          :clip-path="`url(#clip-${uid})`"
        />
        <path
          :d="linePath"
          fill="none"
          :stroke="palette.line"
          stroke-width="2.5"
          stroke-linecap="round"
          stroke-linejoin="round"
          :clip-path="`url(#clip-${uid})`"
        />
      </svg>
    </div>
  </div>
</template>

<script setup>
const props = defineProps({
  title: { type: String, required: true },
  value: { type: String, required: true },
  unit: { type: String, default: '' },
  badge: { type: String, default: '' },
  badgeVariant: { type: String, default: 'green' },
  series: { type: Object, required: true },
  color: { type: String, required: true },
})

const uid = `mc-${Math.random().toString(36).slice(2, 8)}`

const W = 600
const H = 180

const palettes = {
  blue:   { line: '#60a5fa' },
  green:  { line: '#4ade80' },
  violet: { line: '#a78bfa' },
  orange: { line: '#fb923c' },
  slate:  { line: '#94a3b8' },
}
const palette = computed(() => palettes[props.color])

const badgeClass = computed(() => {
  switch (props.badgeVariant) {
    case 'red':   return 'bg-destructive text-destructive-foreground'
    case 'amber': return 'bg-amber-100 text-amber-900 dark:bg-amber-900/40 dark:text-amber-200'
    case 'green':
    default:      return 'bg-sidebar-accent text-sidebar-accent-foreground'
  }
})

function catmullRomToBezier(pts) {
  if (pts.length < 2) return `M 0 ${H / 2} L ${W} ${H / 2}`
  let d = `M ${pts[0].x} ${pts[0].y}`
  for (let i = 0; i < pts.length - 1; i++) {
    const p0 = pts[Math.max(i - 1, 0)]
    const p1 = pts[i]
    const p2 = pts[i + 1]
    const p3 = pts[Math.min(i + 2, pts.length - 1)]
    const cp1x = p1.x + (p2.x - p0.x) / 6
    const cp1y = p1.y + (p2.y - p0.y) / 6
    const cp2x = p2.x - (p3.x - p1.x) / 6
    const cp2y = p2.y - (p3.y - p1.y) / 6
    d += ` C ${cp1x} ${cp1y}, ${cp2x} ${cp2y}, ${p2.x} ${p2.y}`
  }
  return d
}

const svgPoints = computed(() => {
  const data = props.series.data
  if (!data.length) {
    return [{ x: 0, y: H * 0.5 }, { x: W, y: H * 0.5 }]
  }
  const min = Math.min(...data)
  const max = Math.max(...data)
  const range = max - min || 1
  const pad = 10
  return data.map((v, i) => ({
    x: (i / Math.max(data.length - 1, 1)) * W,
    y: pad + (1 - (v - min) / range) * (H - pad * 2),
  }))
})

const linePath = computed(() => catmullRomToBezier(svgPoints.value))
const areaPath = computed(() => {
  const pts = svgPoints.value
  return `${catmullRomToBezier(pts)} L ${pts[pts.length - 1].x} ${H} L ${pts[0].x} ${H} Z`
})
</script>
