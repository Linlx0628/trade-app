import { ref } from 'vue'

export interface ToastAction {
  label: string
  onClick: () => void
}

export interface Toast {
  id: string
  title?: string
  description?: string
  variant?: 'default' | 'destructive' | 'success'
  action?: ToastAction
  duration?: number
}

const toasts = ref<Toast[]>([])
let toastCount = 0

function addToast(toast: Omit<Toast, 'id'>): Toast {
  const id = `toast-${++toastCount}`
  const newToast: Toast = { id, duration: 5000, variant: 'default', ...toast }

  toasts.value.push(newToast)

  if (newToast.duration && newToast.duration > 0) {
    setTimeout(() => {
      dismiss(id)
    }, newToast.duration)
  }

  return newToast
}

function dismiss(id: string) {
  const index = toasts.value.findIndex((t) => t.id === id)
  if (index > -1) {
    toasts.value.splice(index, 1)
  }
}

function dismissAll() {
  toasts.value.splice(0)
}

export function useToast() {
  return {
    toasts,
    toast: addToast,
    dismiss,
    dismissAll,
  }
}
