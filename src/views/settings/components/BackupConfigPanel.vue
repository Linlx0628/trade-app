<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { ShieldCheck, Loader2, FolderOpen, Save, Clock, HardDrive } from 'lucide-vue-next'
import { useToast } from '@/components/ui/toast'
import { Card, CardHeader, CardTitle, CardDescription, CardContent } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Separator } from '@/components/ui/separator'
import { SelectNative } from '@/components/ui/select'
import { backupApi, type BackupConfig, type BackupStatus } from '@/lib/tauri'

const { toast } = useToast()

const config = reactive<BackupConfig>({
  enabled: true,
  backup_dir: '',
  interval_minutes: 120,
  max_backups: 30,
  backup_on_start: true,
  backup_on_close: true,
})
const status = ref<BackupStatus | null>(null)
const saving = ref(false)
const loading = ref(true)
const performing = ref(false)

async function loadConfig() {
  loading.value = true
  try {
    const c = await backupApi.getConfig()
    Object.assign(config, c)
    status.value = await backupApi.getStatus()
  } catch (e: unknown) {
    toast({ title: '加载失败', description: String(e), variant: 'destructive' })
  } finally {
    loading.value = false
  }
}

async function saveConfig() {
  saving.value = true
  try {
    await backupApi.updateConfig({ ...config })
    status.value = await backupApi.getStatus()
    toast({ title: '配置已保存', variant: 'success' })
  } catch (e: unknown) {
    toast({ title: '保存失败', description: String(e), variant: 'destructive' })
  } finally {
    saving.value = false
  }
}

async function performBackup() {
  performing.value = true
  try {
    const info = await backupApi.performBackup()
    status.value = await backupApi.getStatus()
    toast({ title: '备份成功', description: `${info.filename} (${(info.file_size / 1024 / 1024).toFixed(2)} MB)`, variant: 'success' })
  } catch (e: unknown) {
    toast({ title: '备份失败', description: String(e), variant: 'destructive' })
  } finally {
    performing.value = false
  }
}

async function chooseBackupDir() {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const path = await open({ directory: true, multiple: false, title: '选择备份目录' })
    if (path) {
      config.backup_dir = typeof path === 'string' ? path : path
    }
  } catch { /* cancelled */ }
}

function formatSize(mb: number): string {
  if (mb < 1) return `${(mb * 1024).toFixed(0)} KB`
  return `${mb.toFixed(2)} MB`
}

onMounted(loadConfig)
</script>

<template>
  <Card>
    <CardHeader>
      <div class="flex items-center gap-3">
        <div class="flex items-center justify-center w-9 h-9 rounded-lg bg-primary/10">
          <ShieldCheck class="w-4 h-4 text-primary" />
        </div>
        <div class="flex-1">
          <CardTitle class="text-base">自动备份</CardTitle>
          <CardDescription class="mt-0.5">定时自动备份数据库，防止数据丢失</CardDescription>
        </div>
        <Button size="sm" class="gap-2" :disabled="performing" @click="performBackup">
          <Loader2 v-if="performing" class="w-4 h-4 animate-spin" />
          <HardDrive v-else class="w-4 h-4" />
          立即备份
        </Button>
      </div>
    </CardHeader>
    <CardContent>
      <div v-if="loading" class="flex items-center justify-center py-8">
        <Loader2 class="w-5 h-5 animate-spin text-muted-foreground" />
      </div>
      <div v-else class="space-y-5">
        <!-- Enable switch -->
        <div class="flex items-center justify-between">
          <Label class="text-sm font-medium">启用自动备份</Label>
          <button
            :class="config.enabled
              ? 'bg-primary' : 'bg-secondary border border-border'"
            class="relative w-10 h-5 rounded-full transition-colors"
            @click="config.enabled = !config.enabled"
          >
            <span
              :class="config.enabled ? 'translate-x-5' : 'translate-x-0.5'"
              class="absolute top-0.5 w-4 h-4 rounded-full bg-white shadow transition-transform"
            />
          </button>
        </div>

        <template v-if="config.enabled">
          <Separator />

          <!-- Backup directory -->
          <div class="space-y-2">
            <Label class="text-sm font-medium">备份目录</Label>
            <div class="flex items-center gap-2">
              <Input :model-value="config.backup_dir" readonly class="text-xs" />
              <Button variant="outline" size="sm" class="gap-2 shrink-0" @click="chooseBackupDir">
                <FolderOpen class="w-4 h-4" />
                更改
              </Button>
            </div>
            <p class="text-xs text-muted-foreground">可设置为 iCloud / Dropbox 同步目录实现跨设备备份</p>
          </div>

          <!-- Interval -->
          <div class="space-y-2 max-w-xs">
            <Label class="text-sm font-medium flex items-center gap-1.5">
              <Clock class="w-3.5 h-3.5 text-muted-foreground" />
              备份间隔
            </Label>
            <SelectNative :model-value="String(config.interval_minutes)" @update:model-value="config.interval_minutes = Number($event)">
              <option :value="30">每 30 分钟</option>
              <option :value="60">每 1 小时</option>
              <option :value="120">每 2 小时</option>
              <option :value="240">每 4 小时</option>
              <option :value="480">每 8 小时</option>
              <option :value="1440">每 24 小时</option>
            </SelectNative>
          </div>

          <!-- Max backups -->
          <div class="space-y-2 max-w-xs">
            <Label class="text-sm font-medium">最大备份数</Label>
            <Input v-model.number="config.max_backups" type="number" min="5" max="100" />
          </div>

          <!-- Toggles -->
          <div class="space-y-3">
            <div class="flex items-center justify-between">
              <Label class="text-sm font-medium">启动时自动备份</Label>
              <button
                :class="config.backup_on_start ? 'bg-primary' : 'bg-secondary border border-border'"
                class="relative w-10 h-5 rounded-full transition-colors"
                @click="config.backup_on_start = !config.backup_on_start"
              >
                <span
                  :class="config.backup_on_start ? 'translate-x-5' : 'translate-x-0.5'"
                  class="absolute top-0.5 w-4 h-4 rounded-full bg-white shadow transition-transform"
                />
              </button>
            </div>
            <div class="flex items-center justify-between">
              <Label class="text-sm font-medium">关闭时自动备份</Label>
              <button
                :class="config.backup_on_close ? 'bg-primary' : 'bg-secondary border border-border'"
                class="relative w-10 h-5 rounded-full transition-colors"
                @click="config.backup_on_close = !config.backup_on_close"
              >
                <span
                  :class="config.backup_on_close ? 'translate-x-5' : 'translate-x-0.5'"
                  class="absolute top-0.5 w-4 h-4 rounded-full bg-white shadow transition-transform"
                />
              </button>
            </div>
          </div>
        </template>

        <!-- Status -->
        <div v-if="status" class="rounded-lg bg-secondary/50 border border-border/50 p-4 space-y-2">
          <p class="text-xs font-medium text-muted-foreground uppercase tracking-wider">备份状态</p>
          <div class="grid grid-cols-2 gap-3 text-sm">
            <div>
              <span class="text-muted-foreground">上次备份：</span>
              <span class="font-medium text-foreground">{{ status.last_backup_at || '从未备份' }}</span>
            </div>
            <div v-if="status.next_backup_at">
              <span class="text-muted-foreground">下次备份：</span>
              <span class="font-medium text-foreground">{{ status.next_backup_at }}</span>
            </div>
            <div>
              <span class="text-muted-foreground">备份总数：</span>
              <span class="font-medium text-foreground">{{ status.total_backups }} 份</span>
            </div>
            <div>
              <span class="text-muted-foreground">总大小：</span>
              <span class="font-medium text-foreground">{{ formatSize(status.total_size_mb) }}</span>
            </div>
          </div>
        </div>

        <Separator />
        <div class="flex justify-end">
          <Button size="sm" class="gap-2" :disabled="saving" @click="saveConfig">
            <Loader2 v-if="saving" class="w-4 h-4 animate-spin" />
            <Save v-else class="w-4 h-4" />
            保存配置
          </Button>
        </div>
      </div>
    </CardContent>
  </Card>
</template>
