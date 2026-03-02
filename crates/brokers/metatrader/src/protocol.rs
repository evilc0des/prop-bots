use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Messages sent from the Rust client TO MetaTrader.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum OutboundMessage {
    /// Submit a new order.
    #[serde(rename = "order_submit")]
    OrderSubmit {
        id: String,
        instrument: String,
        side: String,
        order_type: String,
        quantity: Decimal,
        price: Option<Decimal>,
        stop_price: Option<Decimal>,
    },
    /// Cancel an existing order.
    #[serde(rename = "order_cancel")]
    OrderCancel { broker_order_id: String },
    /// Modify an existing order.
    #[serde(rename = "order_modify")]
    OrderModify {
        broker_order_id: String,
        quantity: Option<Decimal>,
        price: Option<Decimal>,
        stop_price: Option<Decimal>,
    },
    /// Request current account state.
    #[serde(rename = "account_request")]
    AccountRequest,
    /// Request current positions.
    #[serde(rename = "positions_request")]
    PositionsRequest,
    /// Subscribe to market data.
    #[serde(rename = "subscribe")]
    Subscribe {
        instrument: String,
        timeframe: String,
    },
    /// Unsubscribe from market data.
    #[serde(rename = "unsubscribe")]
    Unsubscribe { instrument: String },
    /// Flatten all positions.
    #[serde(rename = "flatten_all")]
    FlattenAll,
    /// Heartbeat.
    #[serde(rename = "heartbeat")]
    Heartbeat { timestamp: DateTime<Utc> },
}

/// Messages received FROM MetaTrader.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum InboundMessage {
    /// Market data bar.
    #[serde(rename = "bar")]
    Bar {
        instrument: String,
        timestamp: DateTime<Utc>,
        open: Decimal,
        high: Decimal,
        low: Decimal,
        close: Decimal,
        volume: Decimal,
    },
    /// Market data tick.
    #[serde(rename = "tick")]
    Tick {
        instrument: String,
        timestamp: DateTime<Utc>,
        bid: Decimal,
        ask: Decimal,
        last: Decimal,
        volume: Decimal,
    },
    /// Order update (fill, cancel, reject, etc.).
    #[serde(rename = "order_update")]
    OrderUpdate {
        client_order_id: String,
        broker_order_id: String,
        status: String,
        filled_quantity: Decimal,
        fill_price: Option<Decimal>,
        message: Option<String>,
    },
    /// Account state update.
    #[serde(rename = "account_update")]
    AccountUpdate {
        balance: Decimal,
        equity: Decimal,
        unrealized_pnl: Decimal,
        realized_pnl: Decimal,
        margin_used: Decimal,
    },
    /// Position update.
    #[serde(rename = "position_update")]
    PositionUpdate {
        instrument: String,
        side: String,
        quantity: Decimal,
        avg_entry_price: Decimal,
        unrealized_pnl: Decimal,
    },
    /// Heartbeat acknowledgement.
    #[serde(rename = "heartbeat_ack")]
    HeartbeatAck { timestamp: DateTime<Utc> },
    /// Error message.
    #[serde(rename = "error")]
    Error { message: String },
    /// Connection established.
    #[serde(rename = "connected")]
    Connected { version: String },
}

/// Frame a message with a 4-byte length prefix (big-endian).
pub fn frame_message(msg: &[u8]) -> Vec<u8> {
    let len = msg.len() as u32;
    let mut framed = Vec::with_capacity(4 + msg.len());
    framed.extend_from_slice(&len.to_be_bytes());
    framed.extend_from_slice(msg);
    framed
}
