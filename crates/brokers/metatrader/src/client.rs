use async_trait::async_trait;
use chrono::Utc;
use propbot_core::*;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tracing::info;
use uuid::Uuid;

use crate::protocol::*;

/// Configuration for connecting to MetaTrader 5.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaTraderConfig {
    /// Host address (e.g. "127.0.0.1").
    pub host: String,
    /// Port the MT5 EA/script is listening on.
    pub port: u16,
    /// Reconnect interval in seconds.
    pub reconnect_interval_secs: u64,
    /// Heartbeat interval in seconds.
    pub heartbeat_interval_secs: u64,
}

impl Default for MetaTraderConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 5556, // Different default port from NT8 to avoid clashes
            reconnect_interval_secs: 5,
            heartbeat_interval_secs: 10,
        }
    }
}

/// MetaTrader 5 broker adapter.
///
/// Communicates with an MQL5 EA running inside MT5 via TCP socket
/// using length-prefixed JSON messages.
pub struct MetaTraderBroker {
    config: MetaTraderConfig,
    stream: Option<TcpStream>,
    connected: bool,
    account: AccountState,
    positions: HashMap<String, Position>,
    active_orders: HashMap<Uuid, Order>,
    /// Maps our order IDs to broker-assigned IDs.
    order_id_map: HashMap<Uuid, String>,
}

impl MetaTraderBroker {
    pub fn new(config: MetaTraderConfig) -> Self {
        Self {
            config,
            stream: None,
            connected: false,
            account: AccountState::new(Decimal::ZERO),
            positions: HashMap::new(),
            active_orders: HashMap::new(),
            order_id_map: HashMap::new(),
        }
    }

    /// Send a message to MetaTrader.
    async fn send(&mut self, msg: &OutboundMessage) -> Result<(), BrokerError> {
        let stream = self
            .stream
            .as_mut()
            .ok_or_else(|| BrokerError::ConnectionFailed("Not connected".to_string()))?;

        let json = serde_json::to_vec(msg)
            .map_err(|e| BrokerError::Other(format!("Serialization error: {}", e)))?;
        let framed = frame_message(&json);

        stream
            .write_all(&framed)
            .await
            .map_err(|e| BrokerError::ConnectionFailed(format!("Write error: {}", e)))?;

        Ok(())
    }

    /// Read a single framed message from MetaTrader.
    async fn recv(&mut self) -> Result<InboundMessage, BrokerError> {
        let stream = self
            .stream
            .as_mut()
            .ok_or_else(|| BrokerError::ConnectionFailed("Not connected".to_string()))?;

        // Read 4-byte length prefix
        let mut len_buf = [0u8; 4];
        stream
            .read_exact(&mut len_buf)
            .await
            .map_err(|e| BrokerError::ConnectionFailed(format!("Read error: {}", e)))?;
        let len = u32::from_be_bytes(len_buf) as usize;

        // Read message body
        let mut body = vec![0u8; len];
        stream
            .read_exact(&mut body)
            .await
            .map_err(|e| BrokerError::ConnectionFailed(format!("Read error: {}", e)))?;

        let msg: InboundMessage = serde_json::from_slice(&body)
            .map_err(|e| BrokerError::Other(format!("Deserialization error: {}", e)))?;

        Ok(msg)
    }

    /// Process an inbound message, updating internal state.
    #[allow(dead_code)]
    fn process_message(&mut self, msg: &InboundMessage) {
        match msg {
            InboundMessage::AccountUpdate {
                balance,
                equity,
                unrealized_pnl,
                realized_pnl,
                margin_used,
            } => {
                self.account.balance = *balance;
                self.account.equity = *equity;
                self.account.unrealized_pnl = *unrealized_pnl;
                self.account.realized_pnl = *realized_pnl;
                self.account.margin_used = *margin_used;
                self.account.margin_available = *equity - *margin_used;
                if *equity > self.account.high_water_mark {
                    self.account.high_water_mark = *equity;
                }
            }
            InboundMessage::PositionUpdate {
                instrument,
                side,
                quantity,
                avg_entry_price,
                unrealized_pnl,
            } => {
                if quantity.is_zero() {
                    self.positions.remove(instrument);
                } else {
                    let side = match side.as_str() {
                        "buy" | "long" => Side::Buy,
                        _ => Side::Sell,
                    };
                    self.positions.insert(
                        instrument.clone(),
                        Position {
                            instrument: instrument.clone(),
                            side,
                            quantity: *quantity,
                            avg_entry_price: *avg_entry_price,
                            unrealized_pnl: *unrealized_pnl,
                            realized_pnl: Decimal::ZERO,
                            opened_at: Utc::now(),
                            strategy_id: None,
                        },
                    );
                }
                self.account.open_positions = self.positions.len();
            }
            InboundMessage::OrderUpdate {
                client_order_id,
                broker_order_id,
                status,
                filled_quantity,
                fill_price,
                message: _,
            } => {
                if let Ok(uuid) = Uuid::parse_str(client_order_id) {
                    if let Some(order) = self.active_orders.get_mut(&uuid) {
                        order.broker_order_id = Some(broker_order_id.clone());
                        order.filled_quantity = *filled_quantity;
                        order.updated_at = Utc::now();
                        order.status = match status.as_str() {
                            "filled" => OrderStatus::Filled,
                            "partially_filled" => OrderStatus::PartiallyFilled,
                            "cancelled" => OrderStatus::Cancelled,
                            "rejected" => OrderStatus::Rejected,
                            "submitted" => OrderStatus::Submitted,
                            _ => OrderStatus::Pending,
                        };
                        if let Some(price) = fill_price {
                            order.price = Some(*price);
                        }
                        self.order_id_map
                            .insert(uuid, broker_order_id.clone());
                    }
                }
            }
            _ => {}
        }
    }
}

#[async_trait]
impl Broker for MetaTraderBroker {
    async fn connect(&mut self) -> Result<(), BrokerError> {
        let addr = format!("{}:{}", self.config.host, self.config.port);
        info!("Connecting to MetaTrader at {}", addr);

        let stream = TcpStream::connect(&addr)
            .await
            .map_err(|e| BrokerError::ConnectionFailed(format!("TCP connect failed: {}", e)))?;

        self.stream = Some(stream);

        // Wait for Connected message
        let msg = self.recv().await?;
        match msg {
            InboundMessage::Connected { version } => {
                info!("Connected to MetaTrader EA v{}", version);
                self.connected = true;
            }
            InboundMessage::Error { message } => {
                return Err(BrokerError::ConnectionFailed(message));
            }
            _ => {
                return Err(BrokerError::ConnectionFailed(
                    "Unexpected initial message".to_string(),
                ));
            }
        }

        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), BrokerError> {
        if let Some(mut stream) = self.stream.take() {
            let _ = stream.shutdown().await;
        }
        self.connected = false;
        info!("Disconnected from MetaTrader");
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    async fn submit_order(&mut self, mut order: Order) -> Result<Order, BrokerError> {
        let msg = OutboundMessage::OrderSubmit {
            id: order.id.to_string(),
            instrument: order.instrument.clone(),
            side: match order.side {
                Side::Buy => "buy".to_string(),
                Side::Sell => "sell".to_string(),
            },
            order_type: match order.order_type {
                OrderType::Market => "market".to_string(),
                OrderType::Limit => "limit".to_string(),
                OrderType::Stop => "stop".to_string(),
                OrderType::StopLimit => "stop_limit".to_string(),
            },
            quantity: order.quantity,
            price: order.price,
            stop_price: order.stop_price,
        };

        self.send(&msg).await?;
        order.status = OrderStatus::Submitted;
        order.updated_at = Utc::now();
        self.active_orders.insert(order.id, order.clone());

        Ok(order)
    }

    async fn cancel_order(&mut self, order_id: Uuid) -> Result<(), BrokerError> {
        let broker_id = self
            .order_id_map
            .get(&order_id)
            .cloned()
            .ok_or(BrokerError::OrderNotFound(order_id))?;

        let msg = OutboundMessage::OrderCancel {
            broker_order_id: broker_id,
        };
        self.send(&msg).await?;
        Ok(())
    }

    async fn modify_order(&mut self, order: Order) -> Result<Order, BrokerError> {
        let broker_id = self
            .order_id_map
            .get(&order.id)
            .cloned()
            .ok_or(BrokerError::OrderNotFound(order.id))?;

        let msg = OutboundMessage::OrderModify {
            broker_order_id: broker_id,
            quantity: Some(order.quantity),
            price: order.price,
            stop_price: order.stop_price,
        };
        self.send(&msg).await?;
        Ok(order)
    }

    async fn account_state(&self) -> Result<AccountState, BrokerError> {
        Ok(self.account.clone())
    }

    async fn positions(&self) -> Result<Vec<Position>, BrokerError> {
        Ok(self.positions.values().cloned().collect())
    }

    async fn active_orders(&self) -> Result<Vec<Order>, BrokerError> {
        Ok(self
            .active_orders
            .values()
            .filter(|o| o.is_active())
            .cloned()
            .collect())
    }

    async fn flatten_all(&mut self) -> Result<(), BrokerError> {
        self.send(&OutboundMessage::FlattenAll).await
    }

    async fn subscribe_market_data(
        &mut self,
        instrument: &str,
        timeframe: Timeframe,
    ) -> Result<mpsc::Receiver<Event>, BrokerError> {
        let tf_str = match timeframe {
            Timeframe::Tick => "tick".to_string(),
            Timeframe::Minute(n) => format!("{}min", n),
            Timeframe::Hour(n) => format!("{}h", n),
            Timeframe::Daily => "daily".to_string(),
            _ => "1min".to_string(),
        };

        self.send(&OutboundMessage::Subscribe {
            instrument: instrument.to_string(),
            timeframe: tf_str,
        })
        .await?;

        // Create a channel for streaming events
        let (_tx, rx) = mpsc::channel(1024);

        // Standard event loop for market data should be cleanly connected here, returning the rx channel
        Ok(rx)
    }
}
