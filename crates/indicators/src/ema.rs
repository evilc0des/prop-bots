use crate::Indicator;
use rust_decimal::Decimal;

/// Exponential Moving Average (EMA).
#[derive(Debug, Clone)]
pub struct Ema {
    len: usize,
    multiplier: Decimal,
    current: Option<Decimal>,
    count: usize,
    /// Accumulates values for the initial SMA seed.
    seed_sum: Decimal,
}

impl Ema {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "EMA period must be > 0");
        let multiplier =
            Decimal::TWO / (Decimal::from(period) + Decimal::ONE);
        Self {
            len: period,
            multiplier,
            current: None,
            count: 0,
            seed_sum: Decimal::ZERO,
        }
    }

    pub fn value(&self) -> Option<Decimal> {
        self.current
    }
}

impl Indicator for Ema {
    fn next(&mut self, value: Decimal) -> Option<Decimal> {
        self.count += 1;

        match self.current {
            None => {
                // Accumulate for SMA seed
                self.seed_sum += value;
                if self.count >= self.len {
                    let sma = self.seed_sum / Decimal::from(self.len);
                    self.current = Some(sma);
                }
            }
            Some(prev) => {
                let ema = (value - prev) * self.multiplier + prev;
                self.current = Some(ema);
            }
        }

        self.current
    }

    fn reset(&mut self) {
        self.current = None;
        self.count = 0;
        self.seed_sum = Decimal::ZERO;
    }

    fn period(&self) -> usize {
        self.len
    }

    fn is_ready(&self) -> bool {
        self.current.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_ema_seed() {
        let mut ema = Ema::new(3);
        assert_eq!(ema.next(dec!(2)), None);
        assert_eq!(ema.next(dec!(4)), None);
        // Third value → SMA seed = (2+4+6)/3 = 4
        let result = ema.next(dec!(6));
        assert_eq!(result, Some(dec!(4)));
    }

    #[test]
    fn test_ema_after_seed() {
        let mut ema = Ema::new(3);
        ema.next(dec!(2));
        ema.next(dec!(4));
        ema.next(dec!(6)); // seed = 4
        // EMA = (8 - 4) * 0.5 + 4 = 6
        let result = ema.next(dec!(8));
        assert_eq!(result, Some(dec!(6)));
    }
}
