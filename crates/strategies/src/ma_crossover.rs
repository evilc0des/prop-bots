use async_trait::async_trait;
use propbot_core::*;
use propbot_indicators::ema::Ema;
use propbot_indicators::sma::Sma;
use propbot_indicators::Indicator;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Moving Average Crossover strategy.
///
/// Goes long when the fast MA crosses above the slow MA.
/// Goes short when the fast MA crosses below the slow MA.
pub struct MaCrossoverStrategy {
    id: String,
    config: MaCrossoverConfig,
    fast_ma: Box<dyn Indicator>,
    slow_ma: Box<dyn Indicator>,
    prev_fast: Option<Decimal>,
    prev_slow: Option<Decimal>,
    position: Option<Side>,
    instrument: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaCrossoverConfig {
    pub instrument: String,
    pub fast_period: usize,
    pub slow_period: usize,
    pub quantity: Decimal,
    /// "sma" or "ema"
    pub ma_type: String,
}

impl Default for MaCrossoverConfig {
    fn default() -> Self {
        Self {
            instrument: "ES".to_string(),
            fast_period: 10,
            slow_period: 20,
            quantity: Decimal::ONE,
            ma_type: "ema".to_string(),
        }
    }
}

impl std::fmt::Debug for MaCrossoverStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MaCrossoverStrategy")
            .field("id", &self.id)
            .field("config", &self.config)
            .field("prev_fast", &self.prev_fast)
            .field("prev_slow", &self.prev_slow)
            .field("position", &self.position)
            .field("instrument", &self.instrument)
            .finish()
    }
}

impl MaCrossoverStrategy {
    pub fn new(config: MaCrossoverConfig) -> Self {
        let fast_ma: Box<dyn Indicator> = match config.ma_type.as_str() {
            "sma" => Box::new(Sma::new(config.fast_period)),
            _ => Box::new(Ema::new(config.fast_period)),
        };
        let slow_ma: Box<dyn Indicator> = match config.ma_type.as_str() {
            "sma" => Box::new(Sma::new(config.slow_period)),
            _ => Box::new(Ema::new(config.slow_period)),
        };
        let instrument = config.instrument.clone();
        Self {
            id: format!("ma_crossover_{}_{}", config.fast_period, config.slow_period),
            config,
            fast_ma,
            slow_ma,
            prev_fast: None,
            prev_slow: None,
            position: None,
            instrument,
        }
    }
}

#[async_trait]
impl Strategy for MaCrossoverStrategy {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        "MA Crossover"
    }

    async fn on_bar(&mut self, bar: &Bar) -> Vec<Signal> {
        let fast = self.fast_ma.next(bar.close);
        let slow = self.slow_ma.next(bar.close);

        let mut signals = Vec::new();

        if let (Some(fast_val), Some(slow_val)) = (fast, slow) {
            if let (Some(prev_f), Some(prev_s)) = (self.prev_fast, self.prev_slow) {
                // Bullish crossover: fast crosses above slow
                if prev_f <= prev_s && fast_val > slow_val {
                    // Close any short position first
                    if self.position == Some(Side::Sell) {
                        signals.push(Signal {
                            id: Uuid::new_v4(),
                            instrument: self.instrument.clone(),
                            action: SignalAction::ExitShort,
                            quantity: Some(self.config.quantity),
                            price: None,
                            strategy_id: self.id.clone(),
                            timestamp: bar.timestamp,
                            metadata: None,
                        });
                    }
                    // Go long
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
                }
                // Bearish crossover: fast crosses below slow
                else if prev_f >= prev_s && fast_val < slow_val {
                    // Close any long position first
                    if self.position == Some(Side::Buy) {
                        signals.push(Signal {
                            id: Uuid::new_v4(),
                            instrument: self.instrument.clone(),
                            action: SignalAction::ExitLong,
                            quantity: Some(self.config.quantity),
                            price: None,
                            strategy_id: self.id.clone(),
                            timestamp: bar.timestamp,
                            metadata: None,
                        });
                    }
                    // Go short
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
                }
            }

            self.prev_fast = Some(fast_val);
            self.prev_slow = Some(slow_val);
        }

        signals
    }

    async fn on_fill(&mut self, _fill: &Fill) {}

    fn reset(&mut self) {
        self.fast_ma.reset();
        self.slow_ma.reset();
        self.prev_fast = None;
        self.prev_slow = None;
        self.position = None;
    }
}
