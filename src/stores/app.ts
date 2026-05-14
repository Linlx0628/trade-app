import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useAppStore = defineStore('app', () => {
  const sidebarCollapsed = ref(false)
  const sidebarWidth = ref(240)
  const isOnline = ref(true)

  function toggleSidebar() {
    sidebarCollapsed.value = !sidebarCollapsed.value
    sidebarWidth.value = sidebarCollapsed.value ? 64 : 240
  }

  function setSidebarCollapsed(collapsed: boolean) {
    sidebarCollapsed.value = collapsed
    sidebarWidth.value = collapsed ? 64 : 240
  }

  function setOnlineStatus(status: boolean) {
    isOnline.value = status
  }

  return {
    sidebarCollapsed,
    sidebarWidth,
    isOnline,
    toggleSidebar,
    setSidebarCollapsed,
    setOnlineStatus,
  }
})
