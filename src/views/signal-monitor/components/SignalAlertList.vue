<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Bell, Trash2, Loader2 } from 'lucide-vue-next'
import { useToast } from '@/components/ui/toast'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { invoke } from '@tauri-apps/api/core'
import { useAccountStore } from '@/stores/account'

const { toast } = useToast()
const accountStore = useAccountStore()

interface SignalAlert {
  id: string
  account_id: string
  symbol: string
  alert_type: string
  condition_value: number | null
  description: string | null
  is_active: boolean
  is_triggered: boolean
  triggered_at: string | null
  created_at: string
}

const alerts = ref<SignalAlert[]>([])
const loading = ref(false)
const deleting = ref<string | null>(null)

const alertTypeLabels: Record<string, string> = {
  buy1: '一买', sell1: '一卖',
  buy2: '二买', sell2: '二卖',
  buy3: '三买', sell3: '三卖',
  zg_break: '突破上沿', zd_break: '跌破下沿',
  divergence: '背离',
}

async function loadAlerts() {
  const acc = accountStore.currentAccount
  if (!acc) return
  loading.value = true
  try {
    alerts.value = await invoke<SignalAlert[]>('get_signal_alerts', { accountId: acc.id })
  } catch (e) {
    console.error('Failed to load alerts:', e)
  } finally {
    loading.value = false
  }
}

async function handleDelete(id: string) {
  deleting.value = id
  try {
    await invoke('delete_signal_alert', { id })
    toast({ title: '已删除', variant: 'success' })
    await loadAlerts()
  } catch (e) {
    toast({ title: '删除失败', description: String(e), variant: 'destructive' })
  } finally {
    deleting.value = null
  }
}

onMounted(loadAlerts)
</script>

<template>
  <div class="space-y-3">
    <div class="flex items-center gap-2">
      <Bell class="w-4 h-4 text-primary" />
      <span class="text-sm font-medium text-foreground">信号提醒</span>
      <div class="flex-1" />
      <Button variant="outline" size="sm" @click="loadAlerts" :disabled="loading">
        <Loader2 v-if="loading" class="w-3.5 h-3.5 animate-spin" />
      </Button>
    </div>

    <div v-if="alerts.length === 0" class="text-center py-6 text-sm text-muted-foreground">
      暂无信号提醒
    </div>

    <div v-else class="space-y-2">
      <div v-for="alert in alerts" :key="alert.id"
        class="flex items-center gap-3 p-2.5 rounded-lg border border-border/50 text-sm">
        <Badge :variant="alert.alert_type.startsWith('buy') ? 'default' : 'destructive'" class="text-xs shrink-0">
          {{ alertTypeLabels[alert.alert_type] || alert.alert_type }}
        </Badge>
        <div class="flex-1 min-w-0">
          <span class="text-foreground">{{ alert.symbol }}</span>
          <span v-if="alert.description" class="text-muted-foreground ml-2 truncate">{{ alert.description }}</span>
        </div>
        <Badge v-if="alert.is_triggered" variant="secondary" class="text-xs shrink-0">已触发</Badge>
        <Button variant="ghost" size="icon" class="h-6 w-6 hover:text-loss shrink-0" :disabled="deleting === alert.id" @click="handleDelete(alert.id)">
          <Loader2 v-if="deleting === alert.id" class="w-3 h-3 animate-spin" />
          <Trash2 v-else class="w-3 h-3" />
        </Button>
      </div>
    </div>
  </div>
</template>
