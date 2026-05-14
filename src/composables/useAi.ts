import { ref } from 'vue'
import { aiApi } from '@/lib/tauri'
import type { AiConfig, AiMessage } from '@/types/common'

export function useAi() {
  const loading = ref(false)
  const result = ref('')
  const error = ref<string | null>(null)

  function getConfig(): AiConfig | null {
    const raw = localStorage.getItem('ai_config')
    if (!raw) return null
    try {
      const cfg = JSON.parse(raw) as AiConfig
      if (!cfg.api_key) return null
      return cfg
    } catch {
      return null
    }
  }

  async function analyze(messages: AiMessage[]) {
    const config = getConfig()
    if (!config) {
      error.value = '请先在设置页面配置 AI'
      return null
    }

    loading.value = true
    error.value = null
    result.value = ''

    try {
      const resp = await aiApi.chat({ config, messages })
      result.value = resp.content
      return resp.content
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : String(e)
      return null
    } finally {
      loading.value = false
    }
  }

  return { loading, result, error, analyze, getConfig }
}
