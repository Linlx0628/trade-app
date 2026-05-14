<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import {
  Wallet,
  Plus,
  Pencil,
  Trash2,
  ShieldCheck,
  TrendingUp,
  Loader2,
  AlertTriangle,
  CheckCircle2,
  CircleDot,
  Coins,
  Percent,
  Banknote,
  Sparkles,
  Save,
  Download,
  Upload,
  HardDrive,
  LayoutTemplate,
  Pin,
} from 'lucide-vue-next'
import { useAccountStore } from '@/stores/account'
import { useTradeTemplateStore } from '@/stores/trade-template'
import { useToast } from '@/components/ui/toast'
import { Card, CardHeader, CardTitle, CardDescription, CardContent } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Badge } from '@/components/ui/badge'
import { Separator } from '@/components/ui/separator'
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
import type { Account } from '@/types/common'

const accountStore = useAccountStore()
const templateStore = useTradeTemplateStore()
const { toast } = useToast()

// --- State ---
const showForm = ref(false)
const editingAccount = ref<Account | null>(null)
const deleteTarget = ref<Account | null>(null)
const formSubmitting = ref(false)

const form = reactive({
  name: '',
  balance: 100000,
  risk_ratio: 0.01,
  point_value: 10,
})

const formErrors = reactive({
  name: '',
  balance: '',
  risk_ratio: '',
})

// --- Computed ---
const isEditing = computed(() => editingAccount.value !== null)
const formTitle = computed(() => (isEditing.value ? '编辑账户' : '新建账户'))
const submitLabel = computed(() => (isEditing.value ? '保存更改' : '创建账户'))

const currentAccount = computed(() => accountStore.currentAccount)
const accounts = computed(() => accountStore.accounts)
const loading = computed(() => accountStore.loading)

const maxRiskAmount = computed(() => {
  if (!currentAccount.value) return 0
  return currentAccount.value.balance * currentAccount.value.risk_ratio
})

// --- Methods ---
function formatCurrency(value: number): string {
  return new Intl.NumberFormat('zh-CN', {
    minimumFractionDigits: 0,
    maximumFractionDigits: 2,
  }).format(value)
}

function formatPercent(value: number): string {
  return `${(value * 100).toFixed(2)}%`
}

function resetForm() {
  form.name = ''
  form.balance = 100000
  form.risk_ratio = 0.01
  form.point_value = 10
  formErrors.name = ''
  formErrors.balance = ''
  formErrors.risk_ratio = ''
  editingAccount.value = null
}

function openCreateForm() {
  resetForm()
  showForm.value = true
}

function openEditForm(account: Account) {
  editingAccount.value = account
  form.name = account.name
  form.balance = account.balance
  form.risk_ratio = account.risk_ratio
  form.point_value = account.point_value
  formErrors.name = ''
  formErrors.balance = ''
  formErrors.risk_ratio = ''
  showForm.value = true
}

function closeForm() {
  showForm.value = false
  resetForm()
}

function validateForm(): boolean {
  let valid = true
  formErrors.name = ''
  formErrors.balance = ''
  formErrors.risk_ratio = ''

  if (!form.name.trim()) {
    formErrors.name = '请输入账户名称'
    valid = false
  }

  if (form.balance <= 0) {
    formErrors.balance = '净值必须大于 0'
    valid = false
  }

  if (form.risk_ratio < 0.01 || form.risk_ratio > 0.1) {
    formErrors.risk_ratio = '风险比例须在 1% ~ 10% 之间'
    valid = false
  }

  return valid
}

async function handleSubmit() {
  if (!validateForm()) return

  formSubmitting.value = true

  try {
    if (isEditing.value) {
      await accountStore.updateAccount({
        id: editingAccount.value!.id,
        name: form.name.trim(),
        balance: form.balance,
        risk_ratio: form.risk_ratio,
        point_value: form.point_value,
      })
      toast({ title: '更新成功', description: `账户 "${form.name}" 已更新`, variant: 'success' })
    } else {
      await accountStore.createAccount({
        name: form.name.trim(),
        balance: form.balance,
        risk_ratio: form.risk_ratio,
        point_value: form.point_value,
      })
      toast({ title: '创建成功', description: `账户 "${form.name}" 已创建`, variant: 'success' })
    }
    closeForm()
  } catch {
    toast({
      title: isEditing.value ? '更新失败' : '创建失败',
      description: accountStore.error || '操作失败，请重试',
      variant: 'destructive',
    })
  } finally {
    formSubmitting.value = false
  }
}

async function handleSelectAccount(id: string) {
  try {
    await accountStore.selectAccount(id)
    toast({ title: '已切换', description: '当前账户已切换', variant: 'success' })
  } catch {
    toast({ title: '切换失败', description: accountStore.error || '切换账户失败', variant: 'destructive' })
  }
}

async function handleDelete() {
  if (!deleteTarget.value) return

  try {
    await accountStore.deleteAccount(deleteTarget.value.id)
    toast({ title: '已删除', description: `账户 "${deleteTarget.value.name}" 已删除`, variant: 'success' })
  } catch {
    toast({ title: '删除失败', description: accountStore.error || '删除账户失败', variant: 'destructive' })
  } finally {
    deleteTarget.value = null
  }
}

// --- Lifecycle ---
onMounted(async () => {
  await accountStore.fetchAccounts()
  // Auto-select first account if none selected
  if (!accountStore.currentAccount && accountStore.accounts.length > 0) {
    await accountStore.selectAccount(accountStore.accounts[0].id)
  }
  // Load AI config from localStorage
  const saved = localStorage.getItem('ai_config')
  if (saved) {
    try { Object.assign(aiConfig, JSON.parse(saved)) } catch {}
  }
  // Load templates
  await loadTemplates()
})

// --- AI Config ---
import type { AiConfig } from '@/types/common'

const aiConfig = reactive<AiConfig>({
  provider: 'anthropic',
  api_key: '',
  model: '',
  base_url: '',
})
const aiSaving = ref(false)

async function saveAiConfig() {
  aiSaving.value = true
  localStorage.setItem('ai_config', JSON.stringify(aiConfig))
  await new Promise(r => setTimeout(r, 300))
  aiSaving.value = false
  toast({ title: 'AI 配置已保存', variant: 'success' })
}

// --- Data IO ---
const exporting = ref(false)
const importing = ref(false)
const backingUp = ref(false)

async function handleExportLogs() {
  const acc = accountStore.currentAccount
  if (!acc) { toast({ title: '请先选择账户', variant: 'destructive' }); return }
  exporting.value = true
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const path = await open({ directory: true, multiple: false, title: '选择导出目录' })
    if (!path) { exporting.value = false; return }
    const dir = typeof path === 'string' ? path : path
    const { dataIoApi } = await import('@/lib/tauri')
    const msg = await dataIoApi.exportTradeLogsCsv(acc.id, `${dir}/交易日志_${new Date().toISOString().slice(0,10)}.csv`)
    toast({ title: '导出成功', description: msg, variant: 'success' })
  } catch (e: unknown) {
    toast({ title: '导出失败', description: e instanceof Error ? e.message : String(e), variant: 'destructive' })
  } finally { exporting.value = false }
}

async function handleExportPlans() {
  const acc = accountStore.currentAccount
  if (!acc) { toast({ title: '请先选择账户', variant: 'destructive' }); return }
  exporting.value = true
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const path = await open({ directory: true, multiple: false, title: '选择导出目录' })
    if (!path) { exporting.value = false; return }
    const dir = typeof path === 'string' ? path : path
    const { dataIoApi } = await import('@/lib/tauri')
    const msg = await dataIoApi.exportTradePlansCsv(acc.id, `${dir}/交易计划_${new Date().toISOString().slice(0,10)}.csv`)
    toast({ title: '导出成功', description: msg, variant: 'success' })
  } catch (e: unknown) {
    toast({ title: '导出失败', description: e instanceof Error ? e.message : String(e), variant: 'destructive' })
  } finally { exporting.value = false }
}

async function handleImport() {
  const acc = accountStore.currentAccount
  if (!acc) { toast({ title: '请先选择账户', variant: 'destructive' }); return }
  importing.value = true
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const selected = await open({ multiple: false, filters: [{ name: 'CSV', extensions: ['csv'] }], title: '选择CSV文件' })
    if (!selected) { importing.value = false; return }
    const filePath = typeof selected === 'string' ? selected : selected
    const { dataIoApi } = await import('@/lib/tauri')
    const result = await dataIoApi.importTradeLogsCsv(filePath, acc.id)
    toast({ title: '导入完成', description: `成功 ${result.imported} 条，跳过 ${result.skipped} 条`, variant: result.imported > 0 ? 'success' : 'destructive' })
    if (result.errors.length > 0) console.warn('Import errors:', result.errors)
  } catch (e: unknown) {
    toast({ title: '导入失败', description: e instanceof Error ? e.message : String(e), variant: 'destructive' })
  } finally { importing.value = false }
}

async function handleBackup() {
  backingUp.value = true
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const path = await open({ directory: true, multiple: false, title: '选择备份目录' })
    if (!path) { backingUp.value = false; return }
    const dir = typeof path === 'string' ? path : path
    const { dataIoApi } = await import('@/lib/tauri')
    const msg = await dataIoApi.createBackup(dir)
    toast({ title: '备份成功', description: msg, variant: 'success' })
  } catch (e: unknown) {
    toast({ title: '备份失败', description: e instanceof Error ? e.message : String(e), variant: 'destructive' })
  } finally { backingUp.value = false }
}

// --- Template Management ---
const templateDeleting = ref<string | null>(null)

async function loadTemplates() {
  if (accountStore.currentAccount) await templateStore.fetchTemplates(accountStore.currentAccount.id)
}

async function handleDeleteTemplate(id: string) {
  templateDeleting.value = id
  try { await templateStore.deleteTemplate(id); toast({ title: '模板已删除', variant: 'success' }) }
  catch { toast({ title: '删除失败', variant: 'destructive' }) }
  finally { templateDeleting.value = null }
}

async function handleTogglePin(id: string) {
  try { await templateStore.togglePin(id) }
  catch { toast({ title: '操作失败', variant: 'destructive' }) }
}
</script>

<template>
  <div class="max-w-5xl mx-auto space-y-6">
    <!-- ============================================================ -->
    <!-- Section 1: Current Account Overview                          -->
    <!-- ============================================================ -->
    <Card v-if="currentAccount" class="border-ring/30 bg-gradient-to-br from-card to-card/80">
      <CardHeader class="pb-4">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-3">
            <div class="flex items-center justify-center w-10 h-10 rounded-lg bg-primary/10">
              <Wallet class="w-5 h-5 text-foreground" />
            </div>
            <div>
              <CardTitle class="text-base">当前账户</CardTitle>
              <CardDescription class="mt-0.5">{{ currentAccount.name }}</CardDescription>
            </div>
          </div>
          <Badge variant="outline" class="gap-1.5 text-xs">
            <CircleDot class="w-3 h-3 text-profit" />
            活跃
          </Badge>
        </div>
      </CardHeader>
      <CardContent>
        <div class="grid grid-cols-2 lg:grid-cols-4 gap-6">
          <!-- Balance -->
          <div class="col-span-2 lg:col-span-1">
            <p class="text-xs font-medium text-muted-foreground mb-1 uppercase tracking-wider">账户净值</p>
            <p class="text-3xl font-bold text-foreground tracking-tight">
              <span class="text-lg text-muted-foreground mr-0.5">&yen;</span>{{ formatCurrency(currentAccount.balance) }}
            </p>
          </div>
          <!-- Risk Ratio -->
          <div>
            <p class="text-xs font-medium text-muted-foreground mb-1 uppercase tracking-wider">风险比例</p>
            <div class="flex items-center gap-2 mt-1">
              <ShieldCheck class="w-5 h-5 text-warning" />
              <span class="text-xl font-semibold text-foreground">{{ formatPercent(currentAccount.risk_ratio) }}</span>
            </div>
          </div>
          <!-- Point Value -->
          <div>
            <p class="text-xs font-medium text-muted-foreground mb-1 uppercase tracking-wider">每点价值</p>
            <div class="flex items-center gap-2 mt-1">
              <Coins class="w-5 h-5 text-muted-foreground" />
              <span class="text-xl font-semibold text-foreground">&yen;{{ currentAccount.point_value }}</span>
            </div>
          </div>
          <!-- Max Risk -->
          <div>
            <p class="text-xs font-medium text-muted-foreground mb-1 uppercase tracking-wider">单笔最大风险</p>
            <div class="flex items-center gap-2 mt-1">
              <TrendingUp class="w-5 h-5 text-loss" />
              <span class="text-xl font-semibold text-loss">&yen;{{ formatCurrency(maxRiskAmount) }}</span>
            </div>
          </div>
        </div>
      </CardContent>
    </Card>

    <!-- Empty state when no current account -->
    <Card v-else class="border-dashed">
      <CardContent class="flex flex-col items-center justify-center py-12">
        <Wallet class="w-12 h-12 text-muted-foreground/50 mb-3" />
        <p class="text-muted-foreground text-sm">尚未选择账户</p>
        <p class="text-muted-foreground/60 text-xs mt-1">请从下方选择或创建一个账户</p>
      </CardContent>
    </Card>

    <!-- ============================================================ -->
    <!-- Section 2 & 3: Account List + Create/Edit Form               -->
    <!-- ============================================================ -->
    <div class="flex items-center justify-between">
      <div>
        <h2 class="text-lg font-semibold text-foreground">账户管理</h2>
        <p class="text-sm text-muted-foreground mt-0.5">管理您的交易账户，点击卡片切换当前使用账户</p>
      </div>
      <Button size="sm" class="gap-2" @click="openCreateForm">
        <Plus class="w-4 h-4" />
        新建账户
      </Button>
    </div>

    <!-- Create / Edit Form -->
    <Transition
      enter-active-class="transition-all duration-300 ease-out"
      enter-from-class="opacity-0 -translate-y-2"
      enter-to-class="opacity-100 translate-y-0"
      leave-active-class="transition-all duration-200 ease-in"
      leave-from-class="opacity-100 translate-y-0"
      leave-to-class="opacity-0 -translate-y-2"
    >
      <Card v-if="showForm" class="border-ring/30 shadow-md">
        <CardHeader class="pb-4">
          <div class="flex items-center gap-3">
            <div class="flex items-center justify-center w-9 h-9 rounded-lg bg-primary/10">
              <Pencil v-if="isEditing" class="w-4 h-4 text-foreground" />
              <Plus v-else class="w-4 h-4 text-foreground" />
            </div>
            <div>
              <CardTitle class="text-base">{{ formTitle }}</CardTitle>
              <CardDescription class="mt-0.5">
                {{ isEditing ? '修改账户配置信息' : '填写以下信息创建新交易账户' }}
              </CardDescription>
            </div>
          </div>
        </CardHeader>
        <CardContent>
          <form @submit.prevent="handleSubmit" class="space-y-5">
            <!-- Account Name -->
            <div class="space-y-2">
              <Label for="account-name" class="text-sm font-medium">
                账户名称 <span class="text-loss">*</span>
              </Label>
              <Input
                id="account-name"
                v-model="form.name"
                placeholder="例如：主账户、模拟盘 A"
                :class="cn(formErrors.name && 'border-loss focus-visible:ring-loss')"
              />
              <p v-if="formErrors.name" class="text-xs text-loss flex items-center gap-1">
                <AlertTriangle class="w-3 h-3" />
                {{ formErrors.name }}
              </p>
            </div>

            <!-- Balance + Risk Ratio row -->
            <div class="grid grid-cols-1 sm:grid-cols-2 gap-5">
              <!-- Balance -->
              <div class="space-y-2">
                <Label for="account-balance" class="text-sm font-medium">
                  账户净值 <span class="text-loss">*</span>
                </Label>
                <div class="relative">
                  <span class="absolute left-3 top-1/2 -translate-y-1/2 text-sm text-muted-foreground">&yen;</span>
                  <Input
                    id="account-balance"
                    v-model="form.balance"
                    type="number"
                    min="0"
                    step="any"
                    class="pl-7"
                    :class="cn(formErrors.balance && 'border-loss focus-visible:ring-loss')"
                  />
                </div>
                <p v-if="formErrors.balance" class="text-xs text-loss flex items-center gap-1">
                  <AlertTriangle class="w-3 h-3" />
                  {{ formErrors.balance }}
                </p>
              </div>

              <!-- Risk Ratio -->
              <div class="space-y-2">
                <Label for="risk-ratio" class="text-sm font-medium">
                  风险比例 (1% ~ 10%)
                </Label>
                <div class="relative">
                  <Input
                    id="risk-ratio"
                    v-model="form.risk_ratio"
                    type="number"
                    min="0.01"
                    max="0.1"
                    step="0.005"
                    :class="cn(formErrors.risk_ratio && 'border-loss focus-visible:ring-loss')"
                  />
                  <span class="absolute right-3 top-1/2 -translate-y-1/2 text-sm text-muted-foreground">
                    {{ formatPercent(form.risk_ratio) }}
                  </span>
                </div>
                <p v-if="formErrors.risk_ratio" class="text-xs text-loss flex items-center gap-1">
                  <AlertTriangle class="w-3 h-3" />
                  {{ formErrors.risk_ratio }}
                </p>
              </div>
            </div>

            <!-- Point Value -->
            <div class="space-y-2 max-w-xs">
              <Label for="point-value" class="text-sm font-medium">每点价值</Label>
              <div class="relative">
                <span class="absolute left-3 top-1/2 -translate-y-1/2 text-sm text-muted-foreground">&yen;</span>
                <Input
                  id="point-value"
                  v-model="form.point_value"
                  type="number"
                  min="1"
                  step="1"
                  class="pl-7"
                />
              </div>
            </div>

            <!-- Risk Preview -->
            <div class="rounded-lg bg-secondary/50 border border-border/50 p-4">
              <p class="text-xs font-medium text-muted-foreground mb-2">风险预览</p>
              <div class="grid grid-cols-2 gap-4 text-sm">
                <div>
                  <span class="text-muted-foreground">单笔最大风险:</span>
                  <span class="ml-2 font-semibold text-loss">
                    &yen;{{ formatCurrency(form.balance * form.risk_ratio) }}
                  </span>
                </div>
                <div>
                  <span class="text-muted-foreground">每点价值:</span>
                  <span class="ml-2 font-semibold text-foreground">&yen;{{ form.point_value }}</span>
                </div>
              </div>
            </div>

            <Separator />

            <!-- Form Actions -->
            <div class="flex items-center justify-end gap-3">
              <Button type="button" variant="outline" size="sm" @click="closeForm">
                取消
              </Button>
              <Button type="submit" size="sm" class="gap-2" :disabled="formSubmitting">
                <Loader2 v-if="formSubmitting" class="w-4 h-4 animate-spin" />
                <CheckCircle2 v-else class="w-4 h-4" />
                {{ submitLabel }}
              </Button>
            </div>
          </form>
        </CardContent>
      </Card>
    </Transition>

    <!-- Account List -->
    <div v-if="loading && accounts.length === 0" class="flex items-center justify-center py-16">
      <Loader2 class="w-6 h-6 animate-spin text-muted-foreground" />
    </div>

    <div v-else-if="accounts.length === 0" class="text-center py-12">
      <Banknote class="w-12 h-12 text-muted-foreground/40 mx-auto mb-3" />
      <p class="text-muted-foreground text-sm">暂无账户</p>
      <p class="text-muted-foreground/60 text-xs mt-1">点击上方「新建账户」按钮创建您的第一个交易账户</p>
    </div>

    <div v-else class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
      <Card
        v-for="account in accounts"
        :key="account.id"
        :class="cn(
          'cursor-pointer transition-all duration-200 hover:shadow-lg hover:-translate-y-0.5 group',
          currentAccount?.id === account.id
            ? 'border-ring ring-1 ring-ring/40 shadow-md'
            : 'border-border hover:border-ring/30',
        )"
        @click="handleSelectAccount(account.id)"
      >
        <CardHeader class="pb-3">
          <div class="flex items-start justify-between">
            <div class="flex items-center gap-2.5 min-w-0">
              <div
                :class="cn(
                  'flex items-center justify-center w-8 h-8 rounded-lg shrink-0',
                  currentAccount?.id === account.id
                    ? 'bg-primary text-primary-foreground'
                    : 'bg-secondary text-secondary-foreground',
                )"
              >
                <Wallet class="w-4 h-4" />
              </div>
              <div class="min-w-0">
                <CardTitle class="text-sm truncate">{{ account.name }}</CardTitle>
                <CardDescription class="text-xs mt-0.5">
                  {{ account.market_type === 'futures' ? '期货' : account.market_type === 'stock' ? '股票' : '综合' }}
                </CardDescription>
              </div>
            </div>
            <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity shrink-0">
              <Button
                variant="ghost"
                size="icon"
                class="h-7 w-7"
                @click.stop="openEditForm(account)"
              >
                <Pencil class="w-3.5 h-3.5" />
              </Button>
              <AlertDialog>
                <AlertDialogTrigger as-child>
                  <Button
                    variant="ghost"
                    size="icon"
                    class="h-7 w-7 hover:text-loss"
                    @click.stop="deleteTarget = account"
                  >
                    <Trash2 class="w-3.5 h-3.5" />
                  </Button>
                </AlertDialogTrigger>
                <AlertDialogContent @click.stop>
                  <AlertDialogHeader>
                    <AlertDialogTitle class="flex items-center gap-2">
                      <AlertTriangle class="w-5 h-5 text-loss" />
                      确认删除账户
                    </AlertDialogTitle>
                    <AlertDialogDescription>
                      您确定要删除账户「{{ deleteTarget?.name }}」吗？此操作无法撤销，账户下的所有相关数据将被永久删除。
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
        </CardHeader>
        <CardContent class="pt-0">
          <div class="space-y-2">
            <div class="flex items-center justify-between text-sm">
              <span class="text-muted-foreground flex items-center gap-1.5">
                <Banknote class="w-3.5 h-3.5" />
                净值
              </span>
              <span class="font-semibold text-foreground">&yen;{{ formatCurrency(account.balance) }}</span>
            </div>
            <Separator />
            <div class="grid grid-cols-2 gap-2 text-xs">
              <div class="flex items-center gap-1.5">
                <Percent class="w-3 h-3 text-muted-foreground" />
                <span class="text-muted-foreground">风险</span>
                <span class="font-medium text-foreground ml-auto">{{ formatPercent(account.risk_ratio) }}</span>
              </div>
              <div class="flex items-center gap-1.5">
                <Coins class="w-3 h-3 text-muted-foreground" />
                <span class="text-muted-foreground">点值</span>
                <span class="font-medium text-foreground ml-auto">&yen;{{ account.point_value }}</span>
              </div>
            </div>
          </div>
          <!-- Active indicator -->
          <div
            v-if="currentAccount?.id === account.id"
            class="mt-3 pt-3 border-t border-border/50 flex items-center justify-center gap-1.5 text-xs text-profit"
          >
            <CheckCircle2 class="w-3.5 h-3.5" />
            当前使用中
          </div>
        </CardContent>
      </Card>
    </div>

    <!-- ============================================================ -->
    <!-- Section: Stats Overview (when current account has stats)     -->
    <!-- ============================================================ -->
    <Card v-if="accountStore.currentAccountStats" class="mt-2">
      <CardHeader class="pb-4">
        <div class="flex items-center gap-3">
          <div class="flex items-center justify-center w-9 h-9 rounded-lg bg-secondary">
            <TrendingUp class="w-4 h-4 text-foreground" />
          </div>
          <div>
            <CardTitle class="text-base">账户统计</CardTitle>
            <CardDescription class="mt-0.5">当前账户的统计数据概览</CardDescription>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        <div class="grid grid-cols-2 lg:grid-cols-5 gap-4">
          <div class="space-y-1">
            <p class="text-xs text-muted-foreground">总盈亏</p>
            <p :class="cn(
              'text-lg font-semibold',
              accountStore.currentAccountStats.total_pnl >= 0 ? 'text-profit' : 'text-loss',
            )">
              {{ accountStore.currentAccountStats.total_pnl >= 0 ? '+' : '' }}&yen;{{ formatCurrency(accountStore.currentAccountStats.total_pnl) }}
            </p>
          </div>
          <div class="space-y-1">
            <p class="text-xs text-muted-foreground">总交易数</p>
            <p class="text-lg font-semibold text-foreground">{{ accountStore.currentAccountStats.total_trades }}</p>
          </div>
          <div class="space-y-1">
            <p class="text-xs text-muted-foreground">胜率</p>
            <p class="text-lg font-semibold text-foreground">
              {{ (accountStore.currentAccountStats.win_rate * 100).toFixed(1) }}%
            </p>
          </div>
          <div class="space-y-1">
            <p class="text-xs text-muted-foreground">当前持仓</p>
            <p class="text-lg font-semibold text-foreground">
              {{ accountStore.currentAccountStats.open_positions }}
            </p>
          </div>
          <div class="space-y-1">
            <p class="text-xs text-muted-foreground">当前净值</p>
            <p class="text-lg font-semibold text-foreground">
              &yen;{{ formatCurrency(accountStore.currentAccountStats.balance) }}
            </p>
          </div>
        </div>
      </CardContent>
    </Card>

    <!-- AI Configuration -->
    <Card class="mt-2">
      <CardHeader class="pb-4">
        <div class="flex items-center gap-3">
          <div class="flex items-center justify-center w-9 h-9 rounded-lg bg-primary/10">
            <Sparkles class="w-4 h-4 text-primary" />
          </div>
          <div>
            <CardTitle class="text-base">AI 助手配置</CardTitle>
            <CardDescription class="mt-0.5">配置 AI 接口以获取智能交易分析和复盘建议</CardDescription>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        <div class="space-y-4 max-w-lg">
          <div class="grid grid-cols-2 gap-4">
            <div class="space-y-2">
              <Label class="text-sm font-medium">AI 提供商</Label>
              <SelectNative v-model="aiConfig.provider">
                <option value="anthropic">Anthropic (Claude)</option>
                <option value="openai">OpenAI (GPT)</option>
              </SelectNative>
            </div>
            <div class="space-y-2">
              <Label class="text-sm font-medium">模型</Label>
              <Input v-model="aiConfig.model" :placeholder="aiConfig.provider === 'anthropic' ? 'claude-sonnet-4-20250514' : 'gpt-4o'" />
            </div>
          </div>
          <div class="space-y-2">
            <Label class="text-sm font-medium">API Key</Label>
            <Input v-model="aiConfig.api_key" type="password" placeholder="sk-..." />
          </div>
          <div class="space-y-2">
            <Label class="text-sm font-medium">自定义 API 地址（可选）</Label>
            <Input v-model="aiConfig.base_url" placeholder="留空使用默认地址" />
          </div>
          <Button size="sm" class="gap-2" :disabled="aiSaving" @click="saveAiConfig">
            <Loader2 v-if="aiSaving" class="w-4 h-4 animate-spin" />
            <Save v-else class="w-4 h-4" />
            保存配置
          </Button>
        </div>
      </CardContent>
    </Card>

    <!-- Template Management -->
    <Card>
      <CardHeader>
        <div class="flex items-center gap-3">
          <div class="flex items-center justify-center w-9 h-9 rounded-lg bg-primary/10">
            <LayoutTemplate class="w-4 h-4 text-primary" />
          </div>
          <div>
            <CardTitle class="text-base">模板管理</CardTitle>
            <CardDescription class="mt-0.5">管理交易计划模板，置顶常用模板</CardDescription>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        <div v-if="templateStore.loading" class="flex items-center justify-center py-8">
          <Loader2 class="w-5 h-5 animate-spin text-muted-foreground" />
        </div>
        <div v-else-if="templateStore.templates.length === 0" class="text-center py-8 text-sm text-muted-foreground">
          暂无模板。可以在交易计划中选择「保存为模板」创建。
        </div>
        <div v-else class="space-y-2">
          <div v-for="t in templateStore.templates" :key="t.id"
            class="flex items-center gap-3 p-3 rounded-lg border border-border/50 hover:bg-secondary/30 transition-colors">
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2">
                <span class="text-sm font-medium text-foreground truncate">{{ t.name }}</span>
                <Badge v-if="t.is_pinned" variant="secondary" class="text-[10px] h-4 px-1.5 gap-0.5">
                  <Pin class="w-2.5 h-2.5" />置顶
                </Badge>
              </div>
              <div class="flex items-center gap-2 mt-0.5 text-xs text-muted-foreground">
                <span v-if="t.symbol">{{ t.symbol }}</span>
                <span v-if="t.symbol">·</span>
                <span>{{ t.direction === 'long' ? '多' : '空' }}</span>
                <span>· 使用 {{ t.usage_count }}次</span>
              </div>
            </div>
            <div class="flex items-center gap-1">
              <Button variant="ghost" size="sm" class="h-7 px-2 text-xs" @click="handleTogglePin(t.id)">
                <Pin :class="['w-3.5 h-3.5', t.is_pinned ? 'text-primary' : 'text-muted-foreground']" />
              </Button>
              <AlertDialog>
                <AlertDialogTrigger as-child>
                  <Button variant="ghost" size="icon" class="h-7 w-7 hover:text-loss">
                    <Loader2 v-if="templateDeleting === t.id" class="w-3.5 h-3.5 animate-spin" />
                    <Trash2 v-else class="w-3.5 h-3.5" />
                  </Button>
                </AlertDialogTrigger>
                <AlertDialogContent>
                  <AlertDialogHeader>
                    <AlertDialogTitle>确认删除模板</AlertDialogTitle>
                    <AlertDialogDescription>确定要删除模板「{{ t.name }}」吗？</AlertDialogDescription>
                  </AlertDialogHeader>
                  <AlertDialogFooter>
                    <AlertDialogCancel>取消</AlertDialogCancel>
                    <AlertDialogAction @click="handleDeleteTemplate(t.id)">确认删除</AlertDialogAction>
                  </AlertDialogFooter>
                </AlertDialogContent>
              </AlertDialog>
            </div>
          </div>
        </div>
      </CardContent>
    </Card>

    <!-- Data Management -->
    <Card>
      <CardHeader>
        <div class="flex items-center gap-3">
          <div class="flex items-center justify-center w-9 h-9 rounded-lg bg-primary/10">
            <HardDrive class="w-4 h-4 text-primary" />
          </div>
          <div>
            <CardTitle class="text-base">数据管理</CardTitle>
            <CardDescription class="mt-0.5">导出、导入和备份交易数据</CardDescription>
          </div>
        </div>
      </CardHeader>
      <CardContent class="space-y-4">
        <div class="space-y-3">
          <p class="text-sm font-medium text-foreground">数据导出</p>
          <div class="flex items-center gap-2">
            <Button variant="outline" size="sm" class="gap-2" :disabled="exporting" @click="handleExportLogs">
              <Loader2 v-if="exporting" class="w-4 h-4 animate-spin" />
              <Download v-else class="w-4 h-4" />导出交易日志
            </Button>
            <Button variant="outline" size="sm" class="gap-2" :disabled="exporting" @click="handleExportPlans">
              <Download class="w-4 h-4" />导出交易计划
            </Button>
          </div>
        </div>
        <div class="border-t border-border/50" />
        <div class="space-y-3">
          <p class="text-sm font-medium text-foreground">数据导入</p>
          <p class="text-xs text-muted-foreground">支持 CSV 格式的交易日志导入</p>
          <Button variant="outline" size="sm" class="gap-2" :disabled="importing" @click="handleImport">
            <Loader2 v-if="importing" class="w-4 h-4 animate-spin" />
            <Upload v-else class="w-4 h-4" />导入 CSV 文件
          </Button>
        </div>
        <div class="border-t border-border/50" />
        <div class="space-y-3">
          <p class="text-sm font-medium text-foreground">数据备份</p>
          <p class="text-xs text-muted-foreground">完整备份数据库到指定目录</p>
          <Button variant="outline" size="sm" class="gap-2" :disabled="backingUp" @click="handleBackup">
            <Loader2 v-if="backingUp" class="w-4 h-4 animate-spin" />
            <HardDrive v-else class="w-4 h-4" />创建备份
          </Button>
        </div>
      </CardContent>
    </Card>
  </div>
</template>
