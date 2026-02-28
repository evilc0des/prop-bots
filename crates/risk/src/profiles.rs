use chrono::NaiveTime;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

/// Configuration for a prop firm's risk rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropFirmProfile {
    pub name: String,
    pub description: Option<String>,
    /// Initial/starting balance for the evaluation.
    pub initial_balance: Decimal,
    /// Maximum loss allowed in a single trading day.
    pub daily_loss_limit: Decimal,
    /// Maximum total drawdown from starting balance (or trailing from high water mark).
    pub max_drawdown: Decimal,
    /// Whether the drawdown is trailing (from high water mark) or fixed (from initial balance).
    pub trailing_drawdown: bool,
    /// Maximum position size (contracts/lots) at any time.
    pub max_position_size: Option<Decimal>,
    /// Maximum number of contracts allowed.
    pub max_contracts: Option<u32>,
    /// Trading allowed only during these hours (UTC).
    pub trading_start_utc: Option<NaiveTime>,
    pub trading_end_utc: Option<NaiveTime>,
    /// Consistency rule: no single day's profit can exceed this % of total profit.
    pub consistency_rule: bool,
    pub consistency_max_pct: Option<Decimal>,
    /// Auto-flatten threshold as a percentage of the daily loss limit (e.g. 0.9 = 90%).
    pub auto_flatten_threshold: Decimal,
}

impl PropFirmProfile {
    /// TopStep 50K evaluation rules.
    pub fn topstep_50k() -> Self {
        Self {
            name: "TopStep 50K".to_string(),
            description: Some("TopStep Trading Combine - $50K account".to_string()),
            initial_balance: dec!(50000),
            daily_loss_limit: dec!(1000),
            max_drawdown: dec!(2000),
            trailing_drawdown: true,
            max_position_size: Some(dec!(5)),
            max_contracts: Some(5),
            trading_start_utc: None,
            trading_end_utc: None,
            consistency_rule: false,
            consistency_max_pct: None,
            auto_flatten_threshold: dec!(0.90),
        }
    }

    /// TopStep 100K evaluation rules.
    pub fn topstep_100k() -> Self {
        Self {
            name: "TopStep 100K".to_string(),
            description: Some("TopStep Trading Combine - $100K account".to_string()),
            initial_balance: dec!(100000),
            daily_loss_limit: dec!(2000),
            max_drawdown: dec!(3000),
            trailing_drawdown: true,
            max_position_size: Some(dec!(10)),
            max_contracts: Some(10),
            trading_start_utc: None,
            trading_end_utc: None,
            consistency_rule: false,
            consistency_max_pct: None,
            auto_flatten_threshold: dec!(0.90),
        }
    }

    /// TopStep 150K evaluation rules.
    pub fn topstep_150k() -> Self {
        Self {
            name: "TopStep 150K".to_string(),
            description: Some("TopStep Trading Combine - $150K account".to_string()),
            initial_balance: dec!(150000),
            daily_loss_limit: dec!(3000),
            max_drawdown: dec!(4500),
            trailing_drawdown: true,
            max_position_size: Some(dec!(15)),
            max_contracts: Some(15),
            trading_start_utc: None,
            trading_end_utc: None,
            consistency_rule: false,
            consistency_max_pct: None,
            auto_flatten_threshold: dec!(0.90),
        }
    }

    /// MFFU (My Funded Futures) evaluation rules.
    pub fn mffu_100k() -> Self {
        Self {
            name: "MFFU 100K".to_string(),
            description: Some("My Funded Futures - $100K evaluation".to_string()),
            initial_balance: dec!(100000),
            daily_loss_limit: dec!(2000),
            max_drawdown: dec!(3000),
            trailing_drawdown: true,
            max_position_size: Some(dec!(10)),
            max_contracts: Some(10),
            trading_start_utc: None,
            trading_end_utc: None,
            consistency_rule: true,
            consistency_max_pct: Some(dec!(30)),
            auto_flatten_threshold: dec!(0.90),
        }
    }

    /// FundingPips evaluation rules.
    pub fn funding_pips_100k() -> Self {
        Self {
            name: "FundingPips 100K".to_string(),
            description: Some("FundingPips - $100K evaluation".to_string()),
            initial_balance: dec!(100000),
            daily_loss_limit: dec!(4000),
            max_drawdown: dec!(8000),
            trailing_drawdown: false,
            max_position_size: None,
            max_contracts: None,
            trading_start_utc: None,
            trading_end_utc: None,
            consistency_rule: false,
            consistency_max_pct: None,
            auto_flatten_threshold: dec!(0.90),
        }
    }
}
