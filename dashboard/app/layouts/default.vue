<template>
  <SidebarProvider>
    <Sidebar collapsible="icon">
      <SidebarHeader class="px-3 py-3">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-2 group-data-[collapsible=icon]:hidden">
            <div class="size-7 rounded-lg bg-primary flex items-center justify-center shrink-0">
              <Icon icon="majesticons:search" class="size-4 text-primary-foreground" />
            </div>
            <span class="text-sm font-bold tracking-tight">tantex</span>
          </div>
          <SidebarTrigger class="-mr-1" />
        </div>
      </SidebarHeader>

      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupContent>
            <SidebarMenu class="space-y-0.5">
              <SidebarMenuItem>
                <SidebarMenuButton
                  :is-active="route.path === '/'"
                  tooltip="Dashboard"
                  as-child
                >
                  <NuxtLink to="/">
                    <Icon icon="majesticons:chart-bar" />
                    <span class="group-data-[collapsible=icon]:hidden">Dashboard</span>
                  </NuxtLink>
                </SidebarMenuButton>
              </SidebarMenuItem>
              <SidebarMenuItem>
                <SidebarMenuButton
                  :is-active="route.path.startsWith('/indexes')"
                  tooltip="Indexes"
                  as-child
                >
                  <NuxtLink to="/indexes">
                    <Icon icon="majesticons:database" />
                    <span class="group-data-[collapsible=icon]:hidden">Indexes</span>
                  </NuxtLink>
                </SidebarMenuButton>
              </SidebarMenuItem>
              <SidebarMenuItem>
                <SidebarMenuButton
                  :is-active="route.path === '/search'"
                  tooltip="Search"
                  as-child
                >
                  <NuxtLink to="/search">
                    <Icon icon="majesticons:search" />
                    <span class="group-data-[collapsible=icon]:hidden">Search</span>
                  </NuxtLink>
                </SidebarMenuButton>
              </SidebarMenuItem>
              <SidebarMenuItem>
                <SidebarMenuButton
                  :is-active="route.path === '/metrics'"
                  tooltip="Metrics"
                  as-child
                >
                  <NuxtLink to="/metrics">
                    <Icon icon="majesticons:pulse" />
                    <span class="group-data-[collapsible=icon]:hidden">Metrics</span>
                  </NuxtLink>
                </SidebarMenuButton>
              </SidebarMenuItem>
              <SidebarMenuItem>
                <SidebarMenuButton
                  :is-active="route.path === '/settings'"
                  tooltip="Settings"
                  as-child
                >
                  <NuxtLink to="/settings">
                    <Icon icon="majesticons:settings-cog" />
                    <span class="group-data-[collapsible=icon]:hidden">Settings</span>
                  </NuxtLink>
                </SidebarMenuButton>
              </SidebarMenuItem>
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>

      <div class="h-px bg-sidebar-border mx-3 opacity-60" />

      <SidebarFooter>
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton
              :is-active="route.path === '/docs'"
              tooltip="Documentation"
              as-child
            >
              <NuxtLink to="/docs">
                <Icon icon="majesticons:book-open" />
                <span class="group-data-[collapsible=icon]:hidden">Documentation</span>
              </NuxtLink>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
        <div class="flex items-center justify-between px-2 py-2">
          <div :class="[
            'flex items-center gap-1.5 px-2 py-1 rounded-full text-xs transition-colors group-data-[collapsible=icon]:hidden',
            serverOnline
              ? 'bg-green-100 dark:bg-green-950/40 text-green-700 dark:text-green-400'
              : 'bg-red-100 dark:bg-red-950/40 text-red-600 dark:text-red-400'
          ]">
            <div :class="['size-1.5 rounded-full', serverOnline ? 'bg-green-500' : 'bg-red-500']" />
            {{ serverOnline ? 'Online' : 'Offline' }}
          </div>
          <div :class="['size-2 rounded-full hidden group-data-[collapsible=icon]:block', serverOnline ? 'bg-green-500' : 'bg-red-500']" />
          <div class="flex items-center gap-1">
            <Button v-if="authRequired" variant="ghost" size="icon-sm" class="text-muted-foreground hover:text-foreground" title="Logout" @click="handleLogout">
              <Icon icon="majesticons:logout" class="size-4" />
            </Button>
            <Button variant="ghost" size="icon-sm" class="text-muted-foreground hover:text-foreground" @click="toggleDark">
              <Icon v-if="isDark" icon="majesticons:sun" class="size-4" />
              <Icon v-else icon="majesticons:moon" class="size-4" />
            </Button>
          </div>
        </div>
      </SidebarFooter>

    </Sidebar>

    <SidebarInset>
      <div class="flex flex-row flex-1 min-h-0">
        <div class="flex flex-col flex-1 min-w-0 mx-auto max-w-7xl w-full">
          <header class="flex h-12 shrink-0 items-center gap-2 px-6">
            <template v-if="breadcrumb.length">
              <nav class="flex items-center gap-1.5 text-sm text-muted-foreground">
                <template v-for="(item, i) in breadcrumb" :key="i">
                  <Icon v-if="i > 0" icon="majesticons:chevron-right" class="size-3.5" />
                  <NuxtLink
                    v-if="item.href"
                    :to="item.href"
                    class="hover:text-foreground transition-colors"
                  >{{ item.label }}</NuxtLink>
                  <span v-else class="text-foreground font-semibold">{{ item.label }}</span>
                </template>
              </nav>
            </template>
            <div id="header-actions" class="ml-auto flex items-center gap-2" />
          </header>
          <div class="flex-1 overflow-y-auto">
            <slot />
          </div>
        </div>
        <aside
          class="w-72 shrink-0 overflow-y-auto p-2"
          :class="{ hidden: !sidebarVisible }"
        >
          <div
            id="sidebar-panel"
            class="bg-sidebar text-sidebar-foreground rounded-lg ring-1 ring-sidebar-border h-full p-4"
          />
        </aside>
      </div>
    </SidebarInset>
  </SidebarProvider>
</template>

<script setup>
import { Icon } from '@iconify/vue'

const route = useRoute()
const router = useRouter()
const { breadcrumb } = useBreadcrumb()
const serverOnline = ref(true)
const sidebarVisible = useState('sidebar-visible', () => false)
const { lastEvent } = useStreamEvents()
const { push: pushHistory } = useMetricsHistory()
const authState = useState('_tantex_auth', () => null)
const authRequired = ref(false)

let es = null
const isDark = ref(false)

onMounted(async () => {
  const stored = localStorage.getItem('dark-mode')
  if (stored === 'true') {
    document.documentElement.classList.add('dark')
    isDark.value = true
  } else if (stored === 'false') {
    document.documentElement.classList.remove('dark')
    isDark.value = false
  } else {
    isDark.value = document.documentElement.classList.contains('dark')
  }

  try {
    const status = await $fetch('/api/auth/status')
    authRequired.value = status.auth_required ?? false
  } catch {}

  es = new EventSource('/api/metrics/stream')
  es.onmessage = (e) => {
    try {
      const data = JSON.parse(e.data)
      if (data.type === 'metrics') serverOnline.value = data.status === 'online'
      lastEvent.value = data
      pushHistory(data)
    } catch {}
  }
  es.onerror = () => { serverOnline.value = false }
})

onUnmounted(() => es?.close())

function toggleDark() {
  isDark.value = !isDark.value
  document.documentElement.classList.toggle('dark', isDark.value)
  localStorage.setItem('dark-mode', String(isDark.value))
}

async function handleLogout() {
  await $fetch('/api/auth/logout', { method: 'POST' }).catch(() => {})
  authState.value = null
  router.push('/login')
}
</script>
