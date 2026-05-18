<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { Activity, RefreshCw } from 'lucide-vue-next'
import { useMarketStore, type SymbolInfo } from '@/stores/market'
import { Button } from '@/components/ui/button'
import { SelectNative } from '@/components/ui/select'
import SymbolSearch from './components/SymbolSearch.vue'
import PriceTicker from './components/PriceTicker.vue'
import QuoteCard from './components/QuoteCard.vue'
import KlineChart from './components/KlineChart.vue'
import ChanlunPanel from './components/ChanlunPanel.vue'
import SignalAlertList from './components/SignalAlertList.vue'

const marketStore = useMarketStore()

const currentPeriod = ref('day')
const selectedSymbols = ref<string[]>([])
const periods = [
  { value: '1m', label: '1分钟' },
  { value: '5m', label: '5分钟' },
  { value: '15m', label: '15分钟' },
  { value: '30m', label: '30分钟' },
  { value: '60m', label: '60分钟' },
  { value: 'day', label: '日线' },
]

const currentQuote = computed(() => {
  const sym = marketStore.currentSymbol
  if (!sym) return null
  return marketStore.quotes[sym] || null
})

function handleSymbolSelect(item: SymbolInfo) {
  if (!selectedSymbols.value.includes(item.symbol)) {
    selectedSymbols.value.push(item.symbol)
  }
  selectSymbol(item.symbol)
}

async function selectSymbol(symbol: string) {
  marketStore.subscribe([symbol], 5000)
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

watch(selectedSymbols, (syms) => {
  if (syms.length > 0) {
    marketStore.subscribe(syms, 5000)
  }
}, { deep: true })

onMounted(() => {
  if (selectedSymbols.value.length > 0) {
    selectSymbol(selectedSymbols.value[0])
  }
})

onUnmounted(() => {
  marketStore.unsubscribe()
})
</script>

<template>
  <div class="space-y-4">
    <!-- Header -->
    <div class="flex items-center gap-3">
      <div class="flex items-center justify-center w-9 h-9 rounded-lg bg-primary/10">
        <Activity class="w-5 h-5 text-primary" />
      </div>
      <div class="flex-1">
        <h1 class="text-lg font-semibold text-foreground">行情监控</h1>
        <p class="text-sm text-muted-foreground">实时行情数据与K线图表</p>
      </div>
      <SymbolSearch @select="handleSymbolSelect" />
    </div>

    <!-- Quote Cards Row -->
    <div v-if="selectedSymbols.length > 0" class="grid grid-cols-2 md:grid-cols-4 gap-3">
      <QuoteCard
        v-for="sym in selectedSymbols"
        :key="sym"
        :quote="marketStore.quotes[sym] || { symbol: sym, name: sym, current: 0, change: 0, change_pct: 0, open: 0, prev_close: 0, high: 0, low: 0, bid: 0, ask: 0, volume: 0, amount: 0, timestamp: '' }"
        :active="marketStore.currentSymbol === sym"
        @click="selectSymbol(sym)"
      />
    </div>

    <!-- Empty state -->
    <div v-if="selectedSymbols.length === 0" class="flex flex-col items-center justify-center py-20 text-muted-foreground">
      <Activity class="w-12 h-12 mb-3 opacity-40" />
      <p class="text-sm">搜索并添加品种以查看行情</p>
    </div>

    <!-- Price Ticker + Chart -->
    <template v-if="currentQuote">
      <PriceTicker :quote="currentQuote" />

      <!-- Period selector + chart -->
      <div class="flex items-center gap-2">
        <SelectNative :model-value="currentPeriod" @update:model-value="changePeriod" class="w-24 text-xs">
          <option v-for="p in periods" :key="p.value" :value="p.value">{{ p.label }}</option>
        </SelectNative>
        <Button variant="ghost" size="icon" class="h-8 w-8" @click="refreshKline">
          <RefreshCw :class="['w-4 h-4', marketStore.loading && 'animate-spin']" />
        </Button>
      </div>

      <div class="h-[500px] rounded-lg border border-border/50 bg-card">
        <KlineChart :data="marketStore.klineData" :loading="marketStore.loading" />
      </div>

      <!-- Chanlun Analysis + Signal Alerts -->
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <ChanlunPanel v-if="marketStore.currentSymbol" :symbol="marketStore.currentSymbol" />
        <SignalAlertList />
      </div>
    </template>
  </div>
</template>
