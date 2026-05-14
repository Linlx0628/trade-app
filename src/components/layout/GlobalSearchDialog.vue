<script setup lang="ts">
import { ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { Search, FileText, BookOpen, StickyNote, Loader2 } from 'lucide-vue-next'
import { searchApi } from '@/lib/tauri'
import { useAccountStore } from '@/stores/account'
import type { SearchResult } from '@/lib/tauri'

const emit = defineEmits<{ (e: 'close'): void }>()

const router = useRouter()
const accountStore = useAccountStore()

const query = ref('')
const results = ref<SearchResult[]>([])
const loading = ref(false)
const selectedIndex = ref(0)

let debounceTimer: ReturnType<typeof setTimeout> | null = null

watch(query, (q) => {
  if (debounceTimer) clearTimeout(debounceTimer)
  if (!q.trim()) { results.value = []; return }
  debounceTimer = setTimeout(async () => {
    if (!accountStore.currentAccount) return
    loading.value = true
    try {
      results.value = await searchApi.search(accountStore.currentAccount.id, q.trim())
      selectedIndex.value = 0
    } catch { results.value = [] }
    finally { loading.value = false }
  }, 300)
})

function go(r: SearchResult) {
  const routes: Record<string, string> = { plan: '/trade-plan', log: '/trade-log', summary: '/trade-summary' }
  router.push(routes[r.item_type] || '/')
  emit('close')
}

function handleKey(e: KeyboardEvent) {
  if (e.key === 'Escape') { emit('close'); return }
  if (results.value.length === 0) return
  if (e.key === 'ArrowDown') { e.preventDefault(); selectedIndex.value = Math.min(selectedIndex.value + 1, results.value.length - 1) }
  else if (e.key === 'ArrowUp') { e.preventDefault(); selectedIndex.value = Math.max(selectedIndex.value - 1, 0) }
  else if (e.key === 'Enter' && results.value[selectedIndex.value]) { go(results.value[selectedIndex.value]) }
}

const typeIcon: Record<string, typeof FileText> = { plan: FileText, log: BookOpen, summary: StickyNote }
const typeLabel: Record<string, string> = { plan: '计划', log: '日志', summary: '总结' }
const typeColor: Record<string, string> = { plan: 'text-primary bg-primary/10', log: 'text-profit bg-profit/10', summary: 'text-warning bg-warning/10' }
</script>

<template>
  <div class="fixed inset-0 z-50 flex items-start justify-center pt-[12vh]" @click.self="emit('close')">
    <div class="fixed inset-0 bg-black/30 backdrop-blur-sm" @click="emit('close')" />
    <div class="relative w-full max-w-lg bg-card border border-border rounded-xl shadow-2xl overflow-hidden" @keydown="handleKey">
      <div class="flex items-center gap-3 px-4 py-3 border-b border-border/50">
        <Search class="w-4 h-4 text-muted-foreground shrink-0" />
        <input
          v-model="query"
          type="text"
          placeholder="搜索计划、日志、总结..."
          class="flex-1 bg-transparent text-sm text-foreground placeholder:text-muted-foreground outline-none"
          autofocus
        />
        <Loader2 v-if="loading" class="w-4 h-4 animate-spin text-muted-foreground" />
        <kbd v-else class="text-[10px] text-muted-foreground bg-secondary px-1.5 py-0.5 rounded">ESC</kbd>
      </div>

      <div v-if="query.trim() && !loading && results.length === 0" class="py-8 text-center text-sm text-muted-foreground">
        没有找到匹配的结果
      </div>

      <div v-if="results.length > 0" class="max-h-[50vh] overflow-y-auto p-1.5">
        <button
          v-for="(r, idx) in results" :key="r.id"
          :class="['w-full flex items-center gap-3 px-3 py-2.5 rounded-lg transition-colors text-left', idx === selectedIndex ? 'bg-secondary' : 'hover:bg-secondary/50']"
          @click="go(r)" @mouseenter="selectedIndex = idx"
        >
          <div :class="['w-8 h-8 rounded-lg flex items-center justify-center shrink-0', typeColor[r.item_type] || 'bg-secondary text-muted-foreground']">
            <component :is="typeIcon[r.item_type] || FileText" class="w-4 h-4" />
          </div>
          <div class="flex-1 min-w-0">
            <p class="text-sm font-medium text-foreground truncate">{{ r.title }}</p>
            <p class="text-xs text-muted-foreground truncate">{{ r.subtitle }}</p>
          </div>
          <span class="text-[10px] text-muted-foreground/60 shrink-0">{{ typeLabel[r.item_type] }}</span>
        </button>
      </div>

      <div v-if="!query.trim()" class="py-6 text-center text-sm text-muted-foreground">
        输入关键词搜索交易计划、日志和总结
      </div>
    </div>
  </div>
</template>
