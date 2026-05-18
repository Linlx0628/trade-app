<script setup lang="ts">
import { ref, watch } from 'vue'
import { Search, Loader2 } from 'lucide-vue-next'
import { useMarketStore, type SymbolInfo } from '@/stores/market'
import { Input } from '@/components/ui/input'

const marketStore = useMarketStore()
const keyword = ref('')
const results = ref<SymbolInfo[]>([])
const searching = ref(false)
let searchTimer: ReturnType<typeof setTimeout> | null = null

const emit = defineEmits<{
  (e: 'select', symbol: SymbolInfo): void
}>()

watch(keyword, (val) => {
  if (searchTimer) clearTimeout(searchTimer)
  if (!val.trim()) {
    results.value = []
    return
  }
  searchTimer = setTimeout(async () => {
    searching.value = true
    try {
      results.value = await marketStore.searchSymbol(val.trim())
    } catch {
      results.value = []
    } finally {
      searching.value = false
    }
  }, 300)
})

function handleSelect(item: SymbolInfo) {
  emit('select', item)
  keyword.value = ''
  results.value = []
}
</script>

<template>
  <div class="relative">
    <div class="relative">
      <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
      <Input
        v-model="keyword"
        placeholder="搜索品种代码或名称..."
        class="pl-9"
      />
      <Loader2 v-if="searching" class="absolute right-3 top-1/2 -translate-y-1/2 w-4 h-4 animate-spin text-muted-foreground" />
    </div>
    <div v-if="results.length > 0" class="absolute top-full left-0 right-0 mt-1 bg-card border border-border rounded-lg shadow-lg z-50 max-h-64 overflow-y-auto">
      <button
        v-for="item in results"
        :key="item.symbol"
        class="w-full px-3 py-2 text-left text-sm hover:bg-secondary/50 transition-colors flex items-center justify-between"
        @click="handleSelect(item)"
      >
        <div class="flex items-center gap-2">
          <span class="font-medium text-foreground">{{ item.name }}</span>
          <span class="text-muted-foreground text-xs">{{ item.symbol }}</span>
        </div>
        <span class="text-xs text-muted-foreground">
          {{ { stock: 'A股', hk_stock: '港股', us_stock: '美股', futures: '期货' }[item.market_type] || item.market_type }}
        </span>
      </button>
    </div>
  </div>
</template>
