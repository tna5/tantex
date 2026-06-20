export default defineNuxtRouteMiddleware(async (to) => {
  if (to.path === '/login') return

  const authState = useState('_tantex_auth', () => null)

  // Already verified this session
  if (authState.value === true) return
  if (authState.value === false) return navigateTo('/login')

  try {
    const { auth_required } = await $fetch('/api/auth/status')
    if (!auth_required) {
      authState.value = true
      return
    }
    // Verify the cookie is accepted by a protected endpoint
    await $fetch('/api/indexes')
    authState.value = true
  } catch (e) {
    const status = e?.response?.status ?? e?.status ?? e?.statusCode
    if (status === 401) {
      authState.value = false
      return navigateTo('/login')
    }
    // Network error — don't block navigation
    authState.value = true
  }
})
