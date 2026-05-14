<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { X, Search, Pin, TrendingUp, TrendingDown, Loader2 } from 'lucide-vue-next'
import { useTradeTemplateStore } from '@/stores/trade-template'
import { useAccountStore } from '@/stores/account'
import { cn } from '@/lib/utils'
import type { TradeTemplate } from '@/types/common'

const emit = defineEmits<{
  (e: 'select', template: TradeTemplate): void
  (e: 'close'): void
}>()

const templateStore = useTradeTemplateStore()
const accountStore = useAccountStore()
const searchQuery = ref('')

onMounted(async () => {
  if (accountStore.currentAccount) {
    await templateStore.fetchTemplates(accountStore.currentAccount.id)
  }
})

const filteredTemplates = computed(() => {
  const q = searchQuery.value.toLowerCase().trim()
  if (!q) return templateStore.templates
  return templateStore.templates.filter(t =>
    t.name.toLowerCase().includes(q) ||
    t.symbol.toLowerCase().includes(q) ||
    t.strategy.toLowerCase().includes(q)
  )
})

const pinnedTemplates = computed(() => filteredTemplates.value.filter(t => t.is_pinned))
const otherTemplates = computed(() => filteredTemplates.value.filter(t => !t.is_pinned))

function formatPercent(v: number) { return `${(v * 100).toFixed(1)}%` }
</script>

<template>
  <div class="fixed inset-0 z-50 flex items-start justify-center pt-[10vh]" @click.self="emit('close')">
    <div class="fixed inset-0 bg-black/30 backdrop-blur-sm" @click="emit('close')" />
    <div class="relative w-full max-w-lg bg-card border border-border rounded-xl shadow-2xl overflow-hidden">
      <!-- Header -->
      <div class="flex items-center gap-3 px-4 py-3 border-b border-border/50">
        <div class="flex-1 flex items-center gap-2 bg-secondary/50 rounded-lg px-3 py-2">
          <Search class="w-4 h-4 text-muted-foreground shrink-0" />
          <input
            v-model="searchQuery"
            type="text"
            placeholder="搜索模板名称、品种或策略..."
            class="flex-1 bg-transparent text-sm text-foreground placeholder:text-muted-foreground outline-none"
            autofocus
          />
        </div>
        <button class="p-1.5 rounded-md hover:bg-secondary text-muted-foreground hover:text-foreground transition-colors" @click="emit('close')">
          <X class="w-4 h-4" />
        </button>
      </div>

      <!-- Loading -->
      <div v-if="templateStore.loading" class="flex items-center justify-center py-12">
        <Loader2 class="w-5 h-5 animate-spin text-muted-foreground" />
      </div>

      <!-- Empty -->
      <div v-else-if="filteredTemplates.length === 0" class="flex flex-col items-center justify-center py-12">
        <p class="text-sm text-muted-foreground">{{ searchQuery ? '没有匹配的模板' : '暂无模板' }}</p>
        <p class="text-xs text-muted-foreground/60 mt-1">可以先在计划卡片中选择「保存为模板」</p>
      </div>

      <!-- Template List -->
      <div v-else class="max-h-[50vh] overflow-y-auto p-2">
        <!-- Pinned section -->
        <template v-if="pinnedTemplates.length > 0">
          <p class="px-2 py-1.5 text-[10px] uppercase tracking-wider text-muted-foreground font-medium">置顶模板</p>
          <button
            v-for="t in pinnedTemplates" :key="t.id"
            class="w-full flex items-center gap-3 px-3 py-2.5 rounded-lg hover:bg-secondary/80 transition-colors text-left group"
            @click="emit('select', t)"
          >
            <div :class="cn('w-8 h-8 rounded-lg flex items-center justify-center shrink-0',
              t.direction === 'long' ? 'bg-profit/10 text-profit' : 'bg-loss/10 text-loss')">
              <TrendingUp v-if="t.direction === 'long'" class="w-4 h-4" />
              <TrendingDown v-else class="w-4 h-4" />
            </div>
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2">
                <span class="text-sm font-medium text-foreground truncate">{{ t.name }}</span>
                <Pin class="w-3 h-3 text-primary shrink-0" />
              </div>
              <div class="flex items-center gap-2 mt-0.5 text-xs text-muted-foreground">
                <span v-if="t.symbol">{{ t.symbol }}</span>
                <span v-if="t.symbol">·</span>
                <span>{{ t.direction === 'long' ? '多' : '空' }}</span>
                <span v-if="t.stop_loss_ratio > 0">· 止损 {{ formatPercent(t.stop_loss_ratio) }}</span>
                <span v-if="t.take_profit_ratio > 0">· 止盈 {{ formatPercent(t.take_profit_ratio) }}</span>
              </div>
            </div>
            <span class="text-[10px] text-muted-foreground/60">使用 {{ t.usage_count }}次</span>
          </button>
        </template>

        <!-- Other templates -->
        <template v-if="otherTemplates.length > 0">
          <p v-if="pinnedTemplates.length > 0" class="px-2 py-1.5 mt-2 text-[10px] uppercase tracking-wider text-muted-foreground font-medium">其他模板</p>
          <button
            v-for="t in otherTemplates" :key="t.id"
            class="w-full flex items-center gap-3 px-3 py-2.5 rounded-lg hover:bg-secondary/80 transition-colors text-left group"
            @click="emit('select', t)"
          >
            <div :class="cn('w-8 h-8 rounded-lg flex items-center justify-center shrink-0',
              t.direction === 'long' ? 'bg-profit/10 text-profit' : 'bg-loss/10 text-loss')">
              <TrendingUp v-if="t.direction === 'long'" class="w-4 h-4" />
              <TrendingDown v-else class="w-4 h-4" />
            </div>
            <div class="flex-1 min-w-0">
              <p class="text-sm font-medium text-foreground truncate">{{ t.name }}</p>
              <div class="flex items-center gap-2 mt-0.5 text-xs text-muted-foreground">
                <span v-if="t.symbol">{{ t.symbol }}</span>
                <span v-if="t.symbol">·</span>
                <span>{{ t.direction === 'long' ? '多' : '空' }}</span>
                <span v-if="t.stop_loss_ratio > 0">· 止损 {{ formatPercent(t.stop_loss_ratio) }}</span>
                <span v-if="t.take_profit_ratio > 0">· 止盈 {{ formatPercent(t.take_profit_ratio) }}</span>
              </div>
            </div>
            <span class="text-[10px] text-muted-foreground/60">使用 {{ t.usage_count }}次</span>
          </button>
        </template>
      </div>
    </div>
  </div>
</template>
