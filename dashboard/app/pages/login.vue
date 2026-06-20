<template>
  <div class="min-h-screen flex items-center justify-center bg-background p-4">
    <Card class="w-full max-w-sm">
      <CardHeader class="text-center pb-4">
        <div class="flex items-center justify-center gap-2 mb-2">
          <div class="size-8 rounded-lg bg-primary flex items-center justify-center">
            <Icon icon="majesticons:search" class="size-5 text-primary-foreground" />
          </div>
          <span class="text-lg font-bold tracking-tight">tantex</span>
        </div>
        <CardTitle class="text-base">API key required</CardTitle>
        <CardDescription>Enter your API key to access the dashboard.</CardDescription>
      </CardHeader>
      <CardContent>
        <form class="space-y-3" @submit.prevent="handleLogin">
          <div v-if="error" class="rounded-md border border-destructive/40 bg-destructive/10 px-3 py-2 text-sm text-destructive">
            {{ error }}
          </div>
          <Input
            v-model="key"
            type="password"
            placeholder="API key"
            autocomplete="current-password"
            :disabled="loading"
          />
          <Button type="submit" class="w-full" :disabled="loading || !key">
            {{ loading ? 'Connecting…' : 'Connect' }}
          </Button>
        </form>
      </CardContent>
    </Card>
  </div>
</template>

<script setup>
import { Icon } from '@iconify/vue'

definePageMeta({ layout: false })
useHead({ title: 'Login — tantex' })

const router = useRouter()
const authState = useState('_tantex_auth', () => null)

const key = ref('')
const loading = ref(false)
const error = ref('')

async function handleLogin() {
  if (!key.value) return
  loading.value = true
  error.value = ''
  try {
    await $fetch('/api/auth/login', { method: 'POST', body: { key: key.value } })
    authState.value = true
    router.push('/')
  } catch (e) {
    const status = e?.response?.status ?? e?.status ?? e?.statusCode
    error.value = status === 401 ? 'Invalid API key.' : 'Connection failed. Is the server running?'
  } finally {
    loading.value = false
  }
}
</script>
