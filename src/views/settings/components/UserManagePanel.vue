<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { User, Plus, Pencil, Trash2, Loader2, CheckCircle2 } from 'lucide-vue-next'
import { useAuthStore, type UserBrief } from '@/stores/auth'
import { useToast } from '@/components/ui/toast'
import { Card, CardHeader, CardTitle, CardDescription, CardContent } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Badge } from '@/components/ui/badge'
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

const authStore = useAuthStore()
const { toast } = useToast()

const showForm = ref(false)
const editing = ref<UserBrief | null>(null)
const formSubmitting = ref(false)
const deleteTarget = ref<UserBrief | null>(null)
const switchingId = ref<string | null>(null)

const form = reactive({
  username: '',
  displayName: '',
})

function resetForm() {
  form.username = ''
  form.displayName = ''
  editing.value = null
}

function openCreate() {
  resetForm()
  showForm.value = true
}

function openEdit(user: UserBrief) {
  editing.value = user
  form.username = user.username
  form.displayName = user.display_name
  showForm.value = true
}

async function handleSubmit() {
  if (!form.displayName.trim()) {
    toast({ title: '请输入显示名称', variant: 'destructive' })
    return
  }

  formSubmitting.value = true
  try {
    if (editing.value) {
      await authStore.updateUser(editing.value.id, form.displayName.trim())
      toast({ title: '已更新', variant: 'success' })
    } else {
      if (!form.username.trim()) {
        toast({ title: '请输入用户名', variant: 'destructive' })
        formSubmitting.value = false
        return
      }
      await authStore.createUser(form.username.trim(), form.displayName.trim())
      toast({ title: '已创建', variant: 'success' })
    }
    showForm.value = false
    resetForm()
  } catch (e: unknown) {
    toast({ title: '操作失败', description: String(e), variant: 'destructive' })
  } finally {
    formSubmitting.value = false
  }
}

async function handleSwitch(id: string) {
  switchingId.value = id
  try {
    await authStore.switchUser(id)
    toast({ title: '已切换用户', variant: 'success' })
  } catch (e: unknown) {
    toast({ title: '切换失败', description: String(e), variant: 'destructive' })
  } finally {
    switchingId.value = null
  }
}

async function handleDelete() {
  if (!deleteTarget.value) return
  try {
    await authStore.deleteUser(deleteTarget.value.id)
    toast({ title: '已删除', variant: 'success' })
  } catch (e: unknown) {
    toast({ title: '删除失败', description: String(e), variant: 'destructive' })
  } finally {
    deleteTarget.value = null
  }
}

onMounted(() => authStore.loadUsers())
</script>

<template>
  <Card>
    <CardHeader>
      <div class="flex items-center gap-3">
        <div class="flex items-center justify-center w-9 h-9 rounded-lg bg-primary/10">
          <User class="w-4 h-4 text-primary" />
        </div>
        <div class="flex-1">
          <CardTitle class="text-base">用户管理</CardTitle>
          <CardDescription class="mt-0.5">管理本地用户，支持多用户切换</CardDescription>
        </div>
        <Button size="sm" class="gap-2" @click="openCreate">
          <Plus class="w-4 h-4" />新增用户
        </Button>
      </div>
    </CardHeader>
    <CardContent>
      <!-- Form -->
      <div v-if="showForm" class="space-y-3 mb-4 p-3 rounded-lg border border-border/50">
        <div class="grid grid-cols-2 gap-3">
          <div class="space-y-1.5" v-if="!editing">
            <Label class="text-xs font-medium">用户名</Label>
            <Input v-model="form.username" placeholder="英文标识" class="text-sm" />
          </div>
          <div class="space-y-1.5" :class="!editing && 'col-span-2'">
            <Label class="text-xs font-medium">显示名称</Label>
            <Input v-model="form.displayName" placeholder="显示名称" class="text-sm" />
          </div>
        </div>
        <div class="flex justify-end gap-2">
          <Button variant="outline" size="sm" @click="showForm = false; resetForm()">取消</Button>
          <Button size="sm" :disabled="formSubmitting" @click="handleSubmit">
            <Loader2 v-if="formSubmitting" class="w-4 h-4 animate-spin" />
            {{ editing ? '保存' : '创建' }}
          </Button>
        </div>
      </div>

      <!-- User list -->
      <div class="space-y-2">
        <div
          v-for="u in authStore.users"
          :key="u.id"
          class="flex items-center gap-3 p-3 rounded-lg border border-border/50 hover:bg-secondary/30 transition-colors"
        >
          <div class="flex items-center justify-center w-8 h-8 rounded-full"
            :class="authStore.currentUser?.id === u.id ? 'bg-primary/15' : 'bg-secondary'">
            <User class="w-4 h-4" :class="authStore.currentUser?.id === u.id ? 'text-primary' : 'text-muted-foreground'" />
          </div>
          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-2">
              <span class="text-sm font-medium text-foreground">{{ u.display_name }}</span>
              <Badge v-if="u.is_default" variant="secondary" class="text-[10px] h-4">默认</Badge>
              <Badge v-if="authStore.currentUser?.id === u.id" variant="outline" class="text-[10px] h-4 text-profit">
                <CheckCircle2 class="w-2.5 h-2.5 mr-0.5" />当前
              </Badge>
            </div>
            <p class="text-xs text-muted-foreground">@{{ u.username }}</p>
          </div>
          <div class="flex items-center gap-1">
            <Button v-if="authStore.currentUser?.id !== u.id" variant="ghost" size="sm" class="h-7 px-2 text-xs"
              :disabled="switchingId === u.id" @click="handleSwitch(u.id)">
              <Loader2 v-if="switchingId === u.id" class="w-3.5 h-3.5 animate-spin" />
              切换
            </Button>
            <Button variant="ghost" size="icon" class="h-7 w-7" @click="openEdit(u)">
              <Pencil class="w-3.5 h-3.5" />
            </Button>
            <AlertDialog v-if="!u.is_default">
              <AlertDialogTrigger as-child>
                <Button variant="ghost" size="icon" class="h-7 w-7 hover:text-loss" @click.stop="deleteTarget = u">
                  <Trash2 class="w-3.5 h-3.5" />
                </Button>
              </AlertDialogTrigger>
              <AlertDialogContent>
                <AlertDialogHeader>
                  <AlertDialogTitle>确认删除用户</AlertDialogTitle>
                  <AlertDialogDescription>确定要删除用户「{{ deleteTarget?.display_name }}」吗？该用户的数据不会被删除。</AlertDialogDescription>
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
    </CardContent>
  </Card>
</template>
