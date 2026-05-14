import { defineStore } from 'pinia'
import { ref } from 'vue'
import { dashboardApi } from '@/lib/tauri'
import type { DashboardStats, PnlTrend, SymbolPnl } from '@/lib/tauri'

const CACHE_TTL = 5 * 60 * 1000 // 5 minutes

export const useDashboardStore = defineStore('dashboard', () => {
  const stats = ref<DashboardStats | null>(null)
  const pnlTrend = ref<PnlTrend[]>([])
  const symbolPnl = ref<SymbolPnl[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  let lastFetchTime = 0
  let lastAccountId = ''

  function isCacheValid(accountId: string): boolean {
    return accountId === lastAccountId && Date.now() - lastFetchTime < CACHE_TTL
  }

  async function fetchAll(accountId: string, force = false) {
    if (!force && isCacheValid(accountId)) return

    loading.value = true
    error.value = null
    try {
      const [s, t, p] = await Promise.all([
        dashboardApi.getStats(accountId),
        dashboardApi.getPnlTrend(accountId, 30),
        dashboardApi.getSymbolPnl(accountId),
      ])
      stats.value = s
      pnlTrend.value = t
      symbolPnl.value = p
      lastFetchTime = Date.now()
      lastAccountId = accountId
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      loading.value = false
    }
  }

  function invalidateCache() {
    lastFetchTime = 0
    lastAccountId = ''
  }

  return { stats, pnlTrend, symbolPnl, loading, error, fetchAll, invalidateCache }
})
