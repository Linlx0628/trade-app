<script setup lang="ts">
import { type HTMLAttributes, computed } from 'vue'
import {
  AlertDialogRoot,
  type AlertDialogEmits,
  type AlertDialogProps,
  useForwardPropsEmits,
} from 'radix-vue'
import { cn } from '@/lib/utils'

const props = defineProps<AlertDialogProps & { class?: HTMLAttributes['class'] }>()
const emits = defineEmits<AlertDialogEmits>()

const delegatedProps = computed(() => {
  const { class: _, ...delegated } = props
  return delegated
})

const forwarded = useForwardPropsEmits(delegatedProps, emits)
</script>

<template>
  <AlertDialogRoot v-bind="forwarded" :class="cn(props.class)">
    <slot />
  </AlertDialogRoot>
</template>
