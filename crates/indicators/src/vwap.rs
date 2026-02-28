use crate::Indicator;
use rust_decimal::Decimal;

/// Volume Weighted Average Price (VWAP).
///
/// Resets each session. Call `reset()` at session boundaries.
#[derive(Debug, Clone)]
pub struct Vwap {
    cumulative_tp_vol: Decimal,
    cumulative_vol: Decimal,
    current: Option<Decimal>,
    count: usize,
}

impl Vwap {
    pub fn new() -> Self {
        Self {
            cumulative_tp_vol: Decimal::ZERO,
            cumulative_vol: Decimal::ZERO,
            current: None,
            count: 0,
        }
    }

    /// Feed high, low, close, volume and compute VWAP.
    pub fn next_hlcv(
        &mut self,
        high: Decimal,
        low: Decimal,
        close: Decimal,
        volume: Decimal,
    ) -> Decimal {
        let typical_price = (high + low + close) / Decimal::from(3);
        self.cumulative_tp_vol += typical_price * volume;
        self.cumulative_vol += volume;
        self.count += 1;

        let vwap = if self.cumulative_vol.is_zero() {
            typical_price
        } else {
            self.cumulative_tp_vol / self.cumulative_vol
        };

        self.current = Some(vwap);
        vwap
    }

    pub fn value(&self) -> Option<Decimal> {
        self.current
    }
}

impl Default for Vwap {
    fn default() -> Self {
        Self::new()
    }
}

impl Indicator for Vwap {
    fn next(&mut self, value: Decimal) -> Option<Decimal> {
        // Simplified: assume volume = 1
        Some(self.next_hlcv(value, value, value, Decimal::ONE))
    }

    fn reset(&mut self) {
        self.cumulative_tp_vol = Decimal::ZERO;
        self.cumulative_vol = Decimal::ZERO;
        self.current = None;
        self.count = 0;
    }

    fn period(&self) -> usize {
        1
    }

    fn is_ready(&self) -> bool {
        self.current.is_some()
    }
}
