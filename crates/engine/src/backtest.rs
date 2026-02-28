use propbot_core::*;
use propbot_brokers_common::simulated::{SimulatedBroker, SimulatedBrokerConfig};
use propbot_risk::PropFirmRiskManager;
use rust_decimal::Decimal;
use tracing::{info, warn};

use crate::metrics;

/// Configuration for a backtest run.
#[derive(Debug, Clone)]
pub struct BacktestConfig {
    pub instrument: Instrument,
    pub broker_config: SimulatedBrokerConfig,
}

/// Run a backtest: feed bars through the strategy and simulated broker.
pub async fn run_backtest(
    bars: Vec<Bar>,
    strategy: &mut dyn Strategy,
    mut risk_manager: Option<&mut PropFirmRiskManager>,
    config: BacktestConfig,
) -> BacktestResult {
    let mut broker = SimulatedBroker::new(config.broker_config.clone());
    broker.connect().await.expect("Simulated broker connect");

    strategy.on_start().await;

    let mut equity_curve = Vec::with_capacity(bars.len());
    let start_date = bars.first().map(|b| b.timestamp).unwrap_or_default();
    let end_date = bars.last().map(|b| b.timestamp).unwrap_or_default();

    info!(
        instrument = %config.instrument.symbol,
        bars = bars.len(),
        "Starting backtest from {} to {}",
        start_date, end_date
    );

    for bar in &bars {
        // Feed bar to the broker (updates positions, processes pending orders)
        broker.set_current_bar(bar.clone());

        // Update risk manager with current account state
        if let Some(ref mut rm) = risk_manager.as_deref_mut() {
            rm.update_account(broker.account());
        }

        // Feed bar to the strategy
        let signals = strategy.on_bar(bar).await;

        // Process signals
        for signal in signals {
            let order = signal_to_order(&signal);

            // Risk check
            let approved = if let Some(ref rm) = risk_manager.as_deref() {
                match rm.evaluate_order(&order, broker.account()) {
                    RiskDecision::Approved => true,
                    RiskDecision::Rejected(reason) => {
                        warn!(order_id = %order.id, %reason, "Order rejected by risk manager");
                        false
                    }
                    RiskDecision::Modified(modified) => {
                        // Submit modified order instead
                        let _ = broker.submit_order(modified).await;
                        false // original not submitted
                    }
                }
            } else {
                true
            };

            if approved {
                match broker.submit_order(order).await {
                    Ok(filled) => {
                        if filled.status == OrderStatus::Filled {
                            // Notify strategy of fill
                            let fill = Fill {
                                order_id: filled.id,
                                instrument: filled.instrument.clone(),
                                side: filled.side,
                                quantity: filled.filled_quantity,
                                price: filled.price.unwrap_or(bar.close),
                                commission: Decimal::ZERO,
                                timestamp: bar.timestamp,
                            };
                            strategy.on_fill(&fill).await;
                        }
                    }
                    Err(e) => {
                        warn!("Order submission failed: {}", e);
                    }
                }
            }

            // Check if risk manager wants to halt
            if let Some(ref rm) = risk_manager.as_deref() {
                if rm.should_halt() {
                    info!("Risk manager halted trading — flattening all positions");
                    let _ = broker.flatten_all().await;
                }
            }
        }

        // Record equity point
        let account = broker.account();
        equity_curve.push(EquityPoint {
            timestamp: bar.timestamp,
            equity: account.equity,
            drawdown: account.current_drawdown(),
        });
    }

    strategy.on_stop().await;

    // Flatten any remaining positions at the last price
    let _ = broker.flatten_all().await;

    // Compute results
    let trades = broker.trade_log().to_vec();
    let account = broker.account().clone();

    metrics::compute_backtest_result(
        strategy.id().to_string(),
        config.instrument.symbol.clone(),
        config.broker_config.initial_balance,
        account,
        trades,
        equity_curve,
        start_date,
        end_date,
    )
}

/// Convert a signal into an order.
fn signal_to_order(signal: &Signal) -> Order {
    let qty = signal.quantity.unwrap_or(Decimal::ONE);

    match signal.action {
        SignalAction::BuyEntry => {
            let mut order = match signal.price {
                Some(price) => Order::limit(&signal.instrument, Side::Buy, qty, price),
                None => Order::market(&signal.instrument, Side::Buy, qty),
            };
            order.strategy_id = Some(signal.strategy_id.clone());
            order
        }
        SignalAction::SellEntry => {
            let mut order = match signal.price {
                Some(price) => Order::limit(&signal.instrument, Side::Sell, qty, price),
                None => Order::market(&signal.instrument, Side::Sell, qty),
            };
            order.strategy_id = Some(signal.strategy_id.clone());
            order
        }
        SignalAction::ExitLong => {
            let mut order = Order::market(&signal.instrument, Side::Sell, qty);
            order.strategy_id = Some(signal.strategy_id.clone());
            order
        }
        SignalAction::ExitShort => {
            let mut order = Order::market(&signal.instrument, Side::Buy, qty);
            order.strategy_id = Some(signal.strategy_id.clone());
            order
        }
        SignalAction::ExitAll => {
            // Will be handled separately (flatten_all)
            let mut order = Order::market(&signal.instrument, Side::Sell, qty);
            order.strategy_id = Some(signal.strategy_id.clone());
            order
        }
    }
}
