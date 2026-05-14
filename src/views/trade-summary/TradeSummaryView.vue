<script setup lang="ts">
import { ref, reactive, computed, onMounted, watch } from 'vue'
import {
  Plus, StickyNote, Loader2, Trash2, Pencil, X,
  TrendingUp, TrendingDown, BarChart3, Calendar, Tag, Sparkles,
} from 'lucide-vue-next'
import { useTradeSummaryStore } from '@/stores/trade-summary'
import { useAccountStore } from '@/stores/account'
import { useToast } from '@/components/ui/toast'
import { useAi } from '@/composables/useAi'
import { renderMarkdown } from '@/lib/markdown'
import { Card, CardHeader, CardTitle, CardDescription, CardContent } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Label } from '@/components/ui/label'
import { Badge } from '@/components/ui/badge'
import { Textarea } from '@/components/ui/textarea'
import { SelectNative } from '@/components/ui/select'
import {
  AlertDialog, AlertDialogTrigger, AlertDialogContent, AlertDialogHeader,
  AlertDialogFooter, AlertDialogTitle, AlertDialogDescription, AlertDialogAction, AlertDialogCancel,
} from '@/components/ui/alert-dialog'
import { cn } from '@/lib/utils'
import type { TradeSummary, SummaryType } from '@/types/common'

const summaryStore = useTradeSummaryStore()
const accountStore = useAccountStore()
const { toast } = useToast()

const showForm = ref(false)
const editingSummary = ref<TradeSummary | null>(null)
const deleteTarget = ref<TradeSummary | null>(null)
const formSubmitting = ref(false)

const form = reactive({
  summary_type: 'daily' as SummaryType,
  summary_date: new Date().toISOString().slice(0, 10),
  emotion_score: 5,
  market_view: '',
  lessons: '',
  improvement: '',
  tagsInput: '',
})

const currentAccount = computed(() => accountStore.currentAccount)
const isEditing = computed(() => editingSummary.value !== null)

const typeTabs = computed(() => [
  { key: 'all' as const, label: '全部' },
  { key: 'daily' as const, label: '日总结' },
  { key: 'weekly' as const, label: '周总结' },
  { key: 'monthly' as const, label: '月总结' },
])

function formatCurrency(v: number) {
  return new Intl.NumberFormat('zh-CN', { minimumFractionDigits: 0, maximumFractionDigits: 2 }).format(v)
}

function formatPercent(v: number) { return `${(v * 100).toFixed(1)}%` }

function resetForm() {
  Object.assign(form, {
    summary_type: 'daily', summary_date: new Date().toISOString().slice(0, 10),
    emotion_score: 5, market_view: '', lessons: '', improvement: '', tagsInput: '',
  })
  editingSummary.value = null
}

function openCreateForm() { resetForm(); showForm.value = true }

function openEditForm(s: TradeSummary) {
  editingSummary.value = s
  Object.assign(form, {
    summary_type: s.summary_type as SummaryType, summary_date: s.summary_date,
    emotion_score: s.emotion_score, market_view: s.market_view,
    lessons: s.lessons, improvement: s.improvement,
    tagsInput: (() => { try { return JSON.parse(s.tags || '[]').join(', ') } catch { return '' } })(),
  })
  showForm.value = true
}

function closeForm() { showForm.value = false; resetForm() }

async function handleSubmit() {
  if (!currentAccount.value) { toast({ title: '请先选择账户', variant: 'destructive' }); return }
  formSubmitting.value = true
  const tags = form.tagsInput.split(/[,，]/).map(t => t.trim()).filter(Boolean)
  try {
    if (isEditing.value) {
      await summaryStore.updateSummary({
        id: editingSummary.value!.id, emotion_score: form.emotion_score,
        market_view: form.market_view, lessons: form.lessons, improvement: form.improvement, tags,
      })
      toast({ title: '更新成功', variant: 'success' })
    } else {
      await summaryStore.createSummary({
        account_id: currentAccount.value.id, summary_type: form.summary_type,
        summary_date: form.summary_date, emotion_score: form.emotion_score,
        market_view: form.market_view, lessons: form.lessons, improvement: form.improvement, tags,
      })
      toast({ title: '创建成功', description: '统计数据已自动聚合', variant: 'success' })
    }
    closeForm()
  } catch {
    toast({ title: '操作失败', description: summaryStore.error || '', variant: 'destructive' })
  } finally { formSubmitting.value = false }
}

async function handleDelete() {
  if (!deleteTarget.value) return
  try { await summaryStore.deleteSummary(deleteTarget.value.id); toast({ title: '已删除', variant: 'success' }) }
  catch { toast({ title: '删除失败', variant: 'destructive' }) }
  finally { deleteTarget.value = null }
}

function typeLabel(t: string) { return t === 'daily' ? '日总结' : t === 'weekly' ? '周总结' : '月总结' }

// --- AI Analysis ---
const { loading: aiLoading, result: aiResult, analyze: aiAnalyze, getConfig: aiGetConfig } = useAi()
const aiResultFor = ref<string | null>(null)

async function aiAnalyzeSummary(s: TradeSummary) {
  if (!aiGetConfig()) {
    toast({ title: '请先配置 AI', description: '前往设置页面填写 API Key', variant: 'destructive' })
    return
  }
  aiResultFor.value = s.id
  const err = await aiAnalyze([
    { role: 'system', content: '你是一位专业的交易复盘分析师。基于交易统计数据，提供简洁实用的复盘建议，包括：1) 交易表现评价 2) 风险管理建议 3) 可改进之处 4) 下一步行动计划。回复用中文，不超过500字。' },
    { role: 'user', content: `请分析以下${typeLabel(s.summary_type)}交易数据（日期: ${s.summary_date}）：\n总交易: ${s.total_trades}笔 (${s.win_trades}胜/${s.loss_trades}负)\n胜率: ${(s.win_rate * 100).toFixed(1)}%\n净盈亏: ${s.net_pnl >= 0 ? '+' : ''}${s.net_pnl.toFixed(0)}元\n盈亏比: ${s.profit_factor.toFixed(2)}\n最大盈利: ${s.max_profit.toFixed(0)}元 / 最大亏损: ${s.max_loss.toFixed(0)}元\n平均盈利: ${s.avg_profit.toFixed(0)}元 / 平均亏损: ${s.avg_loss.toFixed(0)}元\n情绪评分: ${s.emotion_score}/10` },
  ])
  if (err && !aiResult.value) {
    toast({ title: 'AI 分析失败', description: String(err), variant: 'destructive' })
  }
}

function parseTags(s: string): string[] { try { return JSON.parse(s || '[]') } catch { return [] } }

onMounted(async () => {
  await accountStore.fetchAccounts()
  if (!accountStore.currentAccount && accountStore.accounts.length > 0)
    await accountStore.selectAccount(accountStore.accounts[0].id)
  if (accountStore.currentAccount) await summaryStore.fetchSummaries(accountStore.currentAccount.id)
})

watch(() => accountStore.currentAccount, async (acc) => { if (acc) await summaryStore.fetchSummaries(acc.id) })
</script>

<template>
  <div class="max-w-6xl mx-auto space-y-5">
    <!-- Header -->
    <div class="flex items-center justify-between">
      <div>
        <h2 class="text-lg font-semibold text-foreground tracking-tight">交易总结</h2>
        <p class="text-sm text-muted-foreground mt-0.5">定期复盘，持续优化交易策略</p>
      </div>
      <Button size="sm" class="gap-2" :disabled="!currentAccount" @click="openCreateForm">
        <Plus class="w-4 h-4" /> 新建总结
      </Button>
    </div>

    <Card v-if="!currentAccount" class="border-dashed">
      <CardContent class="flex flex-col items-center justify-center py-10">
        <StickyNote class="w-10 h-10 text-muted-foreground/40 mb-3" />
        <p class="text-muted-foreground text-sm">请先在设置页面选择或创建一个账户</p>
      </CardContent>
    </Card>

    <template v-else>
      <!-- Type Filter -->
      <div class="flex items-center gap-1 rounded-lg bg-secondary/50 p-1">
        <button v-for="tab in typeTabs" :key="tab.key"
          :class="cn('inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 text-xs font-medium transition-colors cursor-pointer',
            summaryStore.filterType === tab.key ? 'bg-background text-foreground shadow-sm' : 'text-muted-foreground hover:text-foreground')"
          @click="summaryStore.filterType = tab.key">
          {{ tab.label }}
        </button>
      </div>

      <!-- Loading -->
      <div v-if="summaryStore.loading && summaryStore.filteredSummaries.length === 0" class="flex items-center justify-center py-16">
        <Loader2 class="w-6 h-6 animate-spin text-muted-foreground" />
      </div>

      <!-- Empty -->
      <Card v-else-if="summaryStore.filteredSummaries.length === 0" class="border-dashed">
        <CardContent class="flex flex-col items-center justify-center py-12">
          <StickyNote class="w-12 h-12 text-muted-foreground/30 mb-3" />
          <p class="text-muted-foreground text-sm">暂无交易总结</p>
          <p class="text-muted-foreground/60 text-xs mt-1">点击「新建总结」创建您的第一份复盘报告</p>
        </CardContent>
      </Card>

      <!-- Summary List -->
      <div v-else class="space-y-4">
        <Card v-for="s in summaryStore.filteredSummaries" :key="s.id" class="transition-all duration-200 hover:shadow-md group">
          <CardHeader class="pb-3">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div class="flex items-center justify-center w-9 h-9 rounded-lg bg-primary/10">
                  <Calendar class="w-4 h-4 text-primary" />
                </div>
                <div>
                  <CardTitle class="text-base">{{ s.summary_date }}</CardTitle>
                  <CardDescription class="mt-0.5">{{ typeLabel(s.summary_type) }}</CardDescription>
                </div>
              </div>
              <div class="flex items-center gap-1">
                <Button variant="outline" size="sm" class="h-8 gap-1.5 text-xs px-3" @click.stop="aiAnalyzeSummary(s)" :disabled="aiLoading">
                  <Loader2 v-if="aiLoading && aiResultFor === s.id" class="w-3.5 h-3.5 animate-spin" />
                  <Sparkles v-else class="w-3.5 h-3.5 text-primary" />AI 复盘
                </Button>
                <div class="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
                  <Button variant="ghost" size="icon" class="h-7 w-7" @click.stop="openEditForm(s)"><Pencil class="w-3.5 h-3.5" /></Button>
                <AlertDialog>
                  <AlertDialogTrigger as-child>
                    <Button variant="ghost" size="icon" class="h-7 w-7 hover:text-loss" @click.stop="deleteTarget = s"><Trash2 class="w-3.5 h-3.5" /></Button>
                  </AlertDialogTrigger>
                  <AlertDialogContent @click.stop>
                    <AlertDialogHeader>
                      <AlertDialogTitle>确认删除总结</AlertDialogTitle>
                      <AlertDialogDescription>确定要删除 {{ deleteTarget?.summary_date }} 的总结吗？</AlertDialogDescription>
                    </AlertDialogHeader>
                    <AlertDialogFooter>
                      <AlertDialogCancel @click="deleteTarget = null">取消</AlertDialogCancel>
                      <AlertDialogAction @click="handleDelete">确认删除</AlertDialogAction>
                    </AlertDialogFooter>
                  </AlertDialogContent>
                </AlertDialog>
                </div>
              </div>
            </div>
          </CardHeader>
          <CardContent class="pt-0 space-y-4">
            <!-- Stats Grid -->
            <div class="grid grid-cols-2 lg:grid-cols-5 gap-4">
              <div>
                <p class="text-[10px] uppercase tracking-wider text-muted-foreground mb-1">净盈亏</p>
                <p :class="cn('text-lg font-semibold', s.net_pnl >= 0 ? 'text-profit' : 'text-loss')">
                  {{ s.net_pnl >= 0 ? '+' : '' }}&yen;{{ formatCurrency(s.net_pnl) }}
                </p>
              </div>
              <div>
                <p class="text-[10px] uppercase tracking-wider text-muted-foreground mb-1">总交易</p>
                <p class="text-lg font-semibold text-foreground">{{ s.total_trades }}</p>
                <p class="text-[10px] text-muted-foreground">{{ s.win_trades }}胜 / {{ s.loss_trades }}负</p>
              </div>
              <div>
                <p class="text-[10px] uppercase tracking-wider text-muted-foreground mb-1">胜率</p>
                <p class="text-lg font-semibold text-foreground">{{ formatPercent(s.win_rate) }}</p>
              </div>
              <div>
                <p class="text-[10px] uppercase tracking-wider text-muted-foreground mb-1">盈亏比</p>
                <p class="text-lg font-semibold text-foreground">{{ s.profit_factor.toFixed(2) }}</p>
              </div>
              <div>
                <p class="text-[10px] uppercase tracking-wider text-muted-foreground mb-1">情绪评分</p>
                <p class="text-lg font-semibold text-foreground">{{ s.emotion_score }}/10</p>
              </div>
            </div>

            <!-- Extra stats row -->
            <div class="grid grid-cols-4 gap-4 text-xs text-muted-foreground pt-2 border-t border-border/50">
              <div class="flex items-center gap-1.5"><TrendingUp class="w-3.5 h-3.5 text-profit" />最大盈利: &yen;{{ formatCurrency(s.max_profit) }}</div>
              <div class="flex items-center gap-1.5"><TrendingDown class="w-3.5 h-3.5 text-loss" />最大亏损: &yen;{{ formatCurrency(s.max_loss) }}</div>
              <div class="flex items-center gap-1.5"><BarChart3 class="w-3.5 h-3.5" />平均盈利: &yen;{{ formatCurrency(s.avg_profit) }}</div>
              <div class="flex items-center gap-1.5"><BarChart3 class="w-3.5 h-3.5" />平均亏损: &yen;{{ formatCurrency(s.avg_loss) }}</div>
            </div>

            <!-- Text content -->
            <div v-if="s.market_view || s.lessons || s.improvement" class="space-y-2 pt-2 border-t border-border/50">
              <p v-if="s.market_view"><span class="text-xs font-medium text-muted-foreground">市场观点: </span><span class="text-xs text-foreground">{{ s.market_view }}</span></p>
              <p v-if="s.lessons"><span class="text-xs font-medium text-muted-foreground">经验教训: </span><span class="text-xs text-foreground">{{ s.lessons }}</span></p>
              <p v-if="s.improvement"><span class="text-xs font-medium text-muted-foreground">改进计划: </span><span class="text-xs text-foreground">{{ s.improvement }}</span></p>
            </div>

            <div v-if="parseTags(s.tags).length" class="flex flex-wrap gap-1">
              <Badge v-for="tag in parseTags(s.tags)" :key="tag" variant="secondary" class="text-[10px] h-5 px-1.5 gap-1">
                <Tag class="w-2.5 h-2.5" />{{ tag }}
              </Badge>
            </div>

            <!-- AI Analysis Result -->
            <div v-if="aiResultFor === s.id && (aiLoading || aiResult)" class="mt-3 pt-3 border-t border-primary/20 rounded-lg bg-primary/5 p-3">
              <div class="flex items-center gap-1.5 mb-2">
                <Sparkles class="w-3.5 h-3.5 text-primary" />
                <span class="text-xs font-medium text-primary">AI 复盘分析</span>
                <Loader2 v-if="aiLoading" class="w-3 h-3 animate-spin text-primary ml-auto" />
              </div>
              <div v-if="aiLoading" class="text-xs text-muted-foreground">正在分析中...</div>
              <div v-else class="text-xs text-foreground/80 leading-relaxed ai-content prose prose-sm max-w-none" v-html="renderMarkdown(aiResult)" />
            </div>
          </CardContent>
        </Card>
      </div>
    </template>

    <!-- Form Overlay -->
    <Transition enter-active-class="transition-all duration-300" enter-from-class="opacity-0" enter-to-class="opacity-100" leave-active-class="transition-all duration-200" leave-from-class="opacity-100" leave-to-class="opacity-0">
      <div v-if="showForm" class="fixed inset-0 z-40 bg-black/30 backdrop-blur-sm" @click="closeForm" />
    </Transition>
    <Transition enter-active-class="transition-all duration-300 ease-out" enter-from-class="opacity-0 translate-y-4" enter-to-class="opacity-100 translate-y-0" leave-active-class="transition-all duration-200 ease-in" leave-from-class="opacity-100 translate-y-0" leave-to-class="opacity-0 translate-y-4">
      <div v-if="showForm" class="fixed inset-x-0 bottom-0 z-50 mx-auto max-w-2xl px-4 pb-6">
        <Card class="shadow-xl border-ring/20 max-h-[85vh] flex flex-col">
          <CardHeader class="pb-3 border-b border-border/50 shrink-0">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div class="flex items-center justify-center w-9 h-9 rounded-lg bg-primary/10"><StickyNote class="w-4 h-4 text-primary" /></div>
                <div>
                  <CardTitle class="text-base">{{ isEditing ? '编辑交易总结' : '新建交易总结' }}</CardTitle>
                  <CardDescription class="mt-0.5">{{ isEditing ? '修改主观评价内容' : '统计数据将自动聚合' }}</CardDescription>
                </div>
              </div>
              <Button variant="ghost" size="icon" class="h-8 w-8" @click="closeForm"><X class="w-4 h-4" /></Button>
            </div>
          </CardHeader>
          <div class="flex-1 overflow-y-auto p-6 space-y-4">
            <div v-if="!isEditing" class="grid grid-cols-2 gap-4">
              <div class="space-y-2"><Label class="text-sm font-medium">总结类型</Label>
                <SelectNative v-model="form.summary_type">
                  <option value="daily">日总结</option>
                  <option value="weekly">周总结</option>
                  <option value="monthly">月总结</option>
                </SelectNative>
              </div>
              <div class="space-y-2"><Label class="text-sm font-medium">日期</Label><Input v-model="form.summary_date" type="date" /></div>
            </div>
            <div class="space-y-2"><Label class="text-sm font-medium">情绪评分 (1-10)</Label><Input v-model.number="form.emotion_score" type="number" min="1" max="10" step="1" /></div>
            <div class="space-y-2"><Label class="text-sm font-medium">市场观点</Label><Textarea v-model="form.market_view" placeholder="对当前市场的看法..." class="min-h-[60px]" /></div>
            <div class="space-y-2"><Label class="text-sm font-medium">经验教训</Label><Textarea v-model="form.lessons" placeholder="这次交易中学到了什么..." class="min-h-[60px]" /></div>
            <div class="space-y-2"><Label class="text-sm font-medium">改进计划</Label><Textarea v-model="form.improvement" placeholder="下一步如何改进..." class="min-h-[60px]" /></div>
            <div class="space-y-2"><Label class="text-sm font-medium">标签</Label><Input v-model="form.tagsInput" placeholder="逗号分隔" /></div>
          </div>
          <div class="flex items-center justify-end gap-2 px-6 py-4 border-t border-border/50 shrink-0">
            <Button variant="outline" size="sm" @click="closeForm">取消</Button>
            <Button size="sm" class="gap-2" :disabled="formSubmitting" @click="handleSubmit">
              <Loader2 v-if="formSubmitting" class="w-4 h-4 animate-spin" />{{ isEditing ? '保存' : '创建总结' }}
            </Button>
          </div>
        </Card>
      </div>
    </Transition>
  </div>
</template>
