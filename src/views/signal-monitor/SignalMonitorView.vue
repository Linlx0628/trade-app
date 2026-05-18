<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { RefreshCw, Star, Trash2 } from 'lucide-vue-next'
import { useMarketStore, type SymbolInfo } from '@/stores/market'
import { Button } from '@/components/ui/button'
import { Separator } from '@/components/ui/separator'
import { SelectNative } from '@/components/ui/select'
import PriceTicker from './components/PriceTicker.vue'
import KlineChart from './components/KlineChart.vue'
import ChanlunPanel from './components/ChanlunPanel.vue'
import AIAnalysisPanel from './components/AIAnalysisPanel.vue'

const marketStore = useMarketStore()

const currentPeriod = ref('day')
const searchKeyword = ref('')
const searchResults = ref<SymbolInfo[]>([])
const searchLoading = ref(false)
const activeTab = ref<'main' | 'chanlun' | 'ai'>('main')

const periods = [
  { value: '5m', label: '5分钟' },
  { value: '15m', label: '15分钟' },
  { value: '30m', label: '30分钟' },
  { value: '60m', label: '60分钟' },
  { value: 'day', label: '日K' },
  { value: 'week', label: '周K' },
  { value: 'month', label: '月K' },
]

// ── Watchlist 持久化 ──
interface WatchlistItem {
  symbol: string
  name: string
  market_type: string
}

const watchlist = ref<WatchlistItem[]>([])
const STORAGE_KEY = 'signal_monitor_watchlist'

function loadWatchlist() {
  try {
    const saved = localStorage.getItem(STORAGE_KEY)
    if (saved) watchlist.value = JSON.parse(saved)
  } catch {}
}

function saveWatchlist() {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(watchlist.value))
}

function addToWatchlist(item: SymbolInfo) {
  if (watchlist.value.some(w => w.symbol === item.symbol)) return
  watchlist.value.push({ symbol: item.symbol, name: item.name, market_type: item.market_type })
  saveWatchlist()
}

function removeFromWatchlist(symbol: string) {
  watchlist.value = watchlist.value.filter(w => w.symbol !== symbol)
  saveWatchlist()
  if (marketStore.currentSymbol === symbol) {
    marketStore.unsubscribe()
    marketStore.currentSymbol = null
    marketStore.klineData = []
  }
}

const stocks = computed(() => watchlist.value.filter(w => w.market_type !== 'futures'))
const futures = computed(() => watchlist.value.filter(w => w.market_type === 'futures'))

// ── 当前行情 ──
const currentQuote = computed(() => {
  const sym = marketStore.currentSymbol
  return sym ? marketStore.quotes[sym] || null : null
})

function getQuote(symbol: string) {
  return marketStore.quotes[symbol] || null
}

// ── 搜索 ──
let searchTimer: ReturnType<typeof setTimeout> | null = null

watch(searchKeyword, (val) => {
  if (searchTimer) clearTimeout(searchTimer)
  if (!val.trim()) { searchResults.value = []; return }
  searchTimer = setTimeout(async () => {
    searchLoading.value = true
    try {
      searchResults.value = await marketStore.searchSymbol(val.trim())
    } catch { searchResults.value = [] }
    finally { searchLoading.value = false }
  }, 300)
})

function handleSearchSelect(item: SymbolInfo) {
  addToWatchlist(item)
  selectSymbol(item.symbol)
  searchKeyword.value = ''
  searchResults.value = []
}

// ── 品种选择 ──
async function selectSymbol(symbol: string) {
  const allSymbols = watchlist.value.map(w => w.symbol)
  marketStore.subscribe(allSymbols, 5000)
  await marketStore.fetchKline(symbol, currentPeriod.value, 200)
}

async function changePeriod(period: string) {
  currentPeriod.value = period
  if (marketStore.currentSymbol) {
    await marketStore.fetchKline(marketStore.currentSymbol, period, 200)
  }
}

async function refreshKline() {
  if (marketStore.currentSymbol) {
    await marketStore.fetchKline(marketStore.currentSymbol, currentPeriod.value, 200)
  }
}

function formatChange(val: number): string {
  const sign = val >= 0 ? '+' : ''
  return `${sign}${val.toFixed(2)}%`
}

const marketLabels: Record<string, string> = {
  stock: 'A股', hk_stock: '港股', us_stock: '美股', futures: '期货',
}

// ── Lifecycle ──
onMounted(() => {
  loadWatchlist()
  if (watchlist.value.length > 0) {
    const allSymbols = watchlist.value.map(w => w.symbol)
    marketStore.subscribe(allSymbols, 5000)
    selectSymbol(watchlist.value[0].symbol)
  }
})

onUnmounted(() => {
  // 不取消订阅 — 让行情轮询保持运行
})
</script>

<template>
  <div class="flex h-full gap-4">
    <!-- ── Left: Main Content ── -->
    <div class="flex-1 min-w-0 space-y-4 overflow-y-auto">
      <!-- Empty state -->
      <div v-if="!currentQuote && watchlist.length === 0" class="flex flex-col items-center justify-center py-20 text-muted-foreground">
        <Star class="w-12 h-12 mb-3 opacity-40" />
        <p class="text-sm">在右侧搜索并添加品种到自选列表</p>
      </div>

      <div v-if="!currentQuote && watchlist.length > 0" class="text-center py-10 text-sm text-muted-foreground">
        请从右侧自选列表中选择一个品种查看
      </div>

      <!-- Price Ticker + K线 -->
      <template v-if="currentQuote">
        <PriceTicker :quote="currentQuote" />

        <!-- Period + Tab -->
        <div class="flex items-center gap-3">
          <SelectNative :model-value="currentPeriod" @update:model-value="changePeriod" class="w-24 text-xs">
            <option v-for="p in periods" :key="p.value" :value="p.value">{{ p.label }}</option>
          </SelectNative>
          <Button variant="ghost" size="icon" class="h-8 w-8" @click="refreshKline">
            <RefreshCw :class="['w-4 h-4', marketStore.loading && 'animate-spin']" />
          </Button>
          <div class="flex-1" />
          <div class="flex gap-1">
            <Button size="sm" :variant="activeTab === 'main' ? 'default' : 'ghost'" class="text-xs h-7" @click="activeTab = 'main'">K线</Button>
            <Button size="sm" :variant="activeTab === 'chanlun' ? 'default' : 'ghost'" class="text-xs h-7" @click="activeTab = 'chanlun'">缠论</Button>
            <Button size="sm" :variant="activeTab === 'ai' ? 'default' : 'ghost'" class="text-xs h-7" @click="activeTab = 'ai'">AI分析</Button>
          </div>
        </div>

        <!-- K线图 -->
        <div v-if="activeTab === 'main'" class="h-[500px] rounded-lg border border-border/50 bg-card">
          <KlineChart :data="marketStore.klineData" :loading="marketStore.loading" />
        </div>

        <!-- 缠论分析 -->
        <ChanlunPanel v-if="activeTab === 'chanlun' && marketStore.currentSymbol" :symbol="marketStore.currentSymbol" />

        <!-- AI分析 -->
        <AIAnalysisPanel v-if="activeTab === 'ai' && marketStore.currentSymbol"
          :symbol="marketStore.currentSymbol"
          :symbol-name="currentQuote?.name || marketStore.currentSymbol"
          :kline-data="marketStore.klineData"
          :period="currentPeriod" />
      </template>
    </div>

    <!-- ── Right: Watchlist Sidebar ── -->
    <div class="w-72 shrink-0 border-l border-border pl-4 flex flex-col gap-3 overflow-y-auto">
      <!-- Search -->
      <div class="relative z-50">
        <input
          v-model="searchKeyword"
          placeholder="搜索品种代码或名称..."
          class="w-full h-9 rounded-lg border border-border bg-card px-3 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring"
        />
        <div v-if="searchLoading" class="absolute right-3 top-1/2 -translate-y-1/2">
          <RefreshCw class="w-4 h-4 animate-spin text-muted-foreground" />
        </div>

        <!-- Search Results Dropdown -->
        <div v-if="searchResults.length > 0" class="absolute top-full left-0 right-0 mt-1 max-h-64 overflow-y-auto rounded-lg border border-border bg-card shadow-lg z-50">
          <button
            v-for="item in searchResults.slice(0, 15)"
            :key="item.symbol"
            class="w-full px-3 py-2 text-left text-sm hover:bg-secondary/50 transition-colors flex items-center justify-between"
            @click="handleSearchSelect(item)"
          >
            <div class="flex items-center gap-2 min-w-0">
              <span class="font-medium text-foreground truncate">{{ item.name }}</span>
              <span class="text-muted-foreground text-xs shrink-0">{{ item.symbol }}</span>
            </div>
            <span class="text-[10px] text-muted-foreground shrink-0 ml-1">
              {{ marketLabels[item.market_type] || item.market_type }}
            </span>
          </button>
        </div>
      </div>

      <Separator />

      <!-- Watchlist: 股票 -->
      <div v-if="stocks.length > 0" class="space-y-1">
        <p class="text-xs font-medium text-muted-foreground uppercase tracking-wider">股票</p>
        <div v-for="item in stocks" :key="item.symbol"
          class="group flex items-center gap-2 px-2 py-2 rounded-lg cursor-pointer transition-colors"
          :class="marketStore.currentSymbol === item.symbol ? 'bg-primary/10' : 'hover:bg-secondary/50'"
          @click="selectSymbol(item.symbol)">
          <div class="flex-1 min-w-0">
            <p class="text-sm font-medium text-foreground truncate">{{ item.name }}</p>
            <div class="flex items-center gap-1 text-xs">
              <span class="text-muted-foreground">{{ item.symbol }}</span>
              <span class="text-muted-foreground/50">{{ marketLabels[item.market_type] }}</span>
            </div>
          </div>
          <div v-if="getQuote(item.symbol)" class="text-right shrink-0">
            <p class="text-sm font-semibold"
              :class="getQuote(item.symbol)!.change >= 0 ? 'text-profit' : 'text-loss'">
              {{ getQuote(item.symbol)!.current.toFixed(2) }}
            </p>
            <p class="text-xs"
              :class="getQuote(item.symbol)!.change >= 0 ? 'text-profit' : 'text-loss'">
              {{ formatChange(getQuote(item.symbol)!.change_pct) }}
            </p>
          </div>
          <Button variant="ghost" size="icon" class="h-6 w-6 opacity-0 group-hover:opacity-100 shrink-0 hover:text-loss"
            @click.stop="removeFromWatchlist(item.symbol)">
            <Trash2 class="w-3 h-3" />
          </Button>
        </div>
      </div>

      <!-- Watchlist: 期货 -->
      <div v-if="futures.length > 0" class="space-y-1">
        <p class="text-xs font-medium text-muted-foreground uppercase tracking-wider">期货</p>
        <div v-for="item in futures" :key="item.symbol"
          class="group flex items-center gap-2 px-2 py-2 rounded-lg cursor-pointer transition-colors"
          :class="marketStore.currentSymbol === item.symbol ? 'bg-primary/10' : 'hover:bg-secondary/50'"
          @click="selectSymbol(item.symbol)">
          <div class="flex-1 min-w-0">
            <p class="text-sm font-medium text-foreground truncate">{{ item.name }}</p>
            <div class="flex items-center gap-1 text-xs">
              <span class="text-muted-foreground">{{ item.symbol }}</span>
            </div>
          </div>
          <div v-if="getQuote(item.symbol)" class="text-right shrink-0">
            <p class="text-sm font-semibold"
              :class="getQuote(item.symbol)!.change >= 0 ? 'text-profit' : 'text-loss'">
              {{ getQuote(item.symbol)!.current.toFixed(2) }}
            </p>
            <p class="text-xs"
              :class="getQuote(item.symbol)!.change >= 0 ? 'text-profit' : 'text-loss'">
              {{ formatChange(getQuote(item.symbol)!.change_pct) }}
            </p>
          </div>
          <Button variant="ghost" size="icon" class="h-6 w-6 opacity-0 group-hover:opacity-100 shrink-0 hover:text-loss"
            @click.stop="removeFromWatchlist(item.symbol)">
            <Trash2 class="w-3 h-3" />
          </Button>
        </div>
      </div>

      <!-- Empty watchlist -->
      <div v-if="watchlist.length === 0" class="flex-1 flex flex-col items-center justify-center text-muted-foreground py-10">
        <Star class="w-8 h-8 mb-2 opacity-30" />
        <p class="text-xs">搜索品种并添加到自选</p>
      </div>

      <div class="flex-1" />
    </div>
  </div>
</template>
