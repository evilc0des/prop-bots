use async_trait::async_trait;
use chrono::Utc;
use propbot_core::*;
use rust_decimal::Decimal;
use std::collections::HashMap;
use tokio::sync::mpsc;
use uuid::Uuid;

/// Configuration for the simulated broker (backtesting).
#[derive(Debug, Clone)]
pub struct SimulatedBrokerConfig {
    /// Starting account balance.
    pub initial_balance: Decimal,
    /// Commission per contract/lot.
    pub commission_per_contract: Decimal,
    /// Slippage in ticks per order.
    pub slippage_ticks: Decimal,
    /// Tick size (for slippage calculation).
    pub tick_size: Decimal,
    /// Tick value (for PnL calculation).
    pub tick_value: Decimal,
}

impl Default for SimulatedBrokerConfig {
    fn default() -> Self {
        Self {
            initial_balance: Decimal::new(50_000, 0),
            commission_per_contract: Decimal::new(4, 0), // $4 per contract round-trip
            slippage_ticks: Decimal::ONE,
            tick_size: Decimal::new(25, 2),  // 0.25 (e.g., ES futures)
            tick_value: Decimal::new(1250, 2), // $12.50 per tick
        }
    }
}

/// A simulated broker for backtesting.
///
/// Processes orders against historical data, simulating fills with
/// configurable slippage and commissions.
pub struct SimulatedBroker {
    config: SimulatedBrokerConfig,
    account: AccountState,
    positions: HashMap<String, Position>,
    active_orders: Vec<Order>,
    filled_orders: Vec<Order>,
    trades: Vec<Trade>,
    connected: bool,
    /// Current bar being processed (set by the engine).
    current_bar: Option<Bar>,
}

impl SimulatedBroker {
    pub fn new(config: SimulatedBrokerConfig) -> Self {
        let account = AccountState::new(config.initial_balance);
        Self {
            config,
            account,
            positions: HashMap::new(),
            active_orders: Vec::new(),
            filled_orders: Vec::new(),
            trades: Vec::new(),
            connected: false,
            current_bar: None,
        }
    }

    /// Set the current bar (called by the engine on each step).
    pub fn set_current_bar(&mut self, bar: Bar) {
        self.current_bar = Some(bar.clone());
        // Update unrealized PnL for all positions
        for pos in self.positions.values_mut() {
            pos.update_pnl(bar.close, self.config.tick_size, self.config.tick_value);
        }
        self.update_account_equity();
        // Process working orders against this bar
        self.process_pending_orders(&bar);
    }

    /// Get the trade log.
    pub fn trade_log(&self) -> &[Trade] {
        &self.trades
    }

    /// Get the current account state (non-async).
    pub fn account(&self) -> &AccountState {
        &self.account
    }

    /// Simulate filling a market order at the current bar.
    fn simulate_fill(&mut self, order: &mut Order) -> Option<Fill> {
        let bar = self.current_bar.as_ref()?;

        // Determine fill price with slippage
        let slippage = self.config.slippage_ticks * self.config.tick_size;
        let fill_price = match order.side {
            Side::Buy => bar.close + slippage,
            Side::Sell => bar.close - slippage,
        };

        let commission = self.config.commission_per_contract * order.quantity;

        let fill = Fill {
            order_id: order.id,
            instrument: order.instrument.clone(),
            side: order.side,
            quantity: order.quantity,
            price: fill_price,
            commission,
            timestamp: bar.timestamp,
        };

        order.filled_quantity = order.quantity;
        order.status = OrderStatus::Filled;
        order.updated_at = bar.timestamp;

        // Update positions
        self.apply_fill(&fill);

        Some(fill)
    }

    /// Apply a fill to positions and account.
    fn apply_fill(&mut self, fill: &Fill) {
        let existing = self.positions.get(&fill.instrument);

        match existing {
            Some(pos) if pos.side != fill.side => {
                // Closing or reversing a position
                let close_qty = fill.quantity.min(pos.quantity);
                let remaining_qty = fill.quantity - close_qty;

                // Close the position (or part of it)
                let pnl = self.compute_pnl(pos, fill.price, close_qty);
                let trade = Trade {
                    id: Uuid::new_v4(),
                    instrument: fill.instrument.clone(),
                    side: pos.side,
                    quantity: close_qty,
                    entry_price: pos.avg_entry_price,
                    exit_price: fill.price,
                    pnl,
                    commission: fill.commission,
                    entry_time: pos.opened_at,
                    exit_time: fill.timestamp,
                    strategy_id: pos.strategy_id.clone(),
                };
                self.trades.push(trade);

                // Update account
                self.account.balance += pnl - fill.commission;
                self.account.realized_pnl += pnl;
                self.account.daily_pnl += pnl - fill.commission;

                let pos = self.positions.get_mut(&fill.instrument).unwrap();
                if close_qty >= pos.quantity {
                    self.positions.remove(&fill.instrument);
                } else {
                    pos.quantity -= close_qty;
                }

                // If there's remaining quantity, open a new position in the opposite direction
                if remaining_qty > Decimal::ZERO {
                    self.positions.insert(
                        fill.instrument.clone(),
                        Position {
                            instrument: fill.instrument.clone(),
                            side: fill.side,
                            quantity: remaining_qty,
                            avg_entry_price: fill.price,
                            unrealized_pnl: Decimal::ZERO,
                            realized_pnl: Decimal::ZERO,
                            opened_at: fill.timestamp,
                            strategy_id: None,
                        },
                    );
                }
            }
            Some(pos) if pos.side == fill.side => {
                // Adding to existing position
                let pos = self.positions.get_mut(&fill.instrument).unwrap();
                let total_cost = pos.avg_entry_price * pos.quantity + fill.price * fill.quantity;
                pos.quantity += fill.quantity;
                pos.avg_entry_price = total_cost / pos.quantity;
                self.account.balance -= fill.commission;
                self.account.daily_pnl -= fill.commission;
            }
            _ => {
                // New position
                self.positions.insert(
                    fill.instrument.clone(),
                    Position {
                        instrument: fill.instrument.clone(),
                        side: fill.side,
                        quantity: fill.quantity,
                        avg_entry_price: fill.price,
                        unrealized_pnl: Decimal::ZERO,
                        realized_pnl: Decimal::ZERO,
                        opened_at: fill.timestamp,
                        strategy_id: None,
                    },
                );
                self.account.balance -= fill.commission;
                self.account.daily_pnl -= fill.commission;
            }
        }

        self.update_account_equity();
    }

    fn compute_pnl(&self, pos: &Position, exit_price: Decimal, quantity: Decimal) -> Decimal {
        let price_diff = match pos.side {
            Side::Buy => exit_price - pos.avg_entry_price,
            Side::Sell => pos.avg_entry_price - exit_price,
        };
        let ticks = price_diff / self.config.tick_size;
        ticks * self.config.tick_value * quantity
    }

    fn update_account_equity(&mut self) {
        let unrealized: Decimal = self.positions.values().map(|p| p.unrealized_pnl).sum();
        self.account.unrealized_pnl = unrealized;
        self.account.equity = self.account.balance + unrealized;
        self.account.open_positions = self.positions.len();
        self.account.timestamp = Utc::now();

        if self.account.equity > self.account.high_water_mark {
            self.account.high_water_mark = self.account.equity;
        }
    }

    /// Process pending limit/stop orders against a bar.
    fn process_pending_orders(&mut self, bar: &Bar) {
        let mut to_fill = Vec::new();

        for (i, order) in self.active_orders.iter().enumerate() {
            match order.order_type {
                OrderType::Limit => {
                    if let Some(price) = order.price {
                        let triggered = match order.side {
                            Side::Buy => bar.low <= price,
                            Side::Sell => bar.high >= price,
                        };
                        if triggered {
                            to_fill.push(i);
                        }
                    }
                }
                OrderType::Stop => {
                    if let Some(stop_price) = order.stop_price {
                        let triggered = match order.side {
                            Side::Buy => bar.high >= stop_price,
                            Side::Sell => bar.low <= stop_price,
                        };
                        if triggered {
                            to_fill.push(i);
                        }
                    }
                }
                _ => {}
            }
        }

        // Fill triggered orders (reverse iterate to preserve indices)
        for i in to_fill.into_iter().rev() {
            let mut order = self.active_orders.remove(i);
            self.simulate_fill(&mut order);
            self.filled_orders.push(order);
        }
    }

    /// Reset broker state (for re-running backtests).
    pub fn reset(&mut self) {
        self.account = AccountState::new(self.config.initial_balance);
        self.positions.clear();
        self.active_orders.clear();
        self.filled_orders.clear();
        self.trades.clear();
        self.current_bar = None;
    }
}

#[async_trait]
impl Broker for SimulatedBroker {
    async fn connect(&mut self) -> Result<(), BrokerError> {
        self.connected = true;
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), BrokerError> {
        self.connected = false;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    async fn submit_order(&mut self, mut order: Order) -> Result<Order, BrokerError> {
        order.status = OrderStatus::Submitted;
        order.updated_at = Utc::now();

        match order.order_type {
            OrderType::Market => {
                // Immediate fill
                self.simulate_fill(&mut order);
                self.filled_orders.push(order.clone());
            }
            OrderType::Limit | OrderType::Stop | OrderType::StopLimit => {
                // Add to working orders
                self.active_orders.push(order.clone());
            }
        }

        Ok(order)
    }

    async fn cancel_order(&mut self, order_id: Uuid) -> Result<(), BrokerError> {
        if let Some(pos) = self.active_orders.iter().position(|o| o.id == order_id) {
            let mut order = self.active_orders.remove(pos);
            order.status = OrderStatus::Cancelled;
            self.filled_orders.push(order);
            Ok(())
        } else {
            Err(BrokerError::OrderNotFound(order_id))
        }
    }

    async fn modify_order(&mut self, order: Order) -> Result<Order, BrokerError> {
        if let Some(existing) = self.active_orders.iter_mut().find(|o| o.id == order.id) {
            existing.price = order.price;
            existing.stop_price = order.stop_price;
            existing.quantity = order.quantity;
            existing.updated_at = Utc::now();
            Ok(existing.clone())
        } else {
            Err(BrokerError::OrderNotFound(order.id))
        }
    }

    async fn account_state(&self) -> Result<AccountState, BrokerError> {
        Ok(self.account.clone())
    }

    async fn positions(&self) -> Result<Vec<Position>, BrokerError> {
        Ok(self.positions.values().cloned().collect())
    }

    async fn active_orders(&self) -> Result<Vec<Order>, BrokerError> {
        Ok(self.active_orders.clone())
    }

    async fn flatten_all(&mut self) -> Result<(), BrokerError> {
        let instruments: Vec<String> = self.positions.keys().cloned().collect();
        for instrument in instruments {
            if let Some(pos) = self.positions.get(&instrument) {
                let order = Order::market(&instrument, pos.side.opposite(), pos.quantity);
                self.submit_order(order).await?;
            }
        }
        Ok(())
    }

    async fn subscribe_market_data(
        &mut self,
        _instrument: &str,
        _timeframe: Timeframe,
    ) -> Result<mpsc::Receiver<Event>, BrokerError> {
        // In backtesting, market data is fed by the engine directly.
        let (_tx, rx) = mpsc::channel(1);
        Ok(rx)
    }
}
