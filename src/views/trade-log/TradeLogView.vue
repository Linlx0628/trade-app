<script setup lang="ts">
import { ref, reactive, computed, onMounted, watch } from 'vue'
import {
  BookOpen, ArrowUpRight, ArrowDownRight, Loader2, Trash2,
  Pencil, X, Clock, CheckCircle2, Tag,
} from 'lucide-vue-next'
import { useTradeLogStore } from '@/stores/trade-log'
import { useAccountStore } from '@/stores/account'
import { useToast } from '@/components/ui/toast'
import { Card, CardContent } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Badge } from '@/components/ui/badge'
import { Textarea } from '@/components/ui/textarea'
import { SelectNative } from '@/components/ui/select'
import {
  AlertDialog, AlertDialogTrigger, AlertDialogContent, AlertDialogHeader,
  AlertDialogFooter, AlertDialogTitle, AlertDialogDescription, AlertDialogAction, AlertDialogCancel,
} from '@/components/ui/alert-dialog'
import { cn } from '@/lib/utils'
import type { TradeLog, TradeDirection, TradeLogStatus, MarketType } from '@/types/common'

const logStore = useTradeLogStore()
const accountStore = useAccountStore()
const { toast } = useToast()

const showForm = ref(false)
const editingLog = ref<TradeLog | null>(null)
const deleteTarget = ref<TradeLog | null>(null)
const formSubmitting = ref(false)

const form = reactive({
  symbol: '', name: '', direction: 'long' as TradeDirection,
  market_type: 'futures' as MarketType, entry_price: 0, exit_price: 0,
  stop_loss: 0, lots: 0, commission: 0, pnl: 0, pnl_points: 0,
  status: 'open' as TradeLogStatus, entry_time: '', exit_time: '',
  emotion_before: '', confidence: 5, notes: '', tagsInput: '',
})

const currentAccount = computed(() => accountStore.currentAccount)
const isEditing = computed(() => editingLog.value !== null)

const statusTabs = computed(() => {
  const c = logStore.statusCounts
  return [
    { key: 'all' as const, label: '全部', count: c.all },
    { key: 'open' as const, label: '持仓中', count: c.open || 0 },
    { key: 'closed' as const, label: '已平仓', count: c.closed || 0 },
  ]
})

function formatCurrency(v: number) {
  return new Intl.NumberFormat('zh-CN', { minimumFractionDigits: 0, maximumFractionDigits: 2 }).format(v)
}

function resetForm() {
  Object.assign(form, {
    symbol: '', name: '', direction: 'long', market_type: 'futures',
    entry_price: 0, exit_price: 0, stop_loss: 0, lots: 0, commission: 0,
    pnl: 0, pnl_points: 0, status: 'open', entry_time: '', exit_time: '',
    emotion_before: '', confidence: 5, notes: '', tagsInput: '',
  })
  editingLog.value = null
}

function openEditForm(log: TradeLog) {
  editingLog.value = log
  Object.assign(form, {
    symbol: log.symbol, name: log.name, direction: log.direction as TradeDirection,
    market_type: log.market_type as MarketType, entry_price: log.entry_price,
    exit_price: log.exit_price, stop_loss: log.stop_loss, lots: log.lots,
    commission: log.commission, pnl: log.pnl, pnl_points: log.pnl_points,
    status: log.status as TradeLogStatus, entry_time: log.entry_time ? log.entry_time.slice(0, 16) : '',
    exit_time: log.exit_time ? log.exit_time.slice(0, 16) : '',
    emotion_before: log.emotion_before, confidence: log.confidence, notes: log.notes,
    tagsInput: (() => { try { return JSON.parse(log.tags || '[]').join(', ') } catch { return '' } })(),
  })
  showForm.value = true
}

function closeForm() { showForm.value = false; resetForm() }

async function handleSubmit() {
  if (!currentAccount.value) { toast({ title: '请先选择账户', variant: 'destructive' }); return }
  formSubmitting.value = true
  const tags = form.tagsInput.split(/[,，]/).map(t => t.trim()).filter(Boolean)

  if (form.exit_price > 0 && form.entry_price > 0 && form.lots > 0) {
    const dir = form.direction === 'long' ? 1 : -1
    form.pnl_points = (form.exit_price - form.entry_price) * dir
    form.pnl = form.pnl_points * form.lots * (currentAccount.value?.point_value || 10) - form.commission
  }

  try {
    if (isEditing.value) {
      await logStore.updateLog({
        id: editingLog.value!.id, symbol: form.symbol, name: form.name,
        direction: form.direction, entry_price: form.entry_price, exit_price: form.exit_price,
        stop_loss: form.stop_loss, lots: form.lots, commission: form.commission,
        pnl: form.pnl, pnl_points: form.pnl_points, status: form.status,
        exit_time: form.exit_time, notes: form.notes, tags,
        emotion_after: form.status === 'closed' ? form.emotion_before : undefined,
        confidence: form.confidence,
      })
      toast({ title: '更新成功', variant: 'success' })
    } else {
      await logStore.createLog({
        account_id: currentAccount.value.id, symbol: form.symbol, name: form.name,
        direction: form.direction, market_type: form.market_type,
        entry_price: form.entry_price, exit_price: form.exit_price,
        stop_loss: form.stop_loss, lots: form.lots, commission: form.commission,
        pnl: form.pnl, pnl_points: form.pnl_points, status: form.status,
        entry_time: form.entry_time, exit_time: form.exit_time,
        notes: form.notes, tags, emotion_before: form.emotion_before,
        confidence: form.confidence,
      })
      toast({ title: '创建成功', variant: 'success' })
    }
    closeForm()
  } catch {
    toast({ title: '操作失败', description: logStore.error || '', variant: 'destructive' })
  } finally { formSubmitting.value = false }
}

async function handleDelete() {
  if (!deleteTarget.value) return
  try { await logStore.deleteLog(deleteTarget.value.id); toast({ title: '已删除', variant: 'success' }) }
  catch { toast({ title: '删除失败', variant: 'destructive' }) }
  finally { deleteTarget.value = null }
}

async function handleClosePosition(log: TradeLog) {
  try {
    await logStore.updateLog({ id: log.id, status: 'closed', exit_time: new Date().toISOString().slice(0, 16) })
    // 联动：同步更新关联计划状态为 completed
    if (log.plan_id) {
      const { tradePlanApi } = await import('@/lib/tauri')
      await tradePlanApi.update({ id: log.plan_id, status: 'completed' })
    }
    toast({ title: '已平仓', description: log.plan_id ? '关联计划已标记完成' : '', variant: 'success' })
  } catch { toast({ title: '操作失败', variant: 'destructive' }) }
}

function parseTags(s: string): string[] { try { return JSON.parse(s || '[]') } catch { return [] } }

onMounted(async () => {
  await accountStore.fetchAccounts()
  if (!accountStore.currentAccount && accountStore.accounts.length > 0)
    await accountStore.selectAccount(accountStore.accounts[0].id)
  if (accountStore.currentAccount) await logStore.fetchLogs(accountStore.currentAccount.id)
})

watch(() => accountStore.currentAccount, async (acc) => { if (acc) await logStore.fetchLogs(acc.id) })
</script>

<template>
  <div class="max-w-6xl mx-auto space-y-5">
    <!-- Header -->
    <div class="flex items-center justify-between">
      <div>
        <h2 class="text-lg font-semibold text-foreground tracking-tight">交易日志</h2>
        <p class="text-sm text-muted-foreground mt-0.5">记录每笔交易的执行情况与盈亏结果</p>
      </div>
      <div class="flex items-center gap-3">
        <div v-if="logStore.totalPnl !== 0" class="text-sm">
          <span class="text-muted-foreground">总盈亏: </span>
          <span :class="cn('font-semibold', logStore.totalPnl >= 0 ? 'text-profit' : 'text-loss')">
            {{ logStore.totalPnl >= 0 ? '+' : '' }}&yen;{{ formatCurrency(logStore.totalPnl) }}
          </span>
        </div>
        <p class="text-xs text-muted-foreground">日志由交易计划自动生成</p>
      </div>
    </div>

    <Card v-if="!currentAccount" class="border-dashed">
      <CardContent class="flex flex-col items-center justify-center py-10">
        <BookOpen class="w-10 h-10 text-muted-foreground/40 mb-3" />
        <p class="text-muted-foreground text-sm">请先在设置页面选择或创建一个账户</p>
      </CardContent>
    </Card>

    <template v-else>
      <!-- Status Filter -->
      <div class="flex items-center gap-1 rounded-lg bg-secondary/50 p-1">
        <button v-for="tab in statusTabs" :key="tab.key"
          :class="cn('inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 text-xs font-medium transition-colors cursor-pointer',
            logStore.filterStatus === tab.key ? 'bg-background text-foreground shadow-sm' : 'text-muted-foreground hover:text-foreground')"
          @click="logStore.filterStatus = tab.key">
          {{ tab.label }}
          <span :class="cn('inline-flex items-center justify-center h-4 min-w-[1rem] px-1 rounded-full text-[10px] font-semibold',
            logStore.filterStatus === tab.key ? 'bg-primary/10 text-primary' : 'bg-muted text-muted-foreground')">
            {{ tab.count }}
          </span>
        </button>
      </div>

      <!-- Loading -->
      <div v-if="logStore.loading && logStore.filteredLogs.length === 0" class="flex items-center justify-center py-16">
        <Loader2 class="w-6 h-6 animate-spin text-muted-foreground" />
      </div>

      <!-- Empty -->
      <Card v-else-if="logStore.filteredLogs.length === 0" class="border-dashed">
        <CardContent class="flex flex-col items-center justify-center py-12">
          <BookOpen class="w-12 h-12 text-muted-foreground/30 mb-3" />
          <p class="text-muted-foreground text-sm">暂无交易日志</p>
          <p class="text-muted-foreground/60 text-xs mt-1">在交易计划中执行计划后，日志会自动生成</p>
        </CardContent>
      </Card>

      <!-- Log List -->
      <div v-else class="space-y-3">
        <Card v-for="log in logStore.filteredLogs" :key="log.id" class="transition-all duration-200 hover:shadow-md group">
          <CardContent class="p-4">
            <div class="flex items-start justify-between gap-4">
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2.5 mb-2">
                  <div :class="cn('flex items-center justify-center w-8 h-8 rounded-lg shrink-0',
                    log.direction === 'long' ? 'bg-profit/10 text-profit' : 'bg-loss/10 text-loss')">
                    <ArrowUpRight v-if="log.direction === 'long'" class="w-4 h-4" />
                    <ArrowDownRight v-else class="w-4 h-4" />
                  </div>
                  <div class="min-w-0">
                    <div class="flex items-center gap-2">
                      <span class="font-semibold text-foreground text-sm">{{ log.symbol }}</span>
                      <span v-if="log.name" class="text-muted-foreground text-xs">{{ log.name }}</span>
                    </div>
                    <div class="flex items-center gap-2 mt-0.5 text-xs text-muted-foreground">
                      <span>{{ log.direction === 'long' ? '做多' : '做空' }}</span>
                      <span class="text-muted-foreground/40">|</span>
                      <span>{{ log.market_type === 'futures' ? '期货' : '股票' }}</span>
                      <span v-if="log.entry_time" class="text-muted-foreground/40">|</span>
                      <span v-if="log.entry_time">{{ log.entry_time.slice(0, 10) }}</span>
                    </div>
                  </div>
                </div>
                <div class="grid grid-cols-4 gap-3 mt-3">
                  <div><p class="text-[10px] uppercase tracking-wider text-muted-foreground">入场</p><p class="text-sm font-medium text-foreground">{{ log.entry_price }}</p></div>
                  <div><p class="text-[10px] uppercase tracking-wider text-muted-foreground">出场</p><p class="text-sm font-medium text-foreground">{{ log.exit_price || '-' }}</p></div>
                  <div><p class="text-[10px] uppercase tracking-wider text-muted-foreground">手数</p><p class="text-sm font-medium text-foreground">{{ log.lots }}</p></div>
                  <div><p class="text-[10px] uppercase tracking-wider text-muted-foreground">盈亏</p>
                    <p :class="cn('text-sm font-semibold', log.pnl > 0 ? 'text-profit' : log.pnl < 0 ? 'text-loss' : 'text-foreground')">
                      {{ log.pnl >= 0 ? '+' : '' }}&yen;{{ formatCurrency(log.pnl) }}
                    </p>
                  </div>
                </div>
                <div v-if="parseTags(log.tags).length" class="flex flex-wrap gap-1 mt-2.5">
                  <Badge v-for="tag in parseTags(log.tags)" :key="tag" variant="secondary" class="text-[10px] h-5 px-1.5 gap-1">
                    <Tag class="w-2.5 h-2.5" />{{ tag }}
                  </Badge>
                </div>
              </div>
              <div class="flex flex-col items-end gap-2 shrink-0">
                <div class="flex items-center gap-1.5">
                  <component :is="log.status === 'open' ? Clock : CheckCircle2"
                    :class="cn('w-3.5 h-3.5', log.status === 'open' ? 'text-primary' : 'text-profit')" />
                  <span :class="cn('text-xs font-medium', log.status === 'open' ? 'text-primary' : 'text-profit')">
                    {{ log.status === 'open' ? '持仓中' : '已平仓' }}
                  </span>
                </div>
                <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                  <Button v-if="log.status === 'open'" variant="outline" size="sm" class="h-6 text-[10px] px-2"
                    @click.stop="handleClosePosition(log)">平仓</Button>
                  <Button variant="ghost" size="icon" class="h-7 w-7" @click.stop="openEditForm(log)"><Pencil class="w-3.5 h-3.5" /></Button>
                  <AlertDialog>
                    <AlertDialogTrigger as-child>
                      <Button variant="ghost" size="icon" class="h-7 w-7 hover:text-loss" @click.stop="deleteTarget = log"><Trash2 class="w-3.5 h-3.5" /></Button>
                    </AlertDialogTrigger>
                    <AlertDialogContent @click.stop>
                      <AlertDialogHeader>
                        <AlertDialogTitle>确认删除交易日志</AlertDialogTitle>
                        <AlertDialogDescription>确定要删除「{{ deleteTarget?.symbol }}」的交易日志吗？</AlertDialogDescription>
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
            <p v-if="log.notes" class="mt-3 pt-3 border-t border-border/50 text-xs text-muted-foreground line-clamp-2">{{ log.notes }}</p>
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
                <div class="flex items-center justify-center w-9 h-9 rounded-lg bg-primary/10"><BookOpen class="w-4 h-4 text-primary" /></div>
                <div><CardTitle class="text-base">{{ isEditing ? '编辑交易日志' : '新建交易日志' }}</CardTitle></div>
              </div>
              <Button variant="ghost" size="icon" class="h-8 w-8" @click="closeForm"><X class="w-4 h-4" /></Button>
            </div>
          </CardHeader>
          <div class="flex-1 overflow-y-auto p-6 space-y-4">
            <div class="grid grid-cols-3 gap-4">
              <div class="space-y-2"><Label class="text-sm font-medium">品种 <span class="text-loss">*</span></Label><Input v-model="form.symbol" placeholder="IF2506" /></div>
              <div class="space-y-2"><Label class="text-sm font-medium">方向</Label>
                <div class="grid grid-cols-2 gap-2">
                  <button :class="cn('flex items-center justify-center gap-1 rounded-lg border px-3 py-2 text-xs font-medium cursor-pointer transition-all', form.direction === 'long' ? 'border-profit/40 bg-profit/10 text-profit' : 'border-input text-muted-foreground')" @click="form.direction = 'long'"><ArrowUpRight class="w-3 h-3" />多</button>
                  <button :class="cn('flex items-center justify-center gap-1 rounded-lg border px-3 py-2 text-xs font-medium cursor-pointer transition-all', form.direction === 'short' ? 'border-loss/40 bg-loss/10 text-loss' : 'border-input text-muted-foreground')" @click="form.direction = 'short'"><ArrowDownRight class="w-3 h-3" />空</button>
                </div>
              </div>
              <div class="space-y-2"><Label class="text-sm font-medium">状态</Label>
                <SelectNative v-model="form.status"><option value="open">持仓中</option><option value="closed">已平仓</option></SelectNative>
              </div>
            </div>
            <div class="grid grid-cols-3 gap-4">
              <div class="space-y-2"><Label class="text-sm font-medium">入场价</Label><Input v-model.number="form.entry_price" type="number" step="any" min="0" /></div>
              <div class="space-y-2"><Label class="text-sm font-medium">出场价</Label><Input v-model.number="form.exit_price" type="number" step="any" min="0" /></div>
              <div class="space-y-2"><Label class="text-sm font-medium">手数</Label><Input v-model.number="form.lots" type="number" step="any" min="0" /></div>
            </div>
            <div class="grid grid-cols-3 gap-4">
              <div class="space-y-2"><Label class="text-sm font-medium">止损价</Label><Input v-model.number="form.stop_loss" type="number" step="any" min="0" /></div>
              <div class="space-y-2"><Label class="text-sm font-medium">手续费</Label><Input v-model.number="form.commission" type="number" step="any" min="0" /></div>
              <div class="space-y-2"><Label class="text-sm font-medium">信心指数 (1-10)</Label><Input v-model.number="form.confidence" type="number" min="1" max="10" step="1" /></div>
            </div>
            <div class="grid grid-cols-2 gap-4">
              <div class="space-y-2"><Label class="text-sm font-medium">入场时间</Label><Input v-model="form.entry_time" type="datetime-local" /></div>
              <div class="space-y-2"><Label class="text-sm font-medium">出场时间</Label><Input v-model="form.exit_time" type="datetime-local" /></div>
            </div>
            <div class="space-y-2"><Label class="text-sm font-medium">备注</Label><Textarea v-model="form.notes" placeholder="交易心得、执行情况..." class="min-h-[60px]" /></div>
            <div class="space-y-2"><Label class="text-sm font-medium">标签</Label><Input v-model="form.tagsInput" placeholder="逗号分隔" /></div>
          </div>
          <div class="flex items-center justify-end gap-2 px-6 py-4 border-t border-border/50 shrink-0">
            <Button variant="outline" size="sm" @click="closeForm">取消</Button>
            <Button size="sm" class="gap-2" :disabled="formSubmitting" @click="handleSubmit">
              <Loader2 v-if="formSubmitting" class="w-4 h-4 animate-spin" />{{ isEditing ? '保存' : '创建' }}
            </Button>
          </div>
        </Card>
      </div>
    </Transition>
  </div>
</template>
