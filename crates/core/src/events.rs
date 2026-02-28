use crate::models::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Top-level event enum that flows through the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    MarketData(MarketDataEvent),
    Signal(Signal),
    Order(OrderEvent),
    Risk(RiskEvent),
    System(SystemEvent),
}

/// Market data events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketDataEvent {
    Bar(Bar),
    Tick(Tick),
}

/// Order lifecycle events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderEvent {
    Submitted(Order),
    Filled(Fill),
    PartiallyFilled(Fill),
    Cancelled { order_id: Uuid, reason: String },
    Rejected { order_id: Uuid, reason: String },
}

/// Risk management events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskEvent {
    /// An order was blocked by the risk manager.
    OrderBlocked {
        order_id: Uuid,
        reason: String,
    },
    /// A risk rule threshold was breached.
    Violation(RiskViolation),
    /// Positions auto-flattened due to approaching a limit.
    AutoFlatten {
        reason: String,
    },
}

/// Details about a risk rule violation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskViolation {
    pub rule: String,
    pub message: String,
    pub current_value: String,
    pub threshold: String,
    pub severity: RiskSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskSeverity {
    Warning,
    Critical,
    Breach,
}

/// System lifecycle events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemEvent {
    Started { message: String },
    Stopped { message: String },
    Error { message: String },
    Info { message: String },
}
