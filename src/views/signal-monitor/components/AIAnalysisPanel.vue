<script setup lang="ts">
import { ref } from 'vue'
import { Brain, Loader2, Sparkles } from 'lucide-vue-next'
import { Button } from '@/components/ui/button'
import { invoke } from '@tauri-apps/api/core'
import { renderMarkdown } from '@/lib/markdown'
import type { AiConfig, AiMessage, AiChatResponse } from '@/types/common'
import type { KlineData } from '@/stores/market'

const props = defineProps<{
  symbol: string
  symbolName: string
  klineData: KlineData[]
  period: string
}>()

const periodLabels: Record<string, string> = {
  '5m': '5分钟', '15m': '15分钟', '30m': '30分钟',
  '60m': '60分钟', day: '日线', week: '周线', month: '月线',
}

const analyzing = ref(false)
const result = ref('')
const error = ref('')

function buildPrompt(): string {
  const periodLabel = periodLabels[props.period] || props.period
  const recentBars = props.klineData.slice(-60)

  const dataTable = recentBars.map((k) => {
    const ts = k.timestamp.length > 10 ? k.timestamp.slice(0, 16) : k.timestamp
    const dir = k.close >= k.open ? '↑' : '↓'
    return `${ts} | ${k.open.toFixed(2)} | ${k.high.toFixed(2)} | ${k.low.toFixed(2)} | ${k.close.toFixed(2)} | ${k.volume.toFixed(0)} | ${dir}`
  }).join('\n')

  const latestClose = recentBars.length > 0 ? recentBars[recentBars.length - 1].close.toFixed(2) : 'N/A'

  return `你是一位专业的股票/期货技术分析师。请根据以下K线数据进行分析，给出交易建议。

品种: ${props.symbolName} (${props.symbol})
周期: ${periodLabel}
最新收盘价: ${latestClose}
数据条数: ${recentBars.length}

K线数据（时间 | 开 | 高 | 低 | 收 | 成交量 | 方向）：
${dataTable}

请从以下几个方面分析：
1. **趋势判断**：当前处于上涨趋势、下跌趋势还是横盘整理？说出判断依据。
2. **关键价位**：指出重要的支撑位和压力位。
3. **技术形态**：是否存在典型K线形态（如头肩顶、双底、三角形整理等）。
4. **交易建议**：给出具体的入场点、止损点和止盈点建议。
5. **风险提示**：需要注意的风险因素。

请用中文回答，简洁明了，不要过度解读。`
}

async function runAnalysis() {
  analyzing.value = true
  error.value = ''
  result.value = ''

  try {
    const raw = localStorage.getItem('ai_config')
    if (!raw) { error.value = '请先在设置中配置AI（API Key、模型等）'; return }

    const config: AiConfig = JSON.parse(raw)
    if (!config.api_key) { error.value = '请先在设置中配置AI API Key'; return }

    const prompt = buildPrompt()
    const messages: AiMessage[] = [{ role: 'user', content: prompt }]

    const resp = await invoke<AiChatResponse>('ai_chat', {
      req: { config, messages, temperature: 0.7, max_tokens: 2000 },
    })
    result.value = resp.content
  } catch (e) {
    error.value = String(e)
  } finally {
    analyzing.value = false
  }
}
</script>

<template>
  <div class="space-y-4">
    <div class="flex items-center gap-3">
      <div class="flex items-center justify-center w-8 h-8 rounded-lg bg-primary/10">
        <Brain class="w-4 h-4 text-primary" />
      </div>
      <span class="text-sm font-medium text-foreground">AI 智能分析</span>
      <div class="flex-1" />
      <Button size="sm" class="gap-2" :disabled="analyzing || klineData.length === 0" @click="runAnalysis">
        <Loader2 v-if="analyzing" class="w-4 h-4 animate-spin" />
        <Sparkles v-else class="w-4 h-4" />
        AI分析
      </Button>
    </div>

    <div v-if="!result && !error && !analyzing" class="text-center py-8 text-sm text-muted-foreground">
      <Brain class="w-10 h-10 mx-auto mb-3 opacity-30" />
      <p>点击「AI分析」让AI对当前K线数据进行分析</p>
    </div>

    <div v-if="analyzing" class="flex flex-col items-center py-12 text-muted-foreground gap-3">
      <Loader2 class="w-8 h-8 animate-spin text-primary" />
      <p class="text-sm">AI正在分析K线数据...</p>
    </div>

    <div v-if="error" class="p-4 rounded-lg border border-loss/30 bg-loss/5 text-sm text-loss">
      {{ error }}
    </div>

    <div v-if="result" class="p-4 rounded-lg border border-border/50 bg-card">
      <div class="prose prose-sm dark:prose-invert max-w-none text-sm text-foreground" v-html="renderMarkdown(result)" />
    </div>
  </div>
</template>