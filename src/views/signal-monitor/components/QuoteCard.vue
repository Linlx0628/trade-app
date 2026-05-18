<script setup lang="ts">
import { TrendingUp, TrendingDown, Minus } from 'lucide-vue-next'
import type { MarketQuote } from '@/stores/market'

defineProps<{
  quote: MarketQuote
  active?: boolean
}>()

defineEmits<{
  (e: 'click'): void
}>()
</script>

<template>
  <div
    class="p-3 rounded-lg border cursor-pointer transition-all hover:shadow-md"
    :class="active ? 'border-ring ring-1 ring-ring/40 bg-secondary/50' : 'border-border/50 bg-card hover:border-ring/30'"
    @click="$emit('click')"
  >
    <div class="flex items-center justify-between mb-1">
      <span class="text-sm font-medium text-foreground">{{ quote.name }}</span>
      <component :is="quote.change >= 0 ? TrendingUp : quote.change < 0 ? TrendingDown : Minus"
        class="w-3.5 h-3.5" :class="quote.change >= 0 ? 'text-profit' : 'text-loss'"
      />
    </div>
    <div class="text-lg font-bold" :class="quote.change >= 0 ? 'text-profit' : 'text-loss'">
      {{ quote.current.toFixed(2) }}
    </div>
    <div class="flex items-center gap-2 text-xs mt-0.5">
      <span :class="quote.change_pct >= 0 ? 'text-profit' : 'text-loss'">
        {{ quote.change_pct >= 0 ? '+' : '' }}{{ quote.change_pct.toFixed(2) }}%
      </span>
      <span class="text-muted-foreground">{{ quote.symbol }}</span>
    </div>
  </div>
</template>
