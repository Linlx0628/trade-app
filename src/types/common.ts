export interface PaginatedResult<T> {
  data: T[]
  total: number
  page: number
  pageSize: number
}

export interface Account {
  id: string
  name: string
  balance: number
  risk_ratio: number
  point_value: number
  market_type: 'futures' | 'stock' | 'both'
  created_at: string
  updated_at: string
}

export interface CreateAccountDto {
  name: string
  balance: number
  risk_ratio?: number
  point_value?: number
  market_type?: string
}

export interface UpdateAccountDto {
  id: string
  name?: string
  balance?: number
  risk_ratio?: number
  point_value?: number
  market_type?: string
}

export interface AccountStats {
  balance: number
  total_pnl: number
  total_trades: number
  win_rate: number
  open_positions: number
}

export interface CalculationResult {
  balance: number
  risk_ratio: number
  max_risk_amount: number
  point_value: number
  entry_price: number
  stop_loss: number
  take_profit: number
  stop_loss_points: number
  take_profit_points: number
  suggested_lots: number
  risk_reward_ratio: number
  actual_risk_ratio: number
}

// --- Trade Plan ---

export type TradeDirection = 'long' | 'short'
export type TradePlanStatus = 'planned' | 'executing' | 'completed' | 'cancelled'
export type MarketType = 'futures' | 'stock'

export interface TradePlan {
  id: string
  account_id: string
  symbol: string
  name: string
  direction: TradeDirection
  entry_price: number
  stop_loss: number
  take_profit: number
  lots: number
  market_type: MarketType
  status: TradePlanStatus
  strategy: string
  tags: string
  notes: string
  images: string
  planned_at: string
  created_at: string
  updated_at: string
}

export interface CreateTradePlanDto {
  account_id: string
  symbol: string
  name?: string
  direction: TradeDirection
  entry_price: number
  stop_loss: number
  take_profit: number
  lots?: number
  market_type: MarketType
  strategy?: string
  tags?: string[]
  notes?: string
  planned_at?: string
}

export interface UpdateTradePlanDto {
  id: string
  symbol?: string
  name?: string
  direction?: TradeDirection
  entry_price?: number
  stop_loss?: number
  take_profit?: number
  lots?: number
  market_type?: MarketType
  status?: TradePlanStatus
  strategy?: string
  tags?: string[]
  notes?: string
  planned_at?: string
}
