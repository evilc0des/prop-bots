use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Asset & Instrument
// ---------------------------------------------------------------------------

/// The asset class an instrument belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetClass {
    Futures,
    Cfd,
    Crypto,
}

/// Describes a tradeable instrument (e.g. ES, NQ, BTCUSD).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Instrument {
    pub symbol: String,
    pub asset_class: AssetClass,
    /// Minimum price movement (e.g. 0.25 for ES futures).
    pub tick_size: Decimal,
    /// Dollar value per tick (e.g. $12.50 for ES).
    pub tick_value: Decimal,
    /// Notional value per contract / lot (for margin calculation).
    pub contract_size: Decimal,
    /// Currency the instrument is denominated in.
    pub currency: String,
    /// Exchange or broker-specific identifier.
    pub exchange: Option<String>,
}

// ---------------------------------------------------------------------------
// Market Data
// ---------------------------------------------------------------------------

/// A single OHLCV bar.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bar {
    pub instrument: String,
    pub timestamp: DateTime<Utc>,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
}

/// A single tick (bid/ask/last).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tick {
    pub instrument: String,
    pub timestamp: DateTime<Utc>,
    pub bid: Decimal,
    pub ask: Decimal,
    pub last: Decimal,
    pub volume: Decimal,
}

/// Timeframe for bars.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Timeframe {
    Tick,
    Second(u32),
    Minute(u32),
    Hour(u32),
    Daily,
    Weekly,
    Monthly,
}

// ---------------------------------------------------------------------------
// Orders
// ---------------------------------------------------------------------------

/// Order side.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    Buy,
    Sell,
}

impl Side {
    pub fn opposite(&self) -> Self {
        match self {
            Side::Buy => Side::Sell,
            Side::Sell => Side::Buy,
        }
    }
}

/// The type of order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderType {
    Market,
    Limit,
    Stop,
    StopLimit,
}

/// The lifecycle state of an order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    Pending,
    Submitted,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
}

/// An order to be submitted to a broker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: Uuid,
    pub instrument: String,
    pub side: Side,
    pub order_type: OrderType,
    pub quantity: Decimal,
    pub filled_quantity: Decimal,
    pub price: Option<Decimal>,
    pub stop_price: Option<Decimal>,
    pub status: OrderStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// Optional strategy tag for tracking.
    pub strategy_id: Option<String>,
    /// Broker-assigned ID after submission.
    pub broker_order_id: Option<String>,
}

impl Order {
    /// Create a new market order.
    pub fn market(instrument: &str, side: Side, quantity: Decimal) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            instrument: instrument.to_string(),
            side,
            order_type: OrderType::Market,
            quantity,
            filled_quantity: Decimal::ZERO,
            price: None,
            stop_price: None,
            status: OrderStatus::Pending,
            created_at: now,
            updated_at: now,
            strategy_id: None,
            broker_order_id: None,
        }
    }

    /// Create a new limit order.
    pub fn limit(instrument: &str, side: Side, quantity: Decimal, price: Decimal) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            instrument: instrument.to_string(),
            side,
            order_type: OrderType::Limit,
            quantity,
            filled_quantity: Decimal::ZERO,
            price: Some(price),
            stop_price: None,
            status: OrderStatus::Pending,
            created_at: now,
            updated_at: now,
            strategy_id: None,
            broker_order_id: None,
        }
    }

    /// Create a new stop order.
    pub fn stop(instrument: &str, side: Side, quantity: Decimal, stop_price: Decimal) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            instrument: instrument.to_string(),
            side,
            order_type: OrderType::Stop,
            quantity,
            filled_quantity: Decimal::ZERO,
            price: None,
            stop_price: Some(stop_price),
            status: OrderStatus::Pending,
            created_at: now,
            updated_at: now,
            strategy_id: None,
            broker_order_id: None,
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(
            self.status,
            OrderStatus::Pending | OrderStatus::Submitted | OrderStatus::PartiallyFilled
        )
    }
}

// ---------------------------------------------------------------------------
// Fill
// ---------------------------------------------------------------------------

/// Represents a single fill (partial or full) of an order.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fill {
    pub order_id: Uuid,
    pub instrument: String,
    pub side: Side,
    pub quantity: Decimal,
    pub price: Decimal,
    pub commission: Decimal,
    pub timestamp: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Position
// ---------------------------------------------------------------------------

/// Represents a currently open position.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub instrument: String,
    pub side: Side,
    pub quantity: Decimal,
    pub avg_entry_price: Decimal,
    pub unrealized_pnl: Decimal,
    pub realized_pnl: Decimal,
    pub opened_at: DateTime<Utc>,
    pub strategy_id: Option<String>,
}

impl Position {
    /// Update unrealized PnL based on the current market price.
    pub fn update_pnl(&mut self, current_price: Decimal, tick_size: Decimal, tick_value: Decimal) {
        let price_diff = match self.side {
            Side::Buy => current_price - self.avg_entry_price,
            Side::Sell => self.avg_entry_price - current_price,
        };
        let ticks = price_diff / tick_size;
        self.unrealized_pnl = ticks * tick_value * self.quantity;
    }
}

// ---------------------------------------------------------------------------
// Trade (closed position)
// ---------------------------------------------------------------------------

/// A completed (closed) trade with realized PnL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: Uuid,
    pub instrument: String,
    pub side: Side,
    pub quantity: Decimal,
    pub entry_price: Decimal,
    pub exit_price: Decimal,
    pub pnl: Decimal,
    pub commission: Decimal,
    pub entry_time: DateTime<Utc>,
    pub exit_time: DateTime<Utc>,
    pub strategy_id: Option<String>,
}

impl Trade {
    pub fn net_pnl(&self) -> Decimal {
        self.pnl - self.commission
    }
}

// ---------------------------------------------------------------------------
// Account
// ---------------------------------------------------------------------------

/// Snapshot of the account state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountState {
    pub balance: Decimal,
    pub equity: Decimal,
    pub unrealized_pnl: Decimal,
    pub realized_pnl: Decimal,
    pub daily_pnl: Decimal,
    pub margin_used: Decimal,
    pub margin_available: Decimal,
    pub open_positions: usize,
    /// The highest equity reached (for trailing drawdown).
    pub high_water_mark: Decimal,
    pub timestamp: DateTime<Utc>,
}

impl AccountState {
    pub fn new(starting_balance: Decimal) -> Self {
        let now = Utc::now();
        Self {
            balance: starting_balance,
            equity: starting_balance,
            unrealized_pnl: Decimal::ZERO,
            realized_pnl: Decimal::ZERO,
            daily_pnl: Decimal::ZERO,
            margin_used: Decimal::ZERO,
            margin_available: starting_balance,
            open_positions: 0,
            high_water_mark: starting_balance,
            timestamp: now,
        }
    }

    pub fn current_drawdown(&self) -> Decimal {
        self.high_water_mark - self.equity
    }

    pub fn drawdown_percent(&self) -> Decimal {
        if self.high_water_mark.is_zero() {
            Decimal::ZERO
        } else {
            (self.current_drawdown() / self.high_water_mark) * Decimal::ONE_HUNDRED
        }
    }
}

// ---------------------------------------------------------------------------
// Signal
// ---------------------------------------------------------------------------

/// A trading signal emitted by a strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    pub id: Uuid,
    pub instrument: String,
    pub action: SignalAction,
    pub quantity: Option<Decimal>,
    pub price: Option<Decimal>,
    pub strategy_id: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalAction {
    BuyEntry,
    SellEntry,
    ExitLong,
    ExitShort,
    ExitAll,
}

// ---------------------------------------------------------------------------
// Backtest Results
// ---------------------------------------------------------------------------

/// Aggregate performance metrics from a backtest run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestResult {
    pub id: Uuid,
    pub strategy_id: String,
    pub instrument: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub initial_balance: Decimal,
    pub final_balance: Decimal,
    pub total_trades: usize,
    pub winning_trades: usize,
    pub losing_trades: usize,
    pub gross_profit: Decimal,
    pub gross_loss: Decimal,
    pub net_profit: Decimal,
    pub max_drawdown: Decimal,
    pub max_drawdown_percent: Decimal,
    pub win_rate: Decimal,
    pub profit_factor: Decimal,
    pub sharpe_ratio: Decimal,
    pub sortino_ratio: Decimal,
    pub avg_trade_pnl: Decimal,
    pub avg_winner: Decimal,
    pub avg_loser: Decimal,
    pub total_commission: Decimal,
    /// Per-bar equity snapshots.
    pub equity_curve: Vec<EquityPoint>,
    /// All trades executed.
    pub trades: Vec<Trade>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquityPoint {
    pub timestamp: DateTime<Utc>,
    pub equity: Decimal,
    pub drawdown: Decimal,
}
