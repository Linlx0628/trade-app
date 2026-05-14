<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import {
  TrendingUp, TrendingDown, BarChart3, Activity,
  Wallet, Target, FileText, BookOpen, Loader2,
} from 'lucide-vue-next'
import { useAccountStore } from '@/stores/account'
import { dashboardApi } from '@/lib/tauri'
import type { DashboardStats, PnlTrend, SymbolPnl } from '@/lib/tauri'
import { Card, CardHeader, CardTitle, CardContent, CardDescription } from '@/components/ui/card'
import { cn } from '@/lib/utils'
import { Bar } from 'vue-chartjs'
import {
  Chart as ChartJS, CategoryScale, LinearScale, BarElement,
  PointElement, LineElement, Title, Tooltip, Legend, Filler,
} from 'chart.js'

ChartJS.register(CategoryScale, LinearScale, BarElement, PointElement, LineElement, Title, Tooltip, Legend, Filler)

const router = useRouter()
const accountStore = useAccountStore()

const stats = ref<DashboardStats | null>(null)
const pnlTrend = ref<PnlTrend[]>([])
const symbolPnl = ref<SymbolPnl[]>([])
const loading = ref(false)

const currentAccount = computed(() => accountStore.currentAccount)

async function loadData() {
  if (!currentAccount.value) return
  loading.value = true
  try {
    const [s, t, p] = await Promise.all([
      dashboardApi.getStats(currentAccount.value.id),
      dashboardApi.getPnlTrend(currentAccount.value.id, 30),
      dashboardApi.getSymbolPnl(currentAccount.value.id),
    ])
    stats.value = s
    pnlTrend.value = t
    symbolPnl.value = p
  } catch { /* silent */ }
  finally { loading.value = false }
}

onMounted(async () => {
  await accountStore.fetchAccounts()
  if (!accountStore.currentAccount && accountStore.accounts.length > 0)
    await accountStore.selectAccount(accountStore.accounts[0].id)
  if (accountStore.currentAccount) await loadData()
})

watch(() => accountStore.currentAccount, (acc) => { if (acc) loadData() })

function fmt(v: number) { return new Intl.NumberFormat('zh-CN', { minimumFractionDigits: 0, maximumFractionDigits: 0 }).format(v) }

const pnlChartOptions = computed(() => ({
  responsive: true,
  maintainAspectRatio: false,
  plugins: {
    legend: { display: false },
    tooltip: {
      callbacks: {
        label: (ctx: any) => `盈亏: ${ctx.parsed.y >= 0 ? '+' : ''}${ctx.parsed.y.toFixed(0)}`,
      },
    },
  },
  scales: {
    x: { grid: { display: false }, ticks: { font: { size: 10 }, maxRotation: 0, maxTicksLimit: 10 } },
    y: { grid: { color: 'rgba(0,0,0,0.05)' }, ticks: { font: { size: 10 }, callback: (v: any) => v.toFixed(0) } },
  },
}))

const pnlChartData = computed(() => ({
  labels: pnlTrend.value.map(t => t.date.slice(5)),
  datasets: [{
    data: pnlTrend.value.map(t => t.pnl),
    backgroundColor: pnlTrend.value.map(t => t.pnl >= 0 ? 'rgba(34,197,94,0.6)' : 'rgba(239,68,68,0.6)'),
    borderRadius: 3,
    barPercentage: 0.7,
  }],
}))

const symbolChartOptions = computed(() => ({
  responsive: true,
  maintainAspectRatio: false,
  indexAxis: 'y' as const,
  plugins: { legend: { display: false } },
  scales: {
    x: { grid: { color: 'rgba(0,0,0,0.05)' }, ticks: { font: { size: 10 }, callback: (v: any) => v.toFixed(0) } },
    y: { grid: { display: false }, ticks: { font: { size: 11 } } },
  },
}))

const symbolChartData = computed(() => ({
  labels: symbolPnl.value.slice(0, 8).map(s => s.symbol),
  datasets: [{
    data: symbolPnl.value.slice(0, 8).map(s => s.total_pnl),
    backgroundColor: symbolPnl.value.slice(0, 8).map(s => s.total_pnl >= 0 ? 'rgba(34,197,94,0.7)' : 'rgba(239,68,68,0.7)'),
    borderRadius: 3,
    barPercentage: 0.6,
  }],
}))

const statCards = computed(() => {
  if (!stats.value) return []
  const s = stats.value
  return [
    { label: '账户余额', value: `¥${fmt(s.balance)}`, icon: Wallet, color: 'text-foreground', bg: 'bg-primary/10', iconColor: 'text-primary' },
    { label: '今日盈亏', value: `${s.today_pnl >= 0 ? '+' : ''}¥${fmt(s.today_pnl)}`, icon: s.today_pnl >= 0 ? TrendingUp : TrendingDown, color: s.today_pnl >= 0 ? 'text-profit' : 'text-loss', bg: s.today_pnl >= 0 ? 'bg-profit/10' : 'bg-loss/10', iconColor: s.today_pnl >= 0 ? 'text-profit' : 'text-loss' },
    { label: '总盈亏', value: `${s.total_pnl >= 0 ? '+' : ''}¥${fmt(s.total_pnl)}`, icon: Activity, color: s.total_pnl >= 0 ? 'text-profit' : 'text-loss', bg: s.total_pnl >= 0 ? 'bg-profit/10' : 'bg-loss/10', iconColor: s.total_pnl >= 0 ? 'text-profit' : 'text-loss' },
    { label: '胜率', value: `${(s.win_rate * 100).toFixed(1)}%`, icon: Target, color: 'text-foreground', bg: 'bg-primary/10', iconColor: 'text-primary' },
    { label: '本月交易', value: `${s.month_trades} 笔`, icon: BarChart3, color: 'text-foreground', bg: 'bg-primary/10', iconColor: 'text-primary', sub: `盈亏 ${s.month_pnl >= 0 ? '+' : ''}¥${fmt(s.month_pnl)}` },
  ]
})
</script>

<template>
  <div class="max-w-6xl mx-auto space-y-5">
    <div>
      <h2 class="text-lg font-semibold text-foreground tracking-tight">仪表盘</h2>
      <p class="text-sm text-muted-foreground mt-0.5">交易数据概览与趋势分析</p>
    </div>

    <div v-if="!currentAccount" class="flex items-center justify-center py-16 text-muted-foreground text-sm">
      请先选择账户
    </div>

    <div v-else-if="loading" class="flex items-center justify-center py-16">
      <Loader2 class="w-6 h-6 animate-spin text-muted-foreground" />
    </div>

    <template v-else-if="stats">
      <!-- Stats Cards -->
      <div class="grid grid-cols-2 lg:grid-cols-5 gap-3">
        <Card v-for="card in statCards" :key="card.label" class="transition-all duration-200 hover:shadow-sm">
          <CardContent class="p-4">
            <div class="flex items-center gap-3">
              <div :class="cn('w-9 h-9 rounded-lg flex items-center justify-center shrink-0', card.bg)">
                <component :is="card.icon" :class="cn('w-4 h-4', card.iconColor)" />
              </div>
              <div class="min-w-0">
                <p class="text-[10px] uppercase tracking-wider text-muted-foreground">{{ card.label }}</p>
                <p :class="cn('text-base font-semibold truncate', card.color)">{{ card.value }}</p>
                <p v-if="card.sub" class="text-[10px] text-muted-foreground mt-0.5">{{ card.sub }}</p>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      <!-- Quick Status -->
      <div class="grid grid-cols-2 gap-3">
        <Card class="cursor-pointer hover:shadow-sm transition-all" @click="router.push('/trade-log')">
          <CardContent class="p-4 flex items-center gap-3">
            <div class="w-9 h-9 rounded-lg bg-primary/10 flex items-center justify-center">
              <BookOpen class="w-4 h-4 text-primary" />
            </div>
            <div>
              <p class="text-sm font-medium text-foreground">{{ stats.open_positions }} 个持仓中</p>
              <p class="text-xs text-muted-foreground">点击查看交易日志</p>
            </div>
          </CardContent>
        </Card>
        <Card class="cursor-pointer hover:shadow-sm transition-all" @click="router.push('/trade-plan')">
          <CardContent class="p-4 flex items-center gap-3">
            <div class="w-9 h-9 rounded-lg bg-warning/10 flex items-center justify-center">
              <FileText class="w-4 h-4 text-warning" />
            </div>
            <div>
              <p class="text-sm font-medium text-foreground">{{ stats.pending_plans }} 个待执行计划</p>
              <p class="text-xs text-muted-foreground">点击查看交易计划</p>
            </div>
          </CardContent>
        </Card>
      </div>

      <!-- Charts -->
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <Card>
          <CardHeader class="pb-2">
            <CardTitle class="text-sm font-medium">近 30 天盈亏趋势</CardTitle>
            <CardDescription>每日已平仓交易净盈亏</CardDescription>
          </CardHeader>
          <CardContent>
            <div class="h-56">
              <Bar v-if="pnlTrend.length > 0" :data="pnlChartData" :options="pnlChartOptions" />
              <div v-else class="flex items-center justify-center h-full text-sm text-muted-foreground">暂无数据</div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader class="pb-2">
            <CardTitle class="text-sm font-medium">品种盈亏分布</CardTitle>
            <CardDescription>按交易品种统计的累计盈亏</CardDescription>
          </CardHeader>
          <CardContent>
            <div class="h-56">
              <Bar v-if="symbolPnl.length > 0" :data="symbolChartData" :options="symbolChartOptions" />
              <div v-else class="flex items-center justify-center h-full text-sm text-muted-foreground">暂无数据</div>
            </div>
          </CardContent>
        </Card>
      </div>
    </template>
  </div>
</template>
