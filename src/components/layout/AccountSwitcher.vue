<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { ChevronDown, Wallet, Settings } from 'lucide-vue-next'
import { useAccountStore } from '@/stores/account'
import { useRouter } from 'vue-router'

const accountStore = useAccountStore()
const router = useRouter()
const open = ref(false)
const wrapperRef = ref<HTMLElement | null>(null)

function handleClickOutside(e: MouseEvent) {
  if (wrapperRef.value && !wrapperRef.value.contains(e.target as Node)) {
    open.value = false
  }
}

const currentAccount = computed(() => accountStore.currentAccount)
const accounts = computed(() => accountStore.accounts)

function toggle() {
  open.value = !open.value
}

async function selectAccount(id: string) {
  await accountStore.selectAccount(id)
  open.value = false
}

function goToSettings() {
  open.value = false
  router.push('/settings')
}

function formatBalance(value: number): string {
  return new Intl.NumberFormat('zh-CN', { minimumFractionDigits: 0, maximumFractionDigits: 0 }).format(value)
}

onMounted(() => document.addEventListener('click', handleClickOutside))
onUnmounted(() => document.removeEventListener('click', handleClickOutside))
</script>

<template>
  <div class="relative" ref="wrapperRef">
    <button
      class="w-full flex items-center gap-2 px-3 py-2 rounded-lg hover:bg-sidebar-accent transition-colors cursor-pointer"
      @click="toggle"
    >
      <div class="flex items-center justify-center w-7 h-7 rounded-full bg-primary/10 shrink-0">
        <Wallet class="w-3.5 h-3.5 text-primary" />
      </div>
      <div v-if="currentAccount" class="flex-1 min-w-0 text-left">
        <p class="text-xs font-medium text-sidebar-foreground truncate">{{ currentAccount.name }}</p>
        <p class="text-[10px] text-sidebar-foreground/50">&yen;{{ formatBalance(currentAccount.balance) }}</p>
      </div>
      <div v-else class="flex-1 min-w-0 text-left">
        <p class="text-xs text-sidebar-foreground/50">选择账户</p>
      </div>
      <ChevronDown :class="['w-3.5 h-3.5 text-sidebar-foreground/40 transition-transform', open && 'rotate-180']" />
    </button>

    <!-- Dropdown -->
    <Transition
      enter-active-class="transition-all duration-150 ease-out"
      enter-from-class="opacity-0 -translate-y-1 scale-95"
      enter-to-class="opacity-100 translate-y-0 scale-100"
      leave-active-class="transition-all duration-100 ease-in"
      leave-from-class="opacity-100 translate-y-0 scale-100"
      leave-to-class="opacity-0 -translate-y-1 scale-95"
    >
      <div v-if="open" class="absolute left-2 right-2 top-full mt-1 rounded-lg border border-border bg-card shadow-lg z-50 py-1">
        <div class="max-h-64 overflow-y-auto">
          <button
            v-for="acc in accounts"
            :key="acc.id"
            class="w-full flex items-center gap-2.5 px-3 py-2 text-left hover:bg-secondary/50 transition-colors cursor-pointer"
            :class="currentAccount?.id === acc.id && 'bg-primary/5'"
            @click="selectAccount(acc.id)"
          >
            <div class="flex items-center justify-center w-6 h-6 rounded-full shrink-0"
              :class="currentAccount?.id === acc.id ? 'bg-primary/15' : 'bg-secondary'">
              <Wallet class="w-3 h-3" :class="currentAccount?.id === acc.id ? 'text-primary' : 'text-muted-foreground'" />
            </div>
            <div class="flex-1 min-w-0">
              <p class="text-xs font-medium text-foreground truncate">{{ acc.name }}</p>
              <p class="text-[10px] text-muted-foreground">&yen;{{ formatBalance(acc.balance) }}</p>
            </div>
            <div v-if="currentAccount?.id === acc.id" class="w-1.5 h-1.5 rounded-full bg-primary shrink-0" />
          </button>
        </div>
        <div class="border-t border-border mt-1 pt-1">
          <button
            class="w-full flex items-center gap-2 px-3 py-2 text-xs text-muted-foreground hover:text-foreground hover:bg-secondary/50 transition-colors cursor-pointer"
            @click="goToSettings"
          >
            <Settings class="w-3.5 h-3.5" />
            管理账户
          </button>
        </div>
      </div>
    </Transition>
  </div>
</template>
