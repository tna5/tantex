export function useTantex() {
  const listIndexes = () => $fetch('/api/indexes')
  const getIndex = (name) => $fetch(`/api/indexes/${name}`)
  const createIndex = (name, schema) =>
    $fetch('/api/indexes', { method: 'POST', body: { name, schema } })
  const deleteIndex = (name) =>
    $fetch(`/api/indexes/${name}`, { method: 'DELETE' })
  const commitIndex = (name) =>
    $fetch(`/api/indexes/${name}/commit`, { method: 'POST' })
  const searchIndex = (name, query, limit = 10, offset = 0) =>
    $fetch(`/api/indexes/${name}/search`, { method: 'POST', body: { query, limit, offset } })
  const getSegments = (name) => $fetch(`/api/indexes/${name}/segments`)
  const getConfig = () => $fetch('/api/config')
  const setConfig = (patch) =>
    $fetch('/api/config', { method: 'POST', body: patch })
  const deleteByQuery = (name, query) =>
    $fetch(`/api/indexes/${name}/delete`, { method: 'POST', body: { query } })
  const setIndexSettings = (name, patch) =>
    $fetch(`/api/indexes/${name}/settings`, { method: 'POST', body: patch })

  return { listIndexes, getIndex, createIndex, deleteIndex, commitIndex, searchIndex, getSegments, getConfig, setConfig, deleteByQuery, setIndexSettings }
}

export function useMetricsStream() {
  const status = ref('connecting')
  const totalDocs = ref(0)
  const totalIndexes = ref(0)
  const totalSegments = ref(0)
  const ingestRate = ref(0)
  const totalPendingDocs = ref(0)
  const totalMergesInProgress = ref(0)
  const ramUsedMb = ref(0)
  const ramTotalMb = ref(0)
  const indexes = ref([])

  let es = null

  const connect = () => {
    es = new EventSource('/api/metrics/stream')
    es.onmessage = (e) => {
      try {
        const data = JSON.parse(e.data)
        if (data.type !== 'metrics') return
        status.value = data.status
        if (data.status === 'online') {
          totalDocs.value = data.totalDocs ?? 0
          totalIndexes.value = data.totalIndexes ?? 0
          totalSegments.value = data.totalSegments ?? 0
          ingestRate.value = data.ingestRate ?? 0
          totalPendingDocs.value = data.totalPendingDocs ?? 0
          totalMergesInProgress.value = data.totalMergesInProgress ?? 0
          ramUsedMb.value = data.ramUsedMb ?? 0
          ramTotalMb.value = data.ramTotalMb ?? 0
          indexes.value = data.indexes ?? []
        }
      } catch {}
    }
    es.onerror = () => { status.value = 'offline' }
  }

  onMounted(connect)
  onUnmounted(() => es?.close())

  return { status, totalDocs, totalIndexes, totalSegments, ingestRate, totalPendingDocs, totalMergesInProgress, ramUsedMb, ramTotalMb, indexes }
}
