<script setup lang="ts">
import { TrendingUp, TrendingDown, Minus } from 'lucide-vue-next'
import type { MarketQuote } from '@/stores/market'

defineProps<{
  quote: MarketQuote
}>()

// 根据价格大小自动调整小数位
function formatPrice(v: number): string {
  if (v >= 10000) return v.toFixed(0)
  if (v >= 100) return v.toFixed(2)
  return v.toFixed(3)
}

function formatVolume(v: number): string {
  if (v >= 10000) return `${(v / 10000).toFixed(0)}万`
  return String(Math.round(v))
}

function formatAmount(v: number): string {
  if (v >= 100000000) return `${(v / 100000000).toFixed(2)}亿`
  if (v >= 10000) return `${(v / 10000).toFixed(0)}万`
  return String(Math.round(v))
}
</script>

<template>
  <div class="flex items-center gap-4 p-3 rounded-lg border border-border/50 bg-card">
    <!-- Name + Price -->
    <div class="min-w-[140px]">
      <p class="text-sm font-medium text-foreground mb-0.5">{{ quote.name }}</p>
      <div class="flex items-center gap-1.5">
        <TrendingUp v-if="quote.change > 0" class="w-4 h-4 text-profit" />
        <TrendingDown v-else-if="quote.change < 0" class="w-4 h-4 text-loss" />
        <Minus v-else class="w-4 h-4 text-muted-foreground" />
        <span class="text-2xl font-bold" :class="quote.change >= 0 ? 'text-profit' : 'text-loss'">
          {{ formatPrice(quote.current) }}
        </span>
      </div>
      <div class="flex items-center gap-2 mt-0.5 text-xs">
        <span :class="quote.change >= 0 ? 'text-profit' : 'text-loss'">
          {{ quote.change >= 0 ? '+' : '' }}{{ formatPrice(Math.abs(quote.change)) }}
        </span>
        <span :class="quote.change_pct >= 0 ? 'text-profit' : 'text-loss'">
          {{ quote.change_pct >= 0 ? '+' : '' }}{{ quote.change_pct.toFixed(2) }}%
        </span>
      </div>
    </div>

    <!-- Details -->
    <div class="grid grid-cols-3 gap-x-6 gap-y-1 text-xs">
      <div>
        <span class="text-muted-foreground">开盘</span>
        <span class="ml-1.5 text-foreground">{{ formatPrice(quote.open) }}</span>
      </div>
      <div>
        <span class="text-muted-foreground">最高</span>
        <span class="ml-1.5 text-profit">{{ formatPrice(quote.high) }}</span>
      </div>
      <div>
        <span class="text-muted-foreground">最低</span>
        <span class="ml-1.5 text-loss">{{ formatPrice(quote.low) }}</span>
      </div>
      <div>
        <span class="text-muted-foreground">昨收</span>
        <span class="ml-1.5 text-foreground">{{ formatPrice(quote.prev_close) }}</span>
      </div>
      <div>
        <span class="text-muted-foreground">成交量</span>
        <span class="ml-1.5 text-foreground">{{ formatVolume(quote.volume) }}</span>
      </div>
      <div>
        <span class="text-muted-foreground">成交额</span>
        <span class="ml-1.5 text-foreground">{{ formatAmount(quote.amount) }}</span>
      </div>
    </div>
  </div>
</template>
