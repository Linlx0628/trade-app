<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { FolderArchive, Loader2, RotateCcw, Trash2, RefreshCw } from 'lucide-vue-next'
import { useToast } from '@/components/ui/toast'
import { Card, CardHeader, CardTitle, CardDescription, CardContent } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
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
import { backupApi, type BackupInfo } from '@/lib/tauri'

const { toast } = useToast()

const backups = ref<BackupInfo[]>([])
const loading = ref(false)
const restoring = ref<string | null>(null)
const deleting = ref<string | null>(null)
const restoreTarget = ref<string | null>(null)

async function loadBackups() {
  loading.value = true
  try {
    backups.value = await backupApi.listBackups()
  } catch (e: unknown) {
    toast({ title: '加载失败', description: String(e), variant: 'destructive' })
  } finally {
    loading.value = false
  }
}

async function handleRestore(filename: string) {
  restoring.value = filename
  try {
    await backupApi.restoreBackup(filename)
    toast({ title: '恢复成功', description: '已从备份恢复数据，建议重启应用以加载最新数据', variant: 'success' })
    await loadBackups()
  } catch (e: unknown) {
    toast({ title: '恢复失败', description: String(e), variant: 'destructive' })
  } finally {
    restoring.value = null
    restoreTarget.value = null
  }
}

async function handleDelete(filename: string) {
  deleting.value = filename
  try {
    await backupApi.deleteBackup(filename)
    toast({ title: '已删除', description: `备份 ${filename} 已删除`, variant: 'success' })
    await loadBackups()
  } catch (e: unknown) {
    toast({ title: '删除失败', description: String(e), variant: 'destructive' })
  } finally {
    deleting.value = null
  }
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / 1024 / 1024).toFixed(2)} MB`
}

function extractTime(filename: string): string {
  const match = filename.match(/trademind_backup_(\d{8})_(\d{6})\.db/)
  if (match) {
    const [, date, time] = match
    return `${date.slice(0, 4)}-${date.slice(4, 6)}-${date.slice(6, 8)} ${time.slice(0, 2)}:${time.slice(2, 4)}:${time.slice(4, 6)}`
  }
  return filename
}

onMounted(loadBackups)
</script>

<template>
  <Card>
    <CardHeader>
      <div class="flex items-center gap-3">
        <div class="flex items-center justify-center w-9 h-9 rounded-lg bg-primary/10">
          <FolderArchive class="w-4 h-4 text-primary" />
        </div>
        <div class="flex-1">
          <CardTitle class="text-base">备份列表</CardTitle>
          <CardDescription class="mt-0.5">查看和管理备份文件，可从历史备份恢复数据</CardDescription>
        </div>
        <Button variant="outline" size="sm" class="gap-2" :disabled="loading" @click="loadBackups">
          <RefreshCw :class="['w-4 h-4', loading && 'animate-spin']" />
          刷新
        </Button>
      </div>
    </CardHeader>
    <CardContent>
      <div v-if="loading && backups.length === 0" class="flex items-center justify-center py-8">
        <Loader2 class="w-5 h-5 animate-spin text-muted-foreground" />
      </div>
      <div v-else-if="backups.length === 0" class="text-center py-8 text-sm text-muted-foreground">
        暂无备份文件
      </div>
      <div v-else class="space-y-2">
        <div
          v-for="b in backups"
          :key="b.filename"
          class="flex items-center gap-3 p-3 rounded-lg border border-border/50 hover:bg-secondary/30 transition-colors"
        >
          <div class="flex-1 min-w-0">
            <p class="text-sm font-medium text-foreground">{{ extractTime(b.filename) }}</p>
            <p class="text-xs text-muted-foreground mt-0.5">{{ formatSize(b.file_size) }}</p>
          </div>
          <div class="flex items-center gap-1">
            <Button
              variant="ghost"
              size="sm"
              class="h-7 px-2 text-xs gap-1"
              :disabled="restoring === b.filename"
              @click="restoreTarget = b.filename"
            >
              <Loader2 v-if="restoring === b.filename" class="w-3.5 h-3.5 animate-spin" />
              <RotateCcw v-else class="w-3.5 h-3.5" />
              恢复
            </Button>
            <AlertDialog>
              <AlertDialogTrigger as-child>
                <Button
                  variant="ghost"
                  size="icon"
                  class="h-7 w-7 hover:text-loss"
                  :disabled="deleting === b.filename"
                >
                  <Loader2 v-if="deleting === b.filename" class="w-3.5 h-3.5 animate-spin" />
                  <Trash2 v-else class="w-3.5 h-3.5" />
                </Button>
              </AlertDialogTrigger>
              <AlertDialogContent>
                <AlertDialogHeader>
                  <AlertDialogTitle>确认删除备份</AlertDialogTitle>
                  <AlertDialogDescription>确定要删除备份 {{ extractTime(b.filename) }} 吗？此操作无法撤销。</AlertDialogDescription>
                </AlertDialogHeader>
                <AlertDialogFooter>
                  <AlertDialogCancel>取消</AlertDialogCancel>
                  <AlertDialogAction @click="handleDelete(b.filename)">确认删除</AlertDialogAction>
                </AlertDialogFooter>
              </AlertDialogContent>
            </AlertDialog>
          </div>
        </div>
      </div>

      <!-- Restore confirmation dialog (separate from delete) -->
      <AlertDialog :open="!!restoreTarget" @update:open="(v) => { if (!v) restoreTarget = null }">
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>确认恢复备份</AlertDialogTitle>
            <AlertDialogDescription>
              恢复备份将覆盖当前所有数据。建议先创建一次备份再恢复。确定要从 {{ restoreTarget ? extractTime(restoreTarget) : '' }} 恢复吗？
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel @click="restoreTarget = null">取消</AlertDialogCancel>
            <AlertDialogAction @click="restoreTarget && handleRestore(restoreTarget)">确认恢复</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </CardContent>
  </Card>
</template>
