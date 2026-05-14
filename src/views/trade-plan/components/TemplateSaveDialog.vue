<script setup lang="ts">
import { ref } from 'vue'
import { X, Save, Loader2 } from 'lucide-vue-next'
import { useToast } from '@/components/ui/toast'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import { Card, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'
import type { TradePlan } from '@/types/common'

const props = defineProps<{
  plan: TradePlan
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'saved'): void
}>()

const { toast } = useToast()

const templateName = ref(`${props.plan.symbol} 模板`)
const templateDescription = ref('')
const saving = ref(false)

async function handleSave() {
  if (!templateName.value.trim()) {
    toast({ title: '请输入模板名称', variant: 'destructive' })
    return
  }
  saving.value = true
  try {
    const { tradeTemplateApi } = await import('@/lib/tauri')
    await tradeTemplateApi.createFromPlan({
      plan_id: props.plan.id,
      template_name: templateName.value.trim(),
      template_description: templateDescription.value.trim(),
    })
    toast({ title: '模板已保存', variant: 'success' })
    emit('saved')
  } catch {
    toast({ title: '保存失败', variant: 'destructive' })
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center">
    <div class="fixed inset-0 bg-black/30 backdrop-blur-sm" @click="emit('close')" />
    <Card class="relative w-full max-w-md shadow-2xl border-ring/20 mx-4">
      <CardHeader class="pb-3 border-b border-border/50">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-3">
            <div class="flex items-center justify-center w-9 h-9 rounded-lg bg-primary/10">
              <Save class="w-4 h-4 text-primary" />
            </div>
            <div>
              <CardTitle class="text-base">保存为模板</CardTitle>
              <CardDescription class="mt-0.5">将当前计划参数保存为可复用模板</CardDescription>
            </div>
          </div>
          <Button variant="ghost" size="icon" class="h-8 w-8" @click="emit('close')">
            <X class="w-4 h-4" />
          </Button>
        </div>
      </CardHeader>
      <div class="p-5 space-y-4">
        <!-- Preview -->
        <div class="rounded-lg bg-secondary/50 p-3 space-y-1">
          <p class="text-xs text-muted-foreground">将从计划中提取：</p>
          <div class="flex flex-wrap gap-2 text-xs">
            <span class="bg-background px-2 py-0.5 rounded">{{ plan.symbol }}</span>
            <span class="bg-background px-2 py-0.5 rounded">{{ plan.direction === 'long' ? '做多' : '做空' }}</span>
            <span class="bg-background px-2 py-0.5 rounded">{{ plan.market_type === 'futures' ? '期货' : '股票' }}</span>
            <span class="bg-background px-2 py-0.5 rounded">止损止盈比例</span>
            <span v-if="plan.strategy" class="bg-background px-2 py-0.5 rounded">策略</span>
          </div>
        </div>

        <div class="space-y-2">
          <Label class="text-sm font-medium">模板名称 <span class="text-loss">*</span></Label>
          <Input v-model="templateName" placeholder="例：IF 日内做多模板" />
        </div>
        <div class="space-y-2">
          <Label class="text-sm font-medium">描述</Label>
          <Textarea v-model="templateDescription" placeholder="适用场景、注意事项..." class="min-h-[60px]" />
        </div>
      </div>
      <div class="flex items-center justify-end gap-2 px-5 py-4 border-t border-border/50">
        <Button variant="outline" size="sm" @click="emit('close')">取消</Button>
        <Button size="sm" class="gap-2" :disabled="saving" @click="handleSave">
          <Loader2 v-if="saving" class="w-4 h-4 animate-spin" />
          <Save v-else class="w-4 h-4" />
          保存模板
        </Button>
      </div>
    </Card>
  </div>
</template>
