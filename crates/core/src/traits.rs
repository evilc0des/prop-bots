use crate::events::*;
use crate::models::*;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Strategy Trait
// ---------------------------------------------------------------------------

/// A trading strategy that processes market data and produces signals.
#[async_trait]
pub trait Strategy: Send + Sync {
    /// Unique identifier for this strategy.
    fn id(&self) -> &str;

    /// Human-readable name.
    fn name(&self) -> &str;

    /// Called once on initialization.
    async fn on_start(&mut self) {}

    /// Called on every new bar.
    async fn on_bar(&mut self, bar: &Bar) -> Vec<Signal>;

    /// Called on every new tick (optional, default no-op).
    async fn on_tick(&mut self, _tick: &Tick) -> Vec<Signal> {
        Vec::new()
    }

    /// Called when an order is filled.
    async fn on_fill(&mut self, _fill: &Fill) {}

    /// Called when a position changes.
    async fn on_position_update(&mut self, _position: &Position) {}

    /// Called once on shutdown.
    async fn on_stop(&mut self) {}

    /// Reset internal state (for backtesting multiple runs).
    fn reset(&mut self);
}

// ---------------------------------------------------------------------------
// Broker Trait
// ---------------------------------------------------------------------------

/// Errors that can occur during broker operations.
#[derive(Debug, thiserror::Error)]
pub enum BrokerError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Order rejected: {0}")]
    OrderRejected(String),
    #[error("Order not found: {0}")]
    OrderNotFound(Uuid),
    #[error("Insufficient margin")]
    InsufficientMargin,
    #[error("Broker error: {0}")]
    Other(String),
}

/// A broker adapter that can submit/cancel orders and stream data.
#[async_trait]
pub trait Broker: Send + Sync {
    /// Connect to the broker.
    async fn connect(&mut self) -> Result<(), BrokerError>;

    /// Disconnect from the broker.
    async fn disconnect(&mut self) -> Result<(), BrokerError>;

    /// Check if connected.
    fn is_connected(&self) -> bool;

    /// Submit an order.
    async fn submit_order(&mut self, order: Order) -> Result<Order, BrokerError>;

    /// Cancel an order by its ID.
    async fn cancel_order(&mut self, order_id: Uuid) -> Result<(), BrokerError>;

    /// Modify an existing order.
    async fn modify_order(&mut self, order: Order) -> Result<Order, BrokerError>;

    /// Get the current account state.
    async fn account_state(&self) -> Result<AccountState, BrokerError>;

    /// Get all open positions.
    async fn positions(&self) -> Result<Vec<Position>, BrokerError>;

    /// Get all active (working) orders.
    async fn active_orders(&self) -> Result<Vec<Order>, BrokerError>;

    /// Flatten (close) all open positions.
    async fn flatten_all(&mut self) -> Result<(), BrokerError>;

    /// Subscribe to a market data event stream.
    /// Returns a receiver that yields `Event` variants.
    async fn subscribe_market_data(
        &mut self,
        instrument: &str,
        timeframe: Timeframe,
    ) -> Result<tokio::sync::mpsc::Receiver<Event>, BrokerError>;
}

// ---------------------------------------------------------------------------
// Data Provider Trait
// ---------------------------------------------------------------------------

/// Errors that can occur during data operations.
#[derive(Debug, thiserror::Error)]
pub enum DataError {
    #[error("Data not found: {0}")]
    NotFound(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("API error: {0}")]
    ApiError(String),
}

/// Provides historical market data for backtesting.
#[async_trait]
pub trait DataProvider: Send + Sync {
    /// Load historical bars for an instrument within a date range.
    async fn load_bars(
        &self,
        instrument: &str,
        timeframe: Timeframe,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Bar>, DataError>;

    /// Load historical ticks for an instrument within a date range.
    async fn load_ticks(
        &self,
        instrument: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Tick>, DataError>;

    /// List available instruments.
    async fn available_instruments(&self) -> Result<Vec<String>, DataError>;
}

// ---------------------------------------------------------------------------
// Risk Manager Trait
// ---------------------------------------------------------------------------

/// Errors / decisions from risk management.
#[derive(Debug, Clone)]
pub enum RiskDecision {
    /// Order is approved.
    Approved,
    /// Order is rejected with a reason.
    Rejected(String),
    /// Order is modified (e.g. quantity reduced).
    Modified(Order),
}

/// Evaluates orders against prop firm rules before submission.
pub trait RiskManager: Send + Sync {
    /// Check whether an order should be allowed.
    fn evaluate_order(&self, order: &Order, account: &AccountState) -> RiskDecision;

    /// Update the risk manager with the latest account state.
    fn update_account(&mut self, account: &AccountState);

    /// Called at the start of each trading day to reset daily counters.
    fn reset_daily(&mut self);

    /// Check if trading should be halted (e.g. daily loss limit reached).
    fn should_halt(&self) -> bool;

    /// Get current risk violations / warnings.
    fn active_violations(&self) -> Vec<RiskViolation>;
}
