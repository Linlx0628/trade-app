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
  TradeLog,
  CreateTradeLogDto,
  UpdateTradeLogDto,
  TradeSummary,
  CreateTradeSummaryDto,
  UpdateTradeSummaryDto,
  AiChatRequest,
  AiChatResponse,
  TradeTemplate,
  CreateTradeTemplateDto,
  UpdateTradeTemplateDto,
  CreateTemplateFromPlanDto,
  CreatePlanFromTemplateDto,
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
  execute: (planId: string, entryTime?: string) =>
    tauriCommand<TradeLog>('execute_trade_plan', { planId, entryTime: entryTime || null }),
}

// Trade Log API
export const tradeLogApi = {
  getAll: (accountId: string, status?: string) =>
    tauriCommand<TradeLog[]>('get_trade_logs', { accountId, status: status || null }),
  getById: (id: string) => tauriCommand<TradeLog>('get_trade_log', { id }),
  create: (dto: CreateTradeLogDto) => tauriCommand<TradeLog>('create_trade_log', { dto }),
  update: (dto: UpdateTradeLogDto) => tauriCommand<TradeLog>('update_trade_log', { dto }),
  delete: (id: string) => tauriCommand<void>('delete_trade_log', { id }),
}

// Trade Summary API
export const tradeSummaryApi = {
  getAll: (accountId: string, summaryType?: string) =>
    tauriCommand<TradeSummary[]>('get_trade_summaries', { accountId, summaryType: summaryType || null }),
  getById: (id: string) => tauriCommand<TradeSummary>('get_trade_summary', { id }),
  create: (dto: CreateTradeSummaryDto) => tauriCommand<TradeSummary>('create_trade_summary', { dto }),
  update: (dto: UpdateTradeSummaryDto) => tauriCommand<TradeSummary>('update_trade_summary', { dto }),
  delete: (id: string) => tauriCommand<void>('delete_trade_summary', { id }),
}

// AI API
export const aiApi = {
  chat: (req: AiChatRequest) => tauriCommand<AiChatResponse>('ai_chat', { req }),
}

// Trade Template API
export const tradeTemplateApi = {
  getAll: (accountId: string) =>
    tauriCommand<TradeTemplate[]>('get_trade_templates', { accountId }),
  getById: (id: string) =>
    tauriCommand<TradeTemplate>('get_trade_template', { id }),
  create: (dto: CreateTradeTemplateDto) =>
    tauriCommand<TradeTemplate>('create_trade_template', { dto }),
  update: (dto: UpdateTradeTemplateDto) =>
    tauriCommand<TradeTemplate>('update_trade_template', { dto }),
  delete: (id: string) =>
    tauriCommand<void>('delete_trade_template', { id }),
  createFromPlan: (dto: CreateTemplateFromPlanDto) =>
    tauriCommand<TradeTemplate>('create_template_from_plan', { dto }),
  createPlanFromTemplate: (dto: CreatePlanFromTemplateDto) =>
    tauriCommand<TradePlan>('create_plan_from_template', { dto }),
}
