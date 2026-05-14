//! 交易计算引擎
//!
//! 提供交易相关的核心计算功能：风险金额、建议手数、盈亏比等

use serde::{Deserialize, Serialize};

/// 计算结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculationResult {
    /// 账户余额
    pub balance: f64,
    /// 风险比例
    pub risk_ratio: f64,
    /// 最大风险金额
    pub max_risk_amount: f64,
    /// 每点价值
    pub point_value: f64,
    /// 入场价格
    pub entry_price: f64,
    /// 止损价格
    pub stop_loss: f64,
    /// 止盈价格
    pub take_profit: f64,
    /// 止损点数（绝对值）
    pub stop_loss_points: f64,
    /// 止盈点数（绝对值）
    pub take_profit_points: f64,
    /// 建议手数
    pub suggested_lots: f64,
    /// 盈亏比
    pub risk_reward_ratio: f64,
    /// 实际风险比例
    pub actual_risk_ratio: f64,
}

/// 交易计算服务
///
/// 提供交易风险管理相关的核心计算
pub struct CalculationService;

impl CalculationService {
    /// 计算最大风险金额
    ///
    /// 根据账户余额和风险比例计算单笔交易最大可承受亏损金额
    ///
    /// # 参数
    /// - `balance`: 账户余额
    /// - `risk_ratio`: 风险比例（如 0.02 表示 2%）
    ///
    /// # 返回
    /// 最大风险金额 = 余额 x 风险比例
    pub fn max_risk_amount(balance: f64, risk_ratio: f64) -> f64 {
        (balance * risk_ratio).max(0.0)
    }

    /// 计算建议手数
    ///
    /// 根据最大风险金额、入场价、止损价和每点价值计算建议开仓手数
    ///
    /// # 参数
    /// - `max_risk`: 最大风险金额
    /// - `entry_price`: 入场价格
    /// - `stop_loss`: 止损价格
    /// - `point_value`: 每点价值（合约乘数）
    ///
    /// # 返回
    /// 建议手数 = 最大风险金额 / (止损点数 x 每点价值)
    pub fn suggested_lots(
        max_risk: f64,
        entry_price: f64,
        stop_loss: f64,
        point_value: f64,
    ) -> f64 {
        let stop_loss_points = (entry_price - stop_loss).abs();
        if stop_loss_points <= 0.0 || point_value <= 0.0 {
            return 0.0;
        }
        let lots = max_risk / (stop_loss_points * point_value);
        // 向下取整到整手（至少显示小数后保留精度）
        (lots * 100.0).floor() / 100.0
    }

    /// 计算盈亏比（Risk-Reward Ratio）
    ///
    /// # 参数
    /// - `entry_price`: 入场价格
    /// - `stop_loss`: 止损价格
    /// - `take_profit`: 止盈价格
    ///
    /// # 返回
    /// 盈亏比 = 止盈点数 / 止损点数
    pub fn risk_reward_ratio(entry_price: f64, stop_loss: f64, take_profit: f64) -> f64 {
        let stop_loss_points = (entry_price - stop_loss).abs();
        let take_profit_points = (take_profit - entry_price).abs();

        if stop_loss_points <= 0.0 {
            return 0.0;
        }

        take_profit_points / stop_loss_points
    }

    /// 计算实际风险比例
    ///
    /// 根据实际交易参数计算该笔交易占账户的实际风险比例
    ///
    /// # 参数
    /// - `entry_price`: 入场价格
    /// - `stop_loss`: 止损价格
    /// - `lots`: 手数
    /// - `point_value`: 每点价值
    /// - `balance`: 账户余额
    ///
    /// # 返回
    /// 实际风险比例 = (止损点数 x 手数 x 每点价值) / 账户余额
    pub fn actual_risk_ratio(
        entry_price: f64,
        stop_loss: f64,
        lots: f64,
        point_value: f64,
        balance: f64,
    ) -> f64 {
        if balance <= 0.0 {
            return 0.0;
        }
        let stop_loss_amount = (entry_price - stop_loss).abs() * lots * point_value;
        stop_loss_amount / balance
    }

    /// 执行完整交易计算
    ///
    /// 综合计算所有交易参数，返回完整的计算结果
    ///
    /// # 参数
    /// - `balance`: 账户余额
    /// - `risk_ratio`: 风险比例
    /// - `point_value`: 每点价值
    /// - `entry_price`: 入场价格
    /// - `stop_loss`: 止损价格
    /// - `take_profit`: 止盈价格
    ///
    /// # 返回
    /// 包含所有计算结果的 CalculationResult
    pub fn calculate(
        balance: f64,
        risk_ratio: f64,
        point_value: f64,
        entry_price: f64,
        stop_loss: f64,
        take_profit: f64,
    ) -> CalculationResult {
        // 验证输入
        if entry_price <= 0.0 {
            return CalculationResult {
                balance,
                risk_ratio,
                max_risk_amount: 0.0,
                point_value,
                entry_price,
                stop_loss,
                take_profit,
                stop_loss_points: 0.0,
                take_profit_points: 0.0,
                suggested_lots: 0.0,
                risk_reward_ratio: 0.0,
                actual_risk_ratio: 0.0,
            };
        }

        let max_risk_amount = Self::max_risk_amount(balance, risk_ratio);
        let stop_loss_points = (entry_price - stop_loss).abs();
        let take_profit_points = (take_profit - entry_price).abs();
        let suggested_lots = Self::suggested_lots(max_risk_amount, entry_price, stop_loss, point_value);
        let risk_reward_ratio = Self::risk_reward_ratio(entry_price, stop_loss, take_profit);
        let actual_risk_ratio =
            Self::actual_risk_ratio(entry_price, stop_loss, suggested_lots, point_value, balance);

        CalculationResult {
            balance,
            risk_ratio,
            max_risk_amount,
            point_value,
            entry_price,
            stop_loss,
            take_profit,
            stop_loss_points,
            take_profit_points,
            suggested_lots,
            risk_reward_ratio,
            actual_risk_ratio,
        }
    }
}
