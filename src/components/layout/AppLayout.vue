<script setup lang="ts">
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import {
  LayoutDashboard,
  FileText,
  BookOpen,
  StickyNote,
  Settings,
  PanelLeftClose,
  PanelLeft,
  TrendingUp,
} from 'lucide-vue-next'
import { useAppStore } from '@/stores/app'
import { Separator } from '@/components/ui/separator'
import { cn } from '@/lib/utils'

const route = useRoute()
const router = useRouter()
const appStore = useAppStore()

const pageTitle = computed(() => {
  return (route.meta.title as string) || '交易管理系统'
})

const navItems = [
  { name: '仪表盘', path: '/', icon: LayoutDashboard },
  { name: '交易计划', path: '/trade-plan', icon: FileText },
  { name: '交易日志', path: '/trade-log', icon: BookOpen },
  { name: '交易总结', path: '/trade-summary', icon: StickyNote },
]

const bottomNavItems = [
  { name: '设置', path: '/settings', icon: Settings },
]

function isActive(path: string): boolean {
  if (path === '/') return route.path === '/'
  return route.path.startsWith(path)
}

function navigateTo(path: string) {
  router.push(path)
}
</script>

<template>
  <div class="flex h-screen w-screen overflow-hidden bg-background">
    <!-- Sidebar -->
    <aside
      :class="cn(
        'flex flex-col h-full bg-sidebar border-r border-sidebar-border transition-all duration-300 ease-in-out',
        appStore.sidebarCollapsed ? 'w-16' : 'w-60',
      )"
    >
      <!-- Logo / Brand -->
      <div class="flex items-center h-14 px-4 border-b border-sidebar-border">
        <div class="flex items-center gap-3 overflow-hidden">
          <div class="flex items-center justify-center w-8 h-8 rounded-lg bg-primary/10 shrink-0">
            <TrendingUp class="w-4 h-4 text-primary" />
          </div>
          <Transition
            enter-active-class="transition-opacity duration-200"
            enter-from-class="opacity-0"
            enter-to-class="opacity-100"
            leave-active-class="transition-opacity duration-200"
            leave-from-class="opacity-100"
            leave-to="opacity-0"
          >
            <span
              v-if="!appStore.sidebarCollapsed"
              class="text-base font-semibold text-sidebar-foreground whitespace-nowrap tracking-tight"
            >
              TradePro
            </span>
          </Transition>
        </div>
      </div>

      <!-- Navigation -->
      <nav class="flex-1 flex flex-col py-3 px-2 overflow-y-auto">
        <div class="flex flex-col gap-0.5">
          <button
            v-for="item in navItems"
            :key="item.path"
            :class="cn(
              'flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-all duration-150 cursor-pointer',
              isActive(item.path)
                ? 'bg-primary/10 text-primary'
                : 'text-sidebar-foreground/60 hover:bg-sidebar-accent hover:text-sidebar-foreground',
              appStore.sidebarCollapsed && 'justify-center px-0',
            )"
            :title="appStore.sidebarCollapsed ? item.name : undefined"
            @click="navigateTo(item.path)"
          >
            <component
              :is="item.icon"
              :class="cn('shrink-0', isActive(item.path) ? 'w-[18px] h-[18px]' : 'w-4 h-4')"
            />
            <Transition
              enter-active-class="transition-opacity duration-200"
              enter-from-class="opacity-0"
              enter-to-class="opacity-100"
              leave-active-class="transition-opacity duration-200"
              leave-from-class="opacity-100"
              leave-to-class="opacity-0"
            >
              <span v-if="!appStore.sidebarCollapsed" class="truncate">
                {{ item.name }}
              </span>
            </Transition>
          </button>
        </div>

        <!-- Spacer -->
        <div class="flex-1" />

        <!-- Bottom Navigation -->
        <div class="flex flex-col gap-0.5">
          <Separator class="mb-2" />
          <button
            v-for="item in bottomNavItems"
            :key="item.path"
            :class="cn(
              'flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-all duration-150 cursor-pointer',
              isActive(item.path)
                ? 'bg-primary/10 text-primary'
                : 'text-sidebar-foreground/60 hover:bg-sidebar-accent hover:text-sidebar-foreground',
              appStore.sidebarCollapsed && 'justify-center px-0',
            )"
            :title="appStore.sidebarCollapsed ? item.name : undefined"
            @click="navigateTo(item.path)"
          >
            <component
              :is="item.icon"
              :class="cn('shrink-0', isActive(item.path) ? 'w-[18px] h-[18px]' : 'w-4 h-4')"
            />
            <Transition
              enter-active-class="transition-opacity duration-200"
              enter-from-class="opacity-0"
              enter-to-class="opacity-100"
              leave-active-class="transition-opacity duration-200"
              leave-from-class="opacity-100"
              leave-to-class="opacity-0"
            >
              <span v-if="!appStore.sidebarCollapsed" class="truncate">
                {{ item.name }}
              </span>
            </Transition>
          </button>
        </div>
      </nav>
    </aside>

    <!-- Main Content Area -->
    <div class="flex flex-col flex-1 overflow-hidden">
      <!-- Header -->
      <header class="flex items-center justify-between h-14 px-6 border-b border-border bg-card">
        <div class="flex items-center gap-4">
          <button
            class="inline-flex items-center justify-center rounded-md w-8 h-8 text-muted-foreground hover:text-foreground hover:bg-accent transition-colors cursor-pointer"
            @click="appStore.toggleSidebar()"
          >
            <PanelLeftClose v-if="!appStore.sidebarCollapsed" class="w-4 h-4" />
            <PanelLeft v-else class="w-4 h-4" />
          </button>
          <div>
            <h1 class="text-base font-semibold text-foreground tracking-tight">
              {{ pageTitle }}
            </h1>
          </div>
        </div>
        <div class="flex items-center gap-3">
          <slot name="header-actions" />
        </div>
      </header>

      <!-- Page Content -->
      <main class="flex-1 overflow-y-auto p-6">
        <slot />
      </main>
    </div>
  </div>
</template>
