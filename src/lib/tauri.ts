import { invoke } from '@tauri-apps/api/core'
import type {
  Account,
  AccountStats,
  CreateAccountDto,
  UpdateAccountDto,
  CalculationResult,
  TradePlan,
  CreateTradePlanDto,
  UpdateTradePlanDto,
} from '@/types/common'

export async function tauriCommand<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  return invoke<T>(command, args)
}

// Account API
export const accountApi = {
  getAll: () => tauriCommand<Account[]>('get_accounts'),
  getById: (id: string) => tauriCommand<Account>('get_account', { id }),
  create: (dto: CreateAccountDto) => tauriCommand<Account>('create_account', { dto }),
  update: (dto: UpdateAccountDto) => tauriCommand<Account>('update_account', { dto }),
  delete: (id: string) => tauriCommand<void>('delete_account', { id }),
  getStats: (id: string) => tauriCommand<AccountStats>('get_account_stats', { accountId: id }),
}

// Calculation API
export const calcApi = {
  calculate: (params: {
    accountId: string
    entryPrice: number
    stopLoss: number
    takeProfit: number
  }) => tauriCommand<CalculationResult>('calculate_trade', params),
}

// Trade Plan API
export const tradePlanApi = {
  getAll: (accountId: string, status?: string) =>
    tauriCommand<TradePlan[]>('get_trade_plans', { accountId, status: status || null }),
  getById: (id: string) => tauriCommand<TradePlan>('get_trade_plan', { id }),
  create: (dto: CreateTradePlanDto) => tauriCommand<TradePlan>('create_trade_plan', { dto }),
  update: (dto: UpdateTradePlanDto) => tauriCommand<TradePlan>('update_trade_plan', { dto }),
  delete: (id: string) => tauriCommand<void>('delete_trade_plan', { id }),
}
