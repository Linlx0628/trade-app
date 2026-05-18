import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface MarketQuote {
  symbol: string
  name: string
  open: number
  prev_close: number
  current: number
  high: number
  low: number
  bid: number
  ask: number
  volume: number
  amount: number
  change: number
  change_pct: number
  timestamp: string
}

export interface KlineData {
  timestamp: string
  open: number
  high: number
  low: number
  close: number
  volume: number
}

export interface SymbolInfo {
  symbol: string
  name: string
  market_type: string
}

export const useMarketStore = defineStore('market', () => {
  const quotes = ref<Record<string, MarketQuote>>({})
  const subscriptions = ref<string[]>([])
  const klineData = ref<KlineData[]>([])
  const currentSymbol = ref<string | null>(null)
  const loading = ref(false)
  const connected = ref(false)
  let pollTimer: ReturnType<typeof setInterval> | null = null

  async function fetchQuote(symbol: string): Promise<MarketQuote> {
    const quote = await invoke<MarketQuote>('get_quote', { symbol })
    quotes.value[symbol] = quote
    return quote
  }

  async function fetchQuotes(symbols: string[]): Promise<void> {
    if (symbols.length === 0) return
    const result = await invoke<MarketQuote[]>('get_quotes', { symbols })
    for (const q of result) {
      quotes.value[q.symbol] = q
    }
  }

  async function fetchKline(symbol: string, period: string, count?: number): Promise<void> {
    loading.value = true
    try {
      klineData.value = await invoke<KlineData[]>('get_kline_data', {
        request: { symbol, period, count: count || null },
      })
      currentSymbol.value = symbol
    } finally {
      loading.value = false
    }
  }

  async function searchSymbol(keyword: string): Promise<SymbolInfo[]> {
    return invoke<SymbolInfo[]>('search_symbol', { keyword })
  }

  function subscribe(symbols: string[], intervalMs = 3000): void {
    subscriptions.value = symbols
    connected.value = true

    if (pollTimer) clearInterval(pollTimer)

    fetchQuotes(symbols)
    pollTimer = setInterval(() => {
      fetchQuotes(symbols).catch(() => {
        connected.value = false
        if (pollTimer) clearInterval(pollTimer)
      })
    }, intervalMs)
  }

  function unsubscribe(): void {
    subscriptions.value = []
    connected.value = false
    if (pollTimer) {
      clearInterval(pollTimer)
      pollTimer = null
    }
    invoke('unsubscribe_market').catch(() => {})
  }

  return {
    quotes,
    subscriptions,
    klineData,
    currentSymbol,
    loading,
    connected,
    fetchQuote,
    fetchQuotes,
    fetchKline,
    searchSymbol,
    subscribe,
    unsubscribe,
  }
})
