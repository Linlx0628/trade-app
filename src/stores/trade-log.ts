import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { tradeLogApi } from '@/lib/tauri'
import type { TradeLog, CreateTradeLogDto, UpdateTradeLogDto, TradeLogStatus } from '@/types/common'

export const useTradeLogStore = defineStore('trade-log', () => {
  const logs = ref<TradeLog[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)
  const filterStatus = ref<TradeLogStatus | 'all'>('all')

  const filteredLogs = computed(() => {
    if (filterStatus.value === 'all') return logs.value
    return logs.value.filter((l) => l.status === filterStatus.value)
  })

  const statusCounts = computed(() => {
    const counts: Record<string, number> = { all: logs.value.length, open: 0, closed: 0 }
    for (const l of logs.value) counts[l.status] = (counts[l.status] || 0) + 1
    return counts
  })

  const totalPnl = computed(() => logs.value.reduce((sum, l) => sum + l.pnl, 0))

  async function fetchLogs(accountId: string) {
    loading.value = true
    error.value = null
    try {
      logs.value = await tradeLogApi.getAll(accountId)
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      loading.value = false
    }
  }

  async function createLog(dto: CreateTradeLogDto) {
    error.value = null
    const log = await tradeLogApi.create(dto)
    logs.value.unshift(log)
    return log
  }

  async function updateLog(dto: UpdateTradeLogDto) {
    error.value = null
    const updated = await tradeLogApi.update(dto)
    const idx = logs.value.findIndex((l) => l.id === updated.id)
    if (idx !== -1) logs.value[idx] = updated
    return updated
  }

  async function deleteLog(id: string) {
    error.value = null
    await tradeLogApi.delete(id)
    logs.value = logs.value.filter((l) => l.id !== id)
  }

  return { logs, loading, error, filterStatus, filteredLogs, statusCounts, totalPnl, fetchLogs, createLog, updateLog, deleteLog }
})
