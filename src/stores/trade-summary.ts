import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { tradeSummaryApi } from '@/lib/tauri'
import type { TradeSummary, CreateTradeSummaryDto, UpdateTradeSummaryDto, SummaryType } from '@/types/common'

export const useTradeSummaryStore = defineStore('trade-summary', () => {
  const summaries = ref<TradeSummary[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)
  const filterType = ref<SummaryType | 'all'>('all')

  const filteredSummaries = computed(() => {
    if (filterType.value === 'all') return summaries.value
    return summaries.value.filter((s) => s.summary_type === filterType.value)
  })

  async function fetchSummaries(accountId: string) {
    loading.value = true
    error.value = null
    try {
      summaries.value = await tradeSummaryApi.getAll(accountId)
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      loading.value = false
    }
  }

  async function createSummary(dto: CreateTradeSummaryDto) {
    error.value = null
    const summary = await tradeSummaryApi.create(dto)
    summaries.value.unshift(summary)
    return summary
  }

  async function updateSummary(dto: UpdateTradeSummaryDto) {
    error.value = null
    const updated = await tradeSummaryApi.update(dto)
    const idx = summaries.value.findIndex((s) => s.id === updated.id)
    if (idx !== -1) summaries.value[idx] = updated
    return updated
  }

  async function deleteSummary(id: string) {
    error.value = null
    await tradeSummaryApi.delete(id)
    summaries.value = summaries.value.filter((s) => s.id !== id)
  }

  return { summaries, loading, error, filterType, filteredSummaries, fetchSummaries, createSummary, updateSummary, deleteSummary }
})
