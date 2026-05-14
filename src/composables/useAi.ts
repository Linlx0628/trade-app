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

  async function analyze(messages: AiMessage[]): Promise<string | null> {
    const config = getConfig()
    if (!config) {
      error.value = '请先在设置页面配置 AI（需要填写 API Key）'
      return error.value
    }

    loading.value = true
    error.value = null
    result.value = ''

    try {
      const resp = await aiApi.chat({ config, messages })
      result.value = resp.content
      return resp.content
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e)
      error.value = msg
      return msg
    } finally {
      loading.value = false
    }
  }

  return { loading, result, error, analyze, getConfig }
}
