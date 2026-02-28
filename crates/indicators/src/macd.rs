use crate::ema::Ema;
use crate::Indicator;
use rust_decimal::Decimal;

/// MACD (Moving Average Convergence Divergence).
///
/// Composed of three EMAs:
/// - Fast EMA (default 12)
/// - Slow EMA (default 26)
/// - Signal EMA (default 9)
///
/// Returns the MACD line value. Use `signal()` and `histogram()` for the other components.
#[derive(Debug, Clone)]
pub struct Macd {
    fast_ema: Ema,
    slow_ema: Ema,
    signal_ema: Ema,
    macd_line: Option<Decimal>,
    signal_line: Option<Decimal>,
}

/// MACD output with all three components.
#[derive(Debug, Clone, Copy)]
pub struct MacdOutput {
    pub macd: Decimal,
    pub signal: Decimal,
    pub histogram: Decimal,
}

impl Macd {
    pub fn new(fast_period: usize, slow_period: usize, signal_period: usize) -> Self {
        assert!(fast_period < slow_period, "Fast period must be less than slow period");
        Self {
            fast_ema: Ema::new(fast_period),
            slow_ema: Ema::new(slow_period),
            signal_ema: Ema::new(signal_period),
            macd_line: None,
            signal_line: None,
        }
    }

    /// Standard MACD (12, 26, 9).
    pub fn default_periods() -> Self {
        Self::new(12, 26, 9)
    }

    /// Returns the full MACD output (macd, signal, histogram) if ready.
    pub fn output(&self) -> Option<MacdOutput> {
        match (self.macd_line, self.signal_line) {
            (Some(macd), Some(signal)) => Some(MacdOutput {
                macd,
                signal,
                histogram: macd - signal,
            }),
            _ => None,
        }
    }

    /// Process next value and return full output if ready.
    pub fn next_output(&mut self, value: Decimal) -> Option<MacdOutput> {
        let fast = self.fast_ema.next(value);
        let slow = self.slow_ema.next(value);

        match (fast, slow) {
            (Some(f), Some(s)) => {
                let macd = f - s;
                self.macd_line = Some(macd);
                self.signal_line = self.signal_ema.next(macd);
            }
            _ => {}
        }

        self.output()
    }
}

impl Indicator for Macd {
    fn next(&mut self, value: Decimal) -> Option<Decimal> {
        self.next_output(value).map(|o| o.macd)
    }

    fn reset(&mut self) {
        self.fast_ema.reset();
        self.slow_ema.reset();
        self.signal_ema.reset();
        self.macd_line = None;
        self.signal_line = None;
    }

    fn period(&self) -> usize {
        self.slow_ema.period()
    }

    fn is_ready(&self) -> bool {
        self.signal_line.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_macd_needs_slow_period_data() {
        let mut macd = Macd::new(3, 5, 3);
        // Feed 4 values → slow EMA not ready yet
        for i in 0..4 {
            assert!(macd.next_output(Decimal::from(i + 1)).is_none());
        }
        // 5th value → slow EMA seeds, MACD line ready, but signal needs 3 more
        let out = macd.next_output(dec!(5));
        assert!(out.is_none() || out.unwrap().signal == out.unwrap().macd); // signal may seed
    }
}
