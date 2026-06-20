const MAX_POINTS = 60

function makeSeries() {
  return { labels: [], data: [] }
}

function appendPoint(series, value) {
  const label = new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })
  series.labels.push(label)
  series.data.push(value)
  if (series.labels.length > MAX_POINTS) {
    series.labels.shift()
    series.data.shift()
  }
}

export function useMetricsHistory() {
  const ingestRate = useState('hist-ingest', makeSeries)
  const searchRate = useState('hist-search', makeSeries)
  const totalSegments = useState('hist-segments', makeSeries)
  const totalDocs = useState('hist-docs', makeSeries)
  const pendingDocs = useState('hist-pending', makeSeries)
  const ramUsed = useState('hist-ram', makeSeries)

  function push(event) {
    if (event.type !== 'metrics' || event.status !== 'online') return
    appendPoint(ingestRate.value, event.ingestRate ?? 0)
    appendPoint(totalSegments.value, event.totalSegments ?? 0)
    appendPoint(totalDocs.value, event.totalDocs ?? 0)
    appendPoint(pendingDocs.value, event.totalPendingDocs ?? 0)
    appendPoint(ramUsed.value, event.ramUsedMb ?? 0)

    const idxs = event.indexes ?? []
    const totalSearchRate = idxs.reduce((s, i) => s + (i.search_rate ?? 0), 0)
    appendPoint(searchRate.value, Math.round(totalSearchRate * 10) / 10)
  }

  return { ingestRate, searchRate, totalSegments, totalDocs, pendingDocs, ramUsed, push }
}
