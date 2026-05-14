<script setup lang="ts">
import { type HTMLAttributes, computed } from 'vue'
import { AlertDialogTrigger, type AlertDialogTriggerProps } from 'radix-vue'
import { cn } from '@/lib/utils'

const props = defineProps<AlertDialogTriggerProps & { class?: HTMLAttributes['class'] }>()

const delegatedProps = computed(() => {
  const { class: _, ...delegated } = props
  return delegated
})
</script>

<template>
  <AlertDialogTrigger v-bind="delegatedProps" :class="cn(props.class)">
    <slot />
  </AlertDialogTrigger>
</template>
