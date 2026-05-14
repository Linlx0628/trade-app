<script setup lang="ts">
import { ref, reactive, computed, onMounted, watch } from 'vue'
import {
  Plus,
  FileText,
  ArrowUpRight,
  ArrowDownRight,
  Loader2,
  Trash2,
  Pencil,
  ChevronRight,
  Target,
  ShieldAlert,
  TrendingUp,
  Calculator,
  X,
  Clock,
  CheckCircle2,
  Ban,
  PlayCircle,
  Tag,
  Sparkles,
} from 'lucide-vue-next'
import { useTradePlanStore } from '@/stores/trade-plan'
import { useAccountStore } from '@/stores/account'
import { useToast } from '@/components/ui/toast'
import { Card, CardHeader, CardTitle, CardDescription, CardContent } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Badge } from '@/components/ui/badge'
import { Textarea } from '@/components/ui/textarea'
import { SelectNative } from '@/components/ui/select'
import {
  AlertDialog,
  AlertDialogTrigger,
  AlertDialogContent,
  AlertDialogHeader,
  AlertDialogFooter,
  AlertDialogTitle,
  AlertDialogDescription,
  AlertDialogAction,
  AlertDialogCancel,
} from '@/components/ui/alert-dialog'
import { cn } from '@/lib/utils'
import { useAi } from '@/composables/useAi'
import type { TradePlan, TradeDirection, TradePlanStatus, MarketType } from '@/types/common'

const tradePlanStore = useTradePlanStore()
const accountStore = useAccountStore()
const { toast } = useToast()

// --- Form State ---
const showForm = ref(false)
const editingPlan = ref<TradePlan | null>(null)
const deleteTarget = ref<TradePlan | null>(null)
const formSubmitting = ref(false)
const currentStep = ref(0)

const form = reactive({
  symbol: '',
  name: '',
  direction: 'long' as TradeDirection,
  market_type: 'futures' as MarketType,
  entry_price: 0,
  stop_loss: 0,
  take_profit: 0,
  lots: 0,
  strategy: '',
  notes: '',
  tagsInput: '',
  planned_at: '',
})

const formErrors = reactive({
  symbol: '',
  entry_price: '',
  stop_loss: '',
  take_profit: '',
})

const steps = [
  { label: '基本信息', desc: '品种与方向' },
  { label: '价格参数', desc: '入场/止损/止盈' },
  { label: '策略备注', desc: '策略与标签' },
]

// --- Computed ---
const isEditing = computed(() => editingPlan.value !== null)
const currentAccount = computed(() => accountStore.currentAccount)

const statusTabs = computed(() => {
  const counts = tradePlanStore.statusCounts
  return [
    { key: 'all' as const, label: '全部', count: counts.all },
    { key: 'planned' as const, label: '计划中', count: counts.planned || 0 },
    { key: 'executing' as const, label: '执行中', count: counts.executing || 0 },
    { key: 'completed' as const, label: '已完成', count: counts.completed || 0 },
    { key: 'cancelled' as const, label: '已取消', count: counts.cancelled || 0 },
  ]
})

const displayedPlans = computed(() => tradePlanStore.filteredPlans)

const calcResult = computed(() => tradePlanStore.calculation)

const canProceed = computed(() => {
  if (currentStep.value === 0) return form.symbol.trim() !== ''
  if (currentStep.value === 1) return form.entry_price > 0 && form.stop_loss > 0 && form.take_profit > 0
  return true
})

// --- Methods ---
function formatCurrency(value: number): string {
  return new Intl.NumberFormat('zh-CN', { minimumFractionDigits: 0, maximumFractionDigits: 2 }).format(value)
}

function formatPercent(value: number): string {
  return `${(value * 100).toFixed(2)}%`
}

function resetForm() {
  form.symbol = ''
  form.name = ''
  form.direction = 'long'
  form.market_type = 'futures'
  form.entry_price = 0
  form.stop_loss = 0
  form.take_profit = 0
  form.lots = 0
  form.strategy = ''
  form.notes = ''
  form.tagsInput = ''
  form.planned_at = ''
  formErrors.symbol = ''
  formErrors.entry_price = ''
  formErrors.stop_loss = ''
  formErrors.take_profit = ''
  editingPlan.value = null
  currentStep.value = 0
  tradePlanStore.clearCalculation()
}

function openCreateForm() {
  resetForm()
  showForm.value = true
}

function openEditForm(plan: TradePlan) {
  editingPlan.value = plan
  form.symbol = plan.symbol
  form.name = plan.name
  form.direction = plan.direction as TradeDirection
  form.market_type = plan.market_type as MarketType
  form.entry_price = plan.entry_price
  form.stop_loss = plan.stop_loss
  form.take_profit = plan.take_profit
  form.lots = plan.lots
  form.strategy = plan.strategy
  form.notes = plan.notes
  try {
    form.tagsInput = JSON.parse(plan.tags || '[]').join(', ')
  } catch {
    form.tagsInput = ''
  }
  form.planned_at = plan.planned_at
  currentStep.value = 0
  showForm.value = true
}

function closeForm() {
  showForm.value = false
  resetForm()
}

function nextStep() {
  if (currentStep.value === 0 && !form.symbol.trim()) {
    formErrors.symbol = '请输入交易品种'
    return
  }
  if (currentStep.value < steps.length - 1) {
    currentStep.value++
  }
}

function prevStep() {
  if (currentStep.value > 0) currentStep.value--
}

async function runCalculation() {
  if (!currentAccount.value || form.entry_price <= 0) return
  try {
    await tradePlanStore.calculate(
      currentAccount.value.id,
      form.entry_price,
      form.stop_loss,
      form.take_profit,
    )
    if (calcResult.value) {
      form.lots = calcResult.value.suggested_lots
    }
  } catch {
    // silent
  }
}

watch(() => [form.entry_price, form.stop_loss, form.take_profit], () => {
  if (form.entry_price > 0 && form.stop_loss > 0 && form.take_profit > 0) {
    runCalculation()
  }
})

async function handleSubmit() {
  if (!currentAccount.value) {
    toast({ title: '请先选择账户', variant: 'destructive' })
    return
  }

  formSubmitting.value = true
  const tags = form.tagsInput
    .split(/[,，]/)
    .map((t) => t.trim())
    .filter(Boolean)

  try {
    if (isEditing.value) {
      await tradePlanStore.updatePlan({
        id: editingPlan.value!.id,
        symbol: form.symbol,
        name: form.name,
        direction: form.direction,
        market_type: form.market_type,
        entry_price: form.entry_price,
        stop_loss: form.stop_loss,
        take_profit: form.take_profit,
        lots: form.lots,
        strategy: form.strategy,
        notes: form.notes,
        tags,
        planned_at: form.planned_at,
      })
      toast({ title: '更新成功', description: `交易计划 "${form.symbol}" 已更新`, variant: 'success' })
    } else {
      await tradePlanStore.createPlan({
        account_id: currentAccount.value.id,
        symbol: form.symbol,
        name: form.name,
        direction: form.direction,
        market_type: form.market_type,
        entry_price: form.entry_price,
        stop_loss: form.stop_loss,
        take_profit: form.take_profit,
        lots: form.lots,
        strategy: form.strategy,
        notes: form.notes,
        tags,
        planned_at: form.planned_at,
      })
      toast({ title: '创建成功', description: `交易计划 "${form.symbol}" 已创建`, variant: 'success' })
    }
    closeForm()
  } catch {
    toast({
      title: isEditing.value ? '更新失败' : '创建失败',
      description: tradePlanStore.error || '操作失败，请重试',
      variant: 'destructive',
    })
  } finally {
    formSubmitting.value = false
  }
}

async function handleDelete() {
  if (!deleteTarget.value) return
  try {
    await tradePlanStore.deletePlan(deleteTarget.value.id)
    toast({ title: '已删除', description: `计划 "${deleteTarget.value.symbol}" 已删除`, variant: 'success' })
  } catch {
    toast({ title: '删除失败', variant: 'destructive' })
  } finally {
    deleteTarget.value = null
  }
}

async function handleStatusChange(plan: TradePlan, status: TradePlanStatus) {
  try {
    if (status === 'executing') {
      // 联动：执行计划时自动创建交易日志
      const { tradePlanApi } = await import('@/lib/tauri')
      await tradePlanApi.execute(plan.id)
      await tradePlanStore.fetchPlans(plan.account_id)
      toast({ title: '已执行', description: `交易日志已自动创建`, variant: 'success' })
    } else {
      await tradePlanStore.updatePlan({ id: plan.id, status })
      toast({ title: '状态已更新', variant: 'success' })
    }
  } catch {
    toast({ title: '更新失败', variant: 'destructive' })
  }
}

function getStatusIcon(status: TradePlanStatus) {
  switch (status) {
    case 'planned': return Clock
    case 'executing': return PlayCircle
    case 'completed': return CheckCircle2
    case 'cancelled': return Ban
  }
}

function getStatusColor(status: TradePlanStatus) {
  switch (status) {
    case 'planned': return 'text-muted-foreground'
    case 'executing': return 'text-primary'
    case 'completed': return 'text-profit'
    case 'cancelled': return 'text-destructive'
  }
}

function getStatusLabel(status: TradePlanStatus) {
  switch (status) {
    case 'planned': return '计划中'
    case 'executing': return '执行中'
    case 'completed': return '已完成'
    case 'cancelled': return '已取消'
  }
}

function parseTags(tagsStr: string): string[] {
  try {
    return JSON.parse(tagsStr || '[]')
  } catch {
    return []
  }
}

// --- AI Strategy ---
const ai = useAi()
const aiPlanId = ref<string | null>(null)

async function aiAnalyzePlan(plan: TradePlan) {
  aiPlanId.value = plan.id
  await ai.analyze([
    { role: 'system', content: '你是一位专业的期货/股票交易策略顾问。基于交易计划参数，提供简洁的风险评估和策略建议，包括：1) 风险评估 2) 入场时机分析 3) 盈亏比评价 4) 关键注意事项。回复用中文，不超过400字。' },
    { role: 'user', content: `请分析以下交易计划：\n品种: ${plan.symbol} (${plan.name || plan.symbol})\n方向: ${plan.direction === 'long' ? '做多' : '做空'}\n入场: ${plan.entry_price} / 止损: ${plan.stop_loss} / 止盈: ${plan.take_profit}\n手数: ${plan.lots}\n市场: ${plan.market_type === 'futures' ? '期货' : '股票'}\n策略: ${plan.strategy || '未填写'}` },
  ])
  if (ai.error.value) toast({ title: 'AI 分析失败', description: ai.error.value, variant: 'destructive' })
}

// --- Lifecycle ---
onMounted(async () => {
  await accountStore.fetchAccounts()
  if (!accountStore.currentAccount && accountStore.accounts.length > 0) {
    await accountStore.selectAccount(accountStore.accounts[0].id)
  }
  if (accountStore.currentAccount) {
    await tradePlanStore.fetchPlans(accountStore.currentAccount.id)
  }
})

watch(() => accountStore.currentAccount, async (acc) => {
  if (acc) {
    await tradePlanStore.fetchPlans(acc.id)
  }
})
</script>

<template>
  <div class="max-w-6xl mx-auto space-y-5">
    <!-- Header -->
    <div class="flex items-center justify-between">
      <div>
        <h2 class="text-lg font-semibold text-foreground tracking-tight">交易计划</h2>
        <p class="text-sm text-muted-foreground mt-0.5">制定和管理您的交易计划，严格执行风险管理</p>
      </div>
      <Button size="sm" class="gap-2" :disabled="!currentAccount" @click="openCreateForm">
        <Plus class="w-4 h-4" />
        新建计划
      </Button>
    </div>

    <!-- No account hint -->
    <Card v-if="!currentAccount" class="border-dashed">
      <CardContent class="flex flex-col items-center justify-center py-10">
        <FileText class="w-10 h-10 text-muted-foreground/40 mb-3" />
        <p class="text-muted-foreground text-sm">请先在设置页面选择或创建一个账户</p>
      </CardContent>
    </Card>

    <template v-else>
      <!-- Status Filter Tabs -->
      <div class="flex items-center gap-1 rounded-lg bg-secondary/50 p-1">
        <button
          v-for="tab in statusTabs"
          :key="tab.key"
          :class="cn(
            'inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 text-xs font-medium transition-colors cursor-pointer',
            tradePlanStore.filterStatus === tab.key
              ? 'bg-background text-foreground shadow-sm'
              : 'text-muted-foreground hover:text-foreground',
          )"
          @click="tradePlanStore.filterStatus = tab.key"
        >
          {{ tab.label }}
          <span
            :class="cn(
              'inline-flex items-center justify-center h-4 min-w-[1rem] px-1 rounded-full text-[10px] font-semibold',
              tradePlanStore.filterStatus === tab.key
                ? 'bg-primary/10 text-primary'
                : 'bg-muted text-muted-foreground',
            )"
          >
            {{ tab.count }}
          </span>
        </button>
      </div>

      <!-- Loading -->
      <div v-if="tradePlanStore.loading && displayedPlans.length === 0" class="flex items-center justify-center py-16">
        <Loader2 class="w-6 h-6 animate-spin text-muted-foreground" />
      </div>

      <!-- Empty state -->
      <Card v-else-if="displayedPlans.length === 0" class="border-dashed">
        <CardContent class="flex flex-col items-center justify-center py-12">
          <FileText class="w-12 h-12 text-muted-foreground/30 mb-3" />
          <p class="text-muted-foreground text-sm">
            {{ tradePlanStore.filterStatus === 'all' ? '暂无交易计划' : '该状态下暂无计划' }}
          </p>
          <p class="text-muted-foreground/60 text-xs mt-1">点击「新建计划」开始制定您的交易计划</p>
        </CardContent>
      </Card>

      <!-- Plan List -->
      <div v-else class="space-y-3">
        <Card
          v-for="plan in displayedPlans"
          :key="plan.id"
          class="transition-all duration-200 hover:shadow-md group"
        >
          <CardContent class="p-4">
            <div class="flex items-start justify-between gap-4">
              <!-- Left: Plan Info -->
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2.5 mb-2">
                  <div
                    :class="cn(
                      'flex items-center justify-center w-8 h-8 rounded-lg shrink-0',
                      plan.direction === 'long' ? 'bg-profit/10 text-profit' : 'bg-loss/10 text-loss',
                    )"
                  >
                    <ArrowUpRight v-if="plan.direction === 'long'" class="w-4 h-4" />
                    <ArrowDownRight v-else class="w-4 h-4" />
                  </div>
                  <div class="min-w-0">
                    <div class="flex items-center gap-2">
                      <span class="font-semibold text-foreground text-sm">{{ plan.symbol }}</span>
                      <span v-if="plan.name" class="text-muted-foreground text-xs">{{ plan.name }}</span>
                    </div>
                    <div class="flex items-center gap-2 mt-0.5">
                      <span class="text-xs text-muted-foreground">
                        {{ plan.direction === 'long' ? '做多' : '做空' }}
                      </span>
                      <span class="text-xs text-muted-foreground/40">|</span>
                      <span class="text-xs text-muted-foreground">
                        {{ plan.market_type === 'futures' ? '期货' : '股票' }}
                      </span>
                    </div>
                  </div>
                </div>

                <!-- Price row -->
                <div class="grid grid-cols-3 gap-4 mt-3">
                  <div>
                    <p class="text-[10px] uppercase tracking-wider text-muted-foreground mb-0.5">入场</p>
                    <p class="text-sm font-medium text-foreground">{{ plan.entry_price }}</p>
                  </div>
                  <div>
                    <p class="text-[10px] uppercase tracking-wider text-muted-foreground mb-0.5">止损</p>
                    <p class="text-sm font-medium text-loss">{{ plan.stop_loss }}</p>
                  </div>
                  <div>
                    <p class="text-[10px] uppercase tracking-wider text-muted-foreground mb-0.5">止盈</p>
                    <p class="text-sm font-medium text-profit">{{ plan.take_profit }}</p>
                  </div>
                </div>

                <!-- Tags -->
                <div v-if="parseTags(plan.tags).length" class="flex flex-wrap gap-1 mt-2.5">
                  <Badge
                    v-for="tag in parseTags(plan.tags)"
                    :key="tag"
                    variant="secondary"
                    class="text-[10px] h-5 px-1.5 gap-1"
                  >
                    <Tag class="w-2.5 h-2.5" />
                    {{ tag }}
                  </Badge>
                </div>
              </div>

              <!-- Right: Status + Actions -->
              <div class="flex flex-col items-end gap-2 shrink-0">
                <!-- Status badge -->
                <div class="flex items-center gap-1.5">
                  <component
                    :is="getStatusIcon(plan.status as TradePlanStatus)"
                    :class="cn('w-3.5 h-3.5', getStatusColor(plan.status as TradePlanStatus))"
                  />
                  <span :class="cn('text-xs font-medium', getStatusColor(plan.status as TradePlanStatus))">
                    {{ getStatusLabel(plan.status as TradePlanStatus) }}
                  </span>
                </div>

                <!-- Lots + Risk info -->
                <div class="text-right">
                  <p class="text-[10px] text-muted-foreground uppercase tracking-wider">手数</p>
                  <p class="text-sm font-semibold text-foreground">{{ plan.lots }}</p>
                </div>

                <!-- Actions -->
                <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                  <Button variant="ghost" size="sm" class="h-6 gap-1 text-[10px] px-2" @click.stop="aiAnalyzePlan(plan)" :disabled="ai.loading">
                    <Loader2 v-if="ai.loading && aiPlanId === plan.id" class="w-3 h-3 animate-spin" />
                    <Sparkles v-else class="w-3 h-3" />AI 建议
                  </Button>
                  <!-- Status transitions -->
                  <select
                    v-if="plan.status !== 'completed' && plan.status !== 'cancelled'"
                    class="text-[10px] rounded border border-input bg-background px-1.5 py-0.5 cursor-pointer"
                    :value="plan.status"
                    @change="handleStatusChange(plan, ($event.target as HTMLSelectElement).value as TradePlanStatus)"
                    @click.stop
                  >
                    <option value="planned">计划中</option>
                    <option value="executing">执行中</option>
                    <option value="completed">已完成</option>
                    <option value="cancelled">已取消</option>
                  </select>
                  <Button variant="ghost" size="icon" class="h-7 w-7" @click.stop="openEditForm(plan)">
                    <Pencil class="w-3.5 h-3.5" />
                  </Button>
                  <AlertDialog>
                    <AlertDialogTrigger as-child>
                      <Button
                        variant="ghost"
                        size="icon"
                        class="h-7 w-7 hover:text-loss"
                        @click.stop="deleteTarget = plan"
                      >
                        <Trash2 class="w-3.5 h-3.5" />
                      </Button>
                    </AlertDialogTrigger>
                    <AlertDialogContent @click.stop>
                      <AlertDialogHeader>
                        <AlertDialogTitle>确认删除交易计划</AlertDialogTitle>
                        <AlertDialogDescription>
                          确定要删除「{{ deleteTarget?.symbol }}」的交易计划吗？此操作无法撤销。
                        </AlertDialogDescription>
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

            <!-- Strategy preview -->
            <p v-if="plan.strategy" class="mt-3 pt-3 border-t border-border/50 text-xs text-muted-foreground line-clamp-2">
              {{ plan.strategy }}
            </p>

            <!-- AI Analysis Result -->
            <div v-if="aiPlanId === plan.id && (ai.loading || ai.result)" class="mt-3 pt-3 border-t border-primary/20 rounded-lg bg-primary/5 p-3">
              <div class="flex items-center gap-1.5 mb-2">
                <Sparkles class="w-3.5 h-3.5 text-primary" />
                <span class="text-xs font-medium text-primary">AI 策略建议</span>
                <Loader2 v-if="ai.loading" class="w-3 h-3 animate-spin text-primary ml-auto" />
              </div>
              <div v-if="ai.loading" class="text-xs text-muted-foreground">正在分析中...</div>
              <p v-else class="text-xs text-foreground/80 whitespace-pre-wrap leading-relaxed">{{ ai.result }}</p>
            </div>
          </CardContent>
        </Card>
      </div>
    </template>

    <!-- ============================================================ -->
    <!-- Create / Edit Form Overlay                                   -->
    <!-- ============================================================ -->
    <Transition
      enter-active-class="transition-all duration-300 ease-out"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="transition-all duration-200 ease-in"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div
        v-if="showForm"
        class="fixed inset-0 z-40 bg-black/30 backdrop-blur-sm"
        @click="closeForm"
      />
    </Transition>

    <Transition
      enter-active-class="transition-all duration-300 ease-out"
      enter-from-class="opacity-0 translate-y-4"
      enter-to-class="opacity-100 translate-y-0"
      leave-active-class="transition-all duration-200 ease-in"
      leave-from-class="opacity-100 translate-y-0"
      leave-to-class="opacity-0 translate-y-4"
    >
      <div
        v-if="showForm"
        class="fixed inset-x-0 bottom-0 z-50 mx-auto max-w-3xl px-4 pb-6"
      >
        <Card class="shadow-xl border-ring/20 max-h-[85vh] flex flex-col">
          <!-- Form Header -->
          <CardHeader class="pb-3 border-b border-border/50 shrink-0">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div class="flex items-center justify-center w-9 h-9 rounded-lg bg-primary/10">
                  <FileText class="w-4 h-4 text-primary" />
                </div>
                <div>
                  <CardTitle class="text-base">{{ isEditing ? '编辑交易计划' : '新建交易计划' }}</CardTitle>
                  <CardDescription class="mt-0.5">
                    {{ isEditing ? '修改计划参数' : `当前账户: ${currentAccount?.name || ''}` }}
                  </CardDescription>
                </div>
              </div>
              <Button variant="ghost" size="icon" class="h-8 w-8" @click="closeForm">
                <X class="w-4 h-4" />
              </Button>
            </div>

            <!-- Step Indicator -->
            <div class="flex items-center gap-1 mt-4">
              <template v-for="(step, idx) in steps" :key="idx">
                <button
                  :class="cn(
                    'flex items-center gap-2 rounded-md px-3 py-1.5 text-xs font-medium transition-colors cursor-pointer',
                    currentStep === idx
                      ? 'bg-primary/10 text-primary'
                      : currentStep > idx
                        ? 'text-foreground'
                        : 'text-muted-foreground',
                  )"
                  @click="currentStep = idx"
                >
                  <span
                    :class="cn(
                      'inline-flex items-center justify-center w-5 h-5 rounded-full text-[10px] font-semibold',
                      currentStep === idx
                        ? 'bg-primary text-primary-foreground'
                        : currentStep > idx
                          ? 'bg-profit/10 text-profit'
                          : 'bg-muted text-muted-foreground',
                    )"
                  >
                    {{ idx + 1 }}
                  </span>
                  {{ step.label }}
                </button>
                <ChevronRight v-if="idx < steps.length - 1" class="w-3 h-3 text-muted-foreground/40" />
              </template>
            </div>
          </CardHeader>

          <!-- Form Body -->
          <div class="flex-1 overflow-y-auto p-6">
            <!-- Step 1: Basic Info -->
            <div v-show="currentStep === 0" class="space-y-5">
              <div class="grid grid-cols-2 gap-5">
                <div class="space-y-2">
                  <Label for="plan-symbol" class="text-sm font-medium">
                    交易品种 <span class="text-loss">*</span>
                  </Label>
                  <Input
                    id="plan-symbol"
                    v-model="form.symbol"
                    placeholder="例如: IF2506, rb2510"
                    :class="cn(formErrors.symbol && 'border-loss focus-visible:ring-loss')"
                    @input="formErrors.symbol = ''"
                  />
                  <p v-if="formErrors.symbol" class="text-xs text-loss">{{ formErrors.symbol }}</p>
                </div>
                <div class="space-y-2">
                  <Label for="plan-name" class="text-sm font-medium">品种名称</Label>
                  <Input id="plan-name" v-model="form.name" placeholder="例如: 沪深300指数" />
                </div>
              </div>

              <div class="grid grid-cols-2 gap-5">
                <div class="space-y-2">
                  <Label class="text-sm font-medium">交易方向</Label>
                  <div class="grid grid-cols-2 gap-2">
                    <button
                      :class="cn(
                        'flex items-center justify-center gap-2 rounded-lg border px-4 py-2.5 text-sm font-medium transition-all cursor-pointer',
                        form.direction === 'long'
                          ? 'border-profit/40 bg-profit/10 text-profit'
                          : 'border-input text-muted-foreground hover:border-profit/20',
                      )"
                      @click="form.direction = 'long'"
                    >
                      <ArrowUpRight class="w-4 h-4" />
                      做多
                    </button>
                    <button
                      :class="cn(
                        'flex items-center justify-center gap-2 rounded-lg border px-4 py-2.5 text-sm font-medium transition-all cursor-pointer',
                        form.direction === 'short'
                          ? 'border-loss/40 bg-loss/10 text-loss'
                          : 'border-input text-muted-foreground hover:border-loss/20',
                      )"
                      @click="form.direction = 'short'"
                    >
                      <ArrowDownRight class="w-4 h-4" />
                      做空
                    </button>
                  </div>
                </div>
                <div class="space-y-2">
                  <Label for="plan-market" class="text-sm font-medium">市场类型</Label>
                  <SelectNative id="plan-market" v-model="form.market_type">
                    <option value="futures">期货</option>
                    <option value="stock">股票</option>
                  </SelectNative>
                </div>
              </div>
            </div>

            <!-- Step 2: Price & Calculation -->
            <div v-show="currentStep === 1" class="space-y-5">
              <div class="grid grid-cols-3 gap-4">
                <div class="space-y-2">
                  <Label for="entry-price" class="text-sm font-medium">
                    入场价格 <span class="text-loss">*</span>
                  </Label>
                  <Input
                    id="entry-price"
                    v-model.number="form.entry_price"
                    type="number"
                    step="0.01"
                    min="0"
                    placeholder="0.00"
                  />
                </div>
                <div class="space-y-2">
                  <Label for="stop-loss" class="text-sm font-medium">
                    止损价格 <span class="text-loss">*</span>
                  </Label>
                  <Input
                    id="stop-loss"
                    v-model.number="form.stop_loss"
                    type="number"
                    step="0.01"
                    min="0"
                    placeholder="0.00"
                    class="border-loss/30 focus-visible:ring-loss/50"
                  />
                </div>
                <div class="space-y-2">
                  <Label for="take-profit" class="text-sm font-medium">
                    止盈价格 <span class="text-loss">*</span>
                  </Label>
                  <Input
                    id="take-profit"
                    v-model.number="form.take_profit"
                    type="number"
                    step="0.01"
                    min="0"
                    placeholder="0.00"
                    class="border-profit/30 focus-visible:ring-profit/50"
                  />
                </div>
              </div>

              <!-- Calculation Panel -->
              <div
                v-if="calcResult && calcResult.stop_loss_points > 0"
                class="rounded-xl bg-gradient-to-br from-secondary/80 to-secondary/30 border border-border/50 p-5 space-y-4"
              >
                <div class="flex items-center gap-2 text-sm font-medium text-foreground">
                  <Calculator class="w-4 h-4 text-primary" />
                  自动计算结果
                </div>

                <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
                  <div>
                    <p class="text-[10px] uppercase tracking-wider text-muted-foreground mb-1">最大风险</p>
                    <p class="text-lg font-semibold text-loss">
                      &yen;{{ formatCurrency(calcResult.max_risk_amount) }}
                    </p>
                  </div>
                  <div>
                    <p class="text-[10px] uppercase tracking-wider text-muted-foreground mb-1">建议手数</p>
                    <p class="text-lg font-semibold text-foreground">{{ calcResult.suggested_lots }}</p>
                  </div>
                  <div>
                    <p class="text-[10px] uppercase tracking-wider text-muted-foreground mb-1">盈亏比</p>
                    <p :class="cn(
                      'text-lg font-semibold',
                      calcResult.risk_reward_ratio >= 2 ? 'text-profit' : calcResult.risk_reward_ratio >= 1 ? 'text-warning' : 'text-loss',
                    )">
                      {{ calcResult.risk_reward_ratio.toFixed(2) }} : 1
                    </p>
                  </div>
                  <div>
                    <p class="text-[10px] uppercase tracking-wider text-muted-foreground mb-1">实际风险比</p>
                    <p class="text-lg font-semibold text-foreground">{{ formatPercent(calcResult.actual_risk_ratio) }}</p>
                  </div>
                </div>

                <div class="grid grid-cols-2 gap-4 text-xs text-muted-foreground">
                  <div class="flex items-center gap-1.5">
                    <ShieldAlert class="w-3.5 h-3.5 text-loss" />
                    止损 {{ calcResult.stop_loss_points.toFixed(2) }} 点
                  </div>
                  <div class="flex items-center gap-1.5">
                    <TrendingUp class="w-3.5 h-3.5 text-profit" />
                    止盈 {{ calcResult.take_profit_points.toFixed(2) }} 点
                  </div>
                </div>
              </div>

              <!-- Lots override -->
              <div class="space-y-2 max-w-xs">
                <Label for="plan-lots" class="text-sm font-medium">手数/数量</Label>
                <div class="relative">
                  <Input
                    id="plan-lots"
                    v-model.number="form.lots"
                    type="number"
                    min="0"
                    step="1"
                    :class="calcResult ? 'pr-16' : ''"
                  />
                  <span v-if="calcResult" class="absolute right-3 top-1/2 -translate-y-1/2 text-[10px] text-muted-foreground">
                    建议 {{ calcResult.suggested_lots }}
                  </span>
                </div>
              </div>
            </div>

            <!-- Step 3: Strategy & Notes -->
            <div v-show="currentStep === 2" class="space-y-5">
              <div class="space-y-2">
                <Label for="plan-strategy" class="text-sm font-medium">交易策略</Label>
                <Textarea
                  id="plan-strategy"
                  v-model="form.strategy"
                  placeholder="描述您的交易逻辑、入场理由、缠论分析等..."
                  class="min-h-[120px]"
                />
              </div>

              <div class="grid grid-cols-2 gap-5">
                <div class="space-y-2">
                  <Label for="plan-tags" class="text-sm font-medium">标签</Label>
                  <Input
                    id="plan-tags"
                    v-model="form.tagsInput"
                    placeholder="用逗号分隔，如: 缠论,三买,突破"
                  />
                </div>
                <div class="space-y-2">
                  <Label for="plan-time" class="text-sm font-medium">计划交易时间</Label>
                  <Input
                    id="plan-time"
                    v-model="form.planned_at"
                    type="datetime-local"
                  />
                </div>
              </div>

              <div class="space-y-2">
                <Label for="plan-notes" class="text-sm font-medium">备注</Label>
                <Textarea
                  id="plan-notes"
                  v-model="form.notes"
                  placeholder="其他备注信息..."
                  class="min-h-[80px]"
                />
              </div>
            </div>
          </div>

          <!-- Form Footer -->
          <div class="flex items-center justify-between px-6 py-4 border-t border-border/50 shrink-0">
            <div class="flex items-center gap-2">
              <Button
                v-if="currentStep > 0"
                type="button"
                variant="outline"
                size="sm"
                @click="prevStep"
              >
                上一步
              </Button>
            </div>
            <div class="flex items-center gap-2">
              <Button type="button" variant="outline" size="sm" @click="closeForm">
                取消
              </Button>
              <Button
                v-if="currentStep < steps.length - 1"
                type="button"
                size="sm"
                class="gap-1.5"
                :disabled="!canProceed"
                @click="nextStep"
              >
                下一步
                <ChevronRight class="w-3.5 h-3.5" />
              </Button>
              <Button
                v-else
                type="button"
                size="sm"
                class="gap-2"
                :disabled="formSubmitting"
                @click="handleSubmit"
              >
                <Loader2 v-if="formSubmitting" class="w-4 h-4 animate-spin" />
                <Target v-else class="w-4 h-4" />
                {{ isEditing ? '保存更改' : '创建计划' }}
              </Button>
            </div>
          </div>
        </Card>
      </div>
    </Transition>
  </div>
</template>
