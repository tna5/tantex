<template>
  <div class="analytics-card">
    <div class="card-header">
      <div class="card-meta">
        <h2 class="card-title">Analytics</h2>
        <div class="card-stats">
          <span class="visitors">418.2K Visitors</span>
          <span class="badge">+10%</span>
        </div>
      </div>
      <button class="view-btn">View Analytics</button>
    </div>

    <div class="chart-wrapper">
      <svg
        class="chart-svg"
        :viewBox="`0 0 ${width} ${height}`"
        preserveAspectRatio="none"
        xmlns="http://www.w3.org/2000/svg"
      >
        <defs>
          <linearGradient id="areaGradient" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stop-color="#22c55e" stop-opacity="0.5" />
            <stop offset="100%" stop-color="#22c55e" stop-opacity="0.02" />
          </linearGradient>
          <clipPath id="chartClip">
            <rect :width="width" :height="height" />
          </clipPath>
        </defs>

        <path
          :d="areaPath"
          fill="url(#areaGradient)"
          clip-path="url(#chartClip)"
        />
        <path
          :d="linePath"
          fill="none"
          stroke="#4ade80"
          stroke-width="2.5"
          stroke-linecap="round"
          stroke-linejoin="round"
          clip-path="url(#chartClip)"
        />
      </svg>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'

const width = 600
const height = 180

const rawPoints = [
  { x: 0,   y: 90 },
  { x: 80,  y: 40 },
  { x: 160, y: 20 },
  { x: 240, y: 55 },
  { x: 320, y: 130 },
  { x: 400, y: 155 },
  { x: 460, y: 110 },
  { x: 520, y: 95 },
  { x: 600, y: 90 },
]

function catmullRomToBezier(pts) {
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

const linePath = computed(() => catmullRomToBezier(rawPoints))

const areaPath = computed(() => {
  const line = catmullRomToBezier(rawPoints)
  const last = rawPoints[rawPoints.length - 1]
  const first = rawPoints[0]
  return `${line} L ${last.x} ${height} L ${first.x} ${height} Z`
})
</script>

<style scoped>
.analytics-card {
  background: #111111;
  border-radius: 20px;
  padding: 24px 24px 0;
  width: 480px;
  overflow: hidden;
  font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Display', 'Segoe UI', sans-serif;
}

.card-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  margin-bottom: 28px;
}

.card-title {
  font-size: 22px;
  font-weight: 700;
  color: #ffffff;
  margin: 0 0 8px;
  letter-spacing: -0.3px;
}

.card-stats {
  display: flex;
  align-items: center;
  gap: 10px;
}

.visitors {
  font-size: 14px;
  color: #888888;
  font-weight: 400;
}

.badge {
  background: #e8e8e8;
  color: #111111;
  font-size: 12px;
  font-weight: 600;
  padding: 3px 10px;
  border-radius: 20px;
  letter-spacing: 0.1px;
}

.view-btn {
  background: #1e1e1e;
  color: #ffffff;
  border: 1.5px solid #2e2e2e;
  border-radius: 10px;
  font-size: 13px;
  font-weight: 500;
  padding: 9px 16px;
  cursor: pointer;
  white-space: nowrap;
  font-family: inherit;
  transition: background 0.15s ease, border-color 0.15s ease;
}

.view-btn:hover {
  background: #2a2a2a;
  border-color: #3e3e3e;
}

.view-btn:active {
  background: #333333;
}

.chart-wrapper {
  width: calc(100% + 48px);
  margin-left: -24px;
}

.chart-svg {
  display: block;
  width: 100%;
  height: 180px;
}
</style>