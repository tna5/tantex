export function useStreamEvents() {
  const lastEvent = useState('stream-last-event', () => null)
  return { lastEvent }
}
