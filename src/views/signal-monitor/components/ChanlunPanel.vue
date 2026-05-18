<script setup lang="ts">
import { ref } from 'vue'
import { GitBranch, Loader2, TrendingUp, TrendingDown, Minus, Bell } from 'lucide-vue-next'
import { Button } from '@/components/ui/button'
import { SelectNative } from '@/components/ui/select'
import { Badge } from '@/components/ui/badge'
import { invoke } from '@tauri-apps/api/core'
import { useAccountStore } from '@/stores/account'
import { useToast } from '@/components/ui/toast'

interface Fractal { index: number; fractal_type: string; value: number }
interface Bi { start_index: number; end_index: number; direction: string; start_value: number; end_value: number }
interface Pivot { start_index: number; end_index: number; zg: number; zd: number; zz: number }
interface ChanlunSignal { signal_type: string; index: number; price: number; description: string }
interface ChanlunAnalysis {
  symbol: string
  fractals: Fractal[]
  bis: Bi[]
  pivots: Pivot[]
  signals: ChanlunSignal[]
  current_trend: string
}

const props = defineProps<{
  symbol: string
}>()

const analysis = ref<ChanlunAnalysis | null>(null)
const loading = ref(false)
const period = ref('day')

async function runAnalysis() {
  loading.value = true
  try {
    analysis.value = await invoke<ChanlunAnalysis>('analyze_chanlun', {
      symbol: props.symbol,
      period: period.value,
    })
  } catch (e) {
    console.error('Chanlun analysis failed:', e)
  } finally {
    loading.value = false
  }
}

const signalLabelMap: Record<string, string> = {
  buy1: '一买', sell1: '一卖',
  buy2: '二买', sell2: '二卖',
  buy3: '三买', sell3: '三卖',
}

const accountStore = useAccountStore()
const { toast } = useToast()
const alertCreating = ref<string | null>(null)

async function createAlert(signal: ChanlunSignal) {
  const acc = accountStore.currentAccount
  if (!acc) { toast({ title: '请先选择账户', variant: 'destructive' }); return }
  alertCreating.value = signal.signal_type
  try {
    await invoke('create_signal_alert', {
      dto: {
        account_id: acc.id,
        symbol: props.symbol,
        alert_type: signal.signal_type,
        condition_value: signal.price,
        description: signal.description,
      },
    })
    toast({ title: '已添加信号提醒', description: signal.description, variant: 'success' })
  } catch (e) {
    toast({ title: '添加失败', description: String(e), variant: 'destructive' })
  } finally {
    alertCreating.value = null
  }
}

const trendIcon = {
  up: TrendingUp,
  down: TrendingDown,
  unknown: Minus,
}
</script>

<template>
  <div class="space-y-4">
    <div class="flex items-center gap-3">
      <div class="flex items-center justify-center w-8 h-8 rounded-lg bg-primary/10">
        <GitBranch class="w-4 h-4 text-primary" />
      </div>
      <span class="text-sm font-medium text-foreground">缠论分析</span>
      <div class="flex-1" />
      <SelectNative :model-value="period" @update:model-value="period = $event" class="w-24 text-xs">
        <option value="day">日线</option>
        <option value="60m">60分钟</option>
        <option value="30m">30分钟</option>
        <option value="5m">5分钟</option>
      </SelectNative>
      <Button size="sm" class="gap-2" :disabled="loading || !symbol" @click="runAnalysis">
        <Loader2 v-if="loading" class="w-4 h-4 animate-spin" />
        分析
      </Button>
    </div>

    <div v-if="!analysis && !loading" class="text-center py-8 text-sm text-muted-foreground">
      点击「分析」按钮开始缠论分析
    </div>

    <div v-if="analysis" class="space-y-3">
      <!-- Trend -->
      <div class="flex items-center gap-2 text-sm">
        <span class="text-muted-foreground">当前趋势：</span>
        <Badge :variant="analysis.current_trend === 'up' ? 'default' : analysis.current_trend === 'down' ? 'destructive' : 'secondary'">
          <component :is="(trendIcon as Record<string, typeof TrendingUp>)[analysis.current_trend] || Minus" class="w-3 h-3 mr-1" />
          {{ analysis.current_trend === 'up' ? '上涨' : analysis.current_trend === 'down' ? '下跌' : '未知' }}
        </Badge>
      </div>

      <!-- Summary -->
      <div class="grid grid-cols-3 gap-3 text-xs">
        <div class="rounded-lg bg-secondary/50 p-2.5 text-center">
          <p class="text-muted-foreground">分型数</p>
          <p class="text-lg font-semibold text-foreground mt-0.5">{{ analysis.fractals.length }}</p>
        </div>
        <div class="rounded-lg bg-secondary/50 p-2.5 text-center">
          <p class="text-muted-foreground">笔数</p>
          <p class="text-lg font-semibold text-foreground mt-0.5">{{ analysis.bis.length }}</p>
        </div>
        <div class="rounded-lg bg-secondary/50 p-2.5 text-center">
          <p class="text-muted-foreground">中枢数</p>
          <p class="text-lg font-semibold text-foreground mt-0.5">{{ analysis.pivots.length }}</p>
        </div>
      </div>

      <!-- Signals -->
      <div v-if="analysis.signals.length > 0" class="space-y-2">
        <p class="text-xs font-medium text-muted-foreground uppercase tracking-wider">检测到的信号</p>
        <div v-for="(s, i) in analysis.signals" :key="i"
          class="flex items-center gap-2 p-2 rounded-lg border"
          :class="s.signal_type.startsWith('buy') ? 'border-profit/30 bg-profit/5' : 'border-loss/30 bg-loss/5'"
        >
          <Badge :variant="s.signal_type.startsWith('buy') ? 'default' : 'destructive'" class="text-xs">
            {{ signalLabelMap[s.signal_type] || s.signal_type }}
          </Badge>
          <span class="text-sm text-foreground flex-1 min-w-0">{{ s.description }}</span>
          <Button variant="ghost" size="sm" class="h-6 px-2 text-xs gap-1 shrink-0"
            :disabled="alertCreating === s.signal_type"
            @click="createAlert(s)">
            <Loader2 v-if="alertCreating === s.signal_type" class="w-3 h-3 animate-spin" />
            <Bell v-else class="w-3 h-3" />
            提醒
          </Button>
        </div>
      </div>
      <div v-else class="text-sm text-muted-foreground text-center py-3">
        暂未检测到买卖信号
      </div>

      <!-- Pivots -->
      <div v-if="analysis.pivots.length > 0" class="space-y-2">
        <p class="text-xs font-medium text-muted-foreground uppercase tracking-wider">中枢区间</p>
        <div v-for="(p, i) in analysis.pivots" :key="i" class="text-xs p-2 rounded bg-secondary/30 flex items-center gap-4">
          <span class="text-muted-foreground">中枢 {{ i + 1 }}</span>
          <span>ZG: <span class="text-foreground font-medium">{{ p.zg.toFixed(2) }}</span></span>
          <span>ZD: <span class="text-foreground font-medium">{{ p.zd.toFixed(2) }}</span></span>
          <span>ZZ: <span class="text-foreground font-medium">{{ p.zz.toFixed(2) }}</span></span>
        </div>
      </div>
    </div>
  </div>
</template>
