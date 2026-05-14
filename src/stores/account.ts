import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { accountApi } from '@/lib/tauri'
import type { Account, CreateAccountDto, UpdateAccountDto, AccountStats } from '@/types/common'

export const useAccountStore = defineStore('account', () => {
  const accounts = ref<Account[]>([])
  const currentAccount = ref<Account | null>(null)
  const currentAccountStats = ref<AccountStats | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  const accountCount = computed(() => accounts.value.length)
  const totalBalance = computed(() =>
    accounts.value.reduce((sum, acc) => sum + acc.balance, 0),
  )

  async function fetchAccounts() {
    loading.value = true
    error.value = null
    try {
      accounts.value = await accountApi.getAll()
    } catch (e) {
      error.value = e instanceof Error ? e.message : '获取账户列表失败'
    } finally {
      loading.value = false
    }
  }

  async function fetchAccountById(id: string) {
    loading.value = true
    error.value = null
    try {
      currentAccount.value = await accountApi.getById(id)
    } catch (e) {
      error.value = e instanceof Error ? e.message : '获取账户详情失败'
    } finally {
      loading.value = false
    }
  }

  async function selectAccount(id: string) {
    const account = accounts.value.find((a) => a.id === id)
    if (account) {
      currentAccount.value = account
    } else {
      try {
        currentAccount.value = await accountApi.getById(id)
      } catch (e) {
        error.value = e instanceof Error ? e.message : '获取账户详情失败'
        return
      }
    }
    // Fetch stats for the selected account
    await fetchAccountStats(id)
  }

  async function fetchAccountStats(id: string) {
    try {
      currentAccountStats.value = await accountApi.getStats(id)
    } catch (e) {
      error.value = e instanceof Error ? e.message : '获取账户统计失败'
    }
  }

  async function createAccount(dto: CreateAccountDto) {
    loading.value = true
    error.value = null
    try {
      const newAccount = await accountApi.create(dto)
      accounts.value.push(newAccount)
      return newAccount
    } catch (e) {
      error.value = e instanceof Error ? e.message : '创建账户失败'
      throw e
    } finally {
      loading.value = false
    }
  }

  async function updateAccount(dto: UpdateAccountDto) {
    loading.value = true
    error.value = null
    try {
      const updated = await accountApi.update(dto)
      const index = accounts.value.findIndex((a) => a.id === dto.id)
      if (index > -1) {
        accounts.value[index] = updated
      }
      if (currentAccount.value?.id === dto.id) {
        currentAccount.value = updated
      }
      return updated
    } catch (e) {
      error.value = e instanceof Error ? e.message : '更新账户失败'
      throw e
    } finally {
      loading.value = false
    }
  }

  async function deleteAccount(id: string) {
    loading.value = true
    error.value = null
    try {
      await accountApi.delete(id)
      accounts.value = accounts.value.filter((a) => a.id !== id)
      if (currentAccount.value?.id === id) {
        currentAccount.value = null
        currentAccountStats.value = null
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : '删除账户失败'
      throw e
    } finally {
      loading.value = false
    }
  }

  return {
    accounts,
    currentAccount,
    currentAccountStats,
    loading,
    error,
    accountCount,
    totalBalance,
    fetchAccounts,
    fetchAccountById,
    fetchAccountStats,
    selectAccount,
    createAccount,
    updateAccount,
    deleteAccount,
  }
})
