use async_trait::async_trait;
use propbot_core::*;
use propbot_indicators::atr::Atr;
use propbot_indicators::donchian::DonchianChannel;
use propbot_indicators::Indicator;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Donchian Channel Breakout strategy.
///
/// Enters long when price breaks above the upper Donchian band.
/// Enters short when price breaks below the lower Donchian band.
/// Uses ATR-based trailing stop for exits.
pub struct DonchianBreakoutStrategy {
    id: String,
    config: DonchianBreakoutConfig,
    channel: DonchianChannel,
    atr: Atr,
    position: Option<Side>,
    stop_price: Option<Decimal>,
    instrument: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DonchianBreakoutConfig {
    pub instrument: String,
    pub channel_period: usize,
    pub atr_period: usize,
    pub atr_stop_multiplier: Decimal,
    pub quantity: Decimal,
}

impl Default for DonchianBreakoutConfig {
    fn default() -> Self {
        Self {
            instrument: "ES".to_string(),
            channel_period: 20,
            atr_period: 14,
            atr_stop_multiplier: Decimal::TWO,
            quantity: Decimal::ONE,
        }
    }
}

impl std::fmt::Debug for DonchianBreakoutStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DonchianBreakoutStrategy")
            .field("id", &self.id)
            .field("config", &self.config)
            .field("position", &self.position)
            .field("stop_price", &self.stop_price)
            .field("instrument", &self.instrument)
            .finish()
    }
}

impl DonchianBreakoutStrategy {
    pub fn new(config: DonchianBreakoutConfig) -> Self {
        let channel = DonchianChannel::new(config.channel_period);
        let atr = Atr::new(config.atr_period);
        let instrument = config.instrument.clone();
        Self {
            id: format!("donchian_breakout_{}", config.channel_period),
            config,
            channel,
            atr,
            position: None,
            stop_price: None,
            instrument,
        }
    }
}

#[async_trait]
impl Strategy for DonchianBreakoutStrategy {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        "Donchian Breakout"
    }

    async fn on_bar(&mut self, bar: &Bar) -> Vec<Signal> {
        let donchian = self.channel.next_hl(bar.high, bar.low);
        let atr = self.atr.next_hlc(bar.high, bar.low, bar.close);
        let mut signals = Vec::new();

        // Check stop loss first
        if let (Some(side), Some(stop)) = (self.position, self.stop_price) {
            let stopped = match side {
                Side::Buy => bar.low <= stop,
                Side::Sell => bar.high >= stop,
            };
            if stopped {
                let action = match side {
                    Side::Buy => SignalAction::ExitLong,
                    Side::Sell => SignalAction::ExitShort,
                };
                signals.push(Signal {
                    id: Uuid::new_v4(),
                    instrument: self.instrument.clone(),
                    action,
                    quantity: Some(self.config.quantity),
                    price: None,
                    strategy_id: self.id.clone(),
                    timestamp: bar.timestamp,
                    metadata: None,
                });
                self.position = None;
                self.stop_price = None;
                return signals;
            }
        }

        // Update trailing stop
        if let (Some(side), Some(atr_val)) = (self.position, atr) {
            let trail = atr_val * self.config.atr_stop_multiplier;
            let new_stop = match side {
                Side::Buy => bar.close - trail,
                Side::Sell => bar.close + trail,
            };
            match (side, self.stop_price) {
                (Side::Buy, Some(current_stop)) => {
                    if new_stop > current_stop {
                        self.stop_price = Some(new_stop);
                    }
                }
                (Side::Sell, Some(current_stop)) => {
                    if new_stop < current_stop {
                        self.stop_price = Some(new_stop);
                    }
                }
                _ => {
                    self.stop_price = Some(new_stop);
                }
            }
        }

        // Entry signals
        if let (Some(donchian_out), Some(atr_val)) = (donchian, atr) {
            if self.position.is_none() {
                // Breakout above upper band
                if bar.close > donchian_out.upper {
                    let trail = atr_val * self.config.atr_stop_multiplier;
                    signals.push(Signal {
                        id: Uuid::new_v4(),
                        instrument: self.instrument.clone(),
                        action: SignalAction::BuyEntry,
                        quantity: Some(self.config.quantity),
                        price: None,
                        strategy_id: self.id.clone(),
                        timestamp: bar.timestamp,
                        metadata: None,
                    });
                    self.position = Some(Side::Buy);
                    self.stop_price = Some(bar.close - trail);
                }
                // Breakout below lower band
                else if bar.close < donchian_out.lower {
                    let trail = atr_val * self.config.atr_stop_multiplier;
                    signals.push(Signal {
                        id: Uuid::new_v4(),
                        instrument: self.instrument.clone(),
                        action: SignalAction::SellEntry,
                        quantity: Some(self.config.quantity),
                        price: None,
                        strategy_id: self.id.clone(),
                        timestamp: bar.timestamp,
                        metadata: None,
                    });
                    self.position = Some(Side::Sell);
                    self.stop_price = Some(bar.close + trail);
                }
            }
        }

        signals
    }

    async fn on_fill(&mut self, _fill: &Fill) {}

    fn reset(&mut self) {
        self.channel.reset();
        self.atr.reset();
        self.position = None;
        self.stop_price = None;
    }
}
