import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface User {
  id: string
  username: string
  display_name: string
  avatar: string | null
  is_default: boolean
  last_login_at: string | null
  created_at: string
  updated_at: string
}

export interface UserBrief {
  id: string
  username: string
  display_name: string
  avatar: string | null
  is_default: boolean
}

export const useAuthStore = defineStore('auth', () => {
  const currentUser = ref<User | null>(null)
  const users = ref<UserBrief[]>([])
  const loading = ref(false)

  const isLoggedIn = computed(() => currentUser.value !== null)
  const displayName = computed(() => currentUser.value?.display_name || '未登录')

  async function ensureDefaultUser(): Promise<User> {
    const user = await invoke<User>('ensure_default_user')
    currentUser.value = user
    localStorage.setItem('current_user_id', user.id)
    return user
  }

  async function init(): Promise<void> {
    loading.value = true
    try {
      const savedId = localStorage.getItem('current_user_id')
      if (savedId) {
        try {
          currentUser.value = await invoke<User>('get_current_user', { userId: savedId })
          return
        } catch { /* user may not exist */ }
      }
      await ensureDefaultUser()
    } finally {
      loading.value = false
    }
  }

  async function login(username: string): Promise<User> {
    const user = await invoke<User>('login', { username })
    currentUser.value = user
    localStorage.setItem('current_user_id', user.id)
    return user
  }

  async function switchUser(userId: string): Promise<User> {
    const user = await invoke<User>('get_current_user', { userId })
    currentUser.value = user
    localStorage.setItem('current_user_id', user.id)
    return user
  }

  async function createUser(username: string, displayName: string): Promise<User> {
    const user = await invoke<User>('create_user', {
      dto: { username, display_name: displayName },
    })
    await loadUsers()
    return user
  }

  async function updateUser(id: string, displayName: string): Promise<User> {
    const user = await invoke<User>('update_user', {
      dto: { id, display_name: displayName },
    })
    if (currentUser.value?.id === id) {
      currentUser.value = user
    }
    await loadUsers()
    return user
  }

  async function deleteUser(id: string): Promise<void> {
    await invoke('delete_user', { id })
    await loadUsers()
  }

  async function loadUsers(): Promise<void> {
    users.value = await invoke<UserBrief[]>('list_users')
  }

  async function logout(): Promise<void> {
    currentUser.value = null
    localStorage.removeItem('current_user_id')
    await ensureDefaultUser()
  }

  return {
    currentUser,
    users,
    loading,
    isLoggedIn,
    displayName,
    init,
    ensureDefaultUser,
    login,
    switchUser,
    createUser,
    updateUser,
    deleteUser,
    loadUsers,
    logout,
  }
})
