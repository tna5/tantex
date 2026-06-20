export function useBreadcrumb() {
  const breadcrumb = useState('breadcrumb', () => [])

  function setBreadcrumb(items) {
    breadcrumb.value = items
  }

  return { breadcrumb, setBreadcrumb }
}
