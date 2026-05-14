import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { tradePlanApi, calcApi } from '@/lib/tauri'
import type {
  TradePlan,
  CreateTradePlanDto,
  UpdateTradePlanDto,
  TradePlanStatus,
  CalculationResult,
} from '@/types/common'

export const useTradePlanStore = defineStore('trade-plan', () => {
  const plans = ref<TradePlan[]>([])
  const currentPlan = ref<TradePlan | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)
  const filterStatus = ref<TradePlanStatus | 'all'>('all')
  const calculation = ref<CalculationResult | null>(null)

  const filteredPlans = computed(() => {
    if (filterStatus.value === 'all') return plans.value
    return plans.value.filter((p) => p.status === filterStatus.value)
  })

  const statusCounts = computed(() => {
    const counts: Record<string, number> = { all: plans.value.length, planned: 0, executing: 0, completed: 0, cancelled: 0 }
    for (const p of plans.value) {
      counts[p.status] = (counts[p.status] || 0) + 1
    }
    return counts
  })

  async function fetchPlans(accountId: string) {
    loading.value = true
    error.value = null
    try {
      plans.value = await tradePlanApi.getAll(accountId)
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      loading.value = false
    }
  }

  async function createPlan(dto: CreateTradePlanDto) {
    error.value = null
    const plan = await tradePlanApi.create(dto)
    plans.value.unshift(plan)
    return plan
  }

  async function updatePlan(dto: UpdateTradePlanDto) {
    error.value = null
    const updated = await tradePlanApi.update(dto)
    const idx = plans.value.findIndex((p) => p.id === updated.id)
    if (idx !== -1) plans.value[idx] = updated
    if (currentPlan.value?.id === updated.id) currentPlan.value = updated
    return updated
  }

  async function deletePlan(id: string) {
    error.value = null
    await tradePlanApi.delete(id)
    plans.value = plans.value.filter((p) => p.id !== id)
    if (currentPlan.value?.id === id) currentPlan.value = null
  }

  async function calculate(accountId: string, entryPrice: number, stopLoss: number, takeProfit: number) {
    calculation.value = await calcApi.calculate({
      accountId,
      entryPrice,
      stopLoss,
      takeProfit,
    })
    return calculation.value
  }

  function clearCalculation() {
    calculation.value = null
  }

  return {
    plans,
    currentPlan,
    loading,
    error,
    filterStatus,
    filteredPlans,
    statusCounts,
    calculation,
    fetchPlans,
    createPlan,
    updatePlan,
    deletePlan,
    calculate,
    clearCalculation,
  }
})
