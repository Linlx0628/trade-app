import { defineStore } from 'pinia'
import { ref } from 'vue'
import { tradeTemplateApi } from '@/lib/tauri'
import type { TradeTemplate, CreateTradeTemplateDto, UpdateTradeTemplateDto } from '@/types/common'

export const useTradeTemplateStore = defineStore('trade-template', () => {
  const templates = ref<TradeTemplate[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchTemplates(accountId: string) {
    loading.value = true
    error.value = null
    try {
      templates.value = await tradeTemplateApi.getAll(accountId)
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      loading.value = false
    }
  }

  async function createTemplate(dto: CreateTradeTemplateDto) {
    const t = await tradeTemplateApi.create(dto)
    templates.value.unshift(t)
    return t
  }

  async function updateTemplate(dto: UpdateTradeTemplateDto) {
    const t = await tradeTemplateApi.update(dto)
    const idx = templates.value.findIndex(x => x.id === t.id)
    if (idx >= 0) templates.value[idx] = t
    return t
  }

  async function deleteTemplate(id: string) {
    await tradeTemplateApi.delete(id)
    templates.value = templates.value.filter(x => x.id !== id)
  }

  async function togglePin(id: string) {
    const t = templates.value.find(x => x.id === id)
    if (!t) return
    const updated = await tradeTemplateApi.update({ id, is_pinned: !t.is_pinned })
    const idx = templates.value.findIndex(x => x.id === id)
    if (idx >= 0) templates.value[idx] = updated
    return updated
  }

  return { templates, loading, error, fetchTemplates, createTemplate, updateTemplate, deleteTemplate, togglePin }
})
