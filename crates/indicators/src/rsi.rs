use crate::Indicator;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::VecDeque;

/// Relative Strength Index (RSI).
/// Uses EMA (Wilder's smoothing) for average gain/loss.
#[derive(Debug, Clone)]
pub struct Rsi {
    len: usize,
    prev_value: Option<Decimal>,
    gains: VecDeque<Decimal>,
    losses: VecDeque<Decimal>,
    avg_gain: Option<Decimal>,
    avg_loss: Option<Decimal>,
    count: usize,
}

impl Rsi {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "RSI period must be > 0");
        Self {
            len: period,
            prev_value: None,
            gains: VecDeque::with_capacity(period),
            losses: VecDeque::with_capacity(period),
            avg_gain: None,
            avg_loss: None,
            count: 0,
        }
    }

    pub fn value(&self) -> Option<Decimal> {
        match (self.avg_gain, self.avg_loss) {
            (Some(ag), Some(al)) => {
                if al.is_zero() {
                    Some(dec!(100))
                } else {
                    let rs = ag / al;
                    Some(dec!(100) - (dec!(100) / (Decimal::ONE + rs)))
                }
            }
            _ => None,
        }
    }
}

impl Indicator for Rsi {
    fn next(&mut self, value: Decimal) -> Option<Decimal> {
        if let Some(prev) = self.prev_value {
            let change = value - prev;
            let gain = if change > Decimal::ZERO { change } else { Decimal::ZERO };
            let loss = if change < Decimal::ZERO { change.abs() } else { Decimal::ZERO };

            self.count += 1;

            match self.avg_gain {
                None => {
                    // Accumulate initial period
                    self.gains.push_back(gain);
                    self.losses.push_back(loss);

                    if self.count >= self.len {
                        let sum_gain: Decimal = self.gains.iter().sum();
                        let sum_loss: Decimal = self.losses.iter().sum();
                        let period_dec = Decimal::from(self.len);
                        self.avg_gain = Some(sum_gain / period_dec);
                        self.avg_loss = Some(sum_loss / period_dec);
                    }
                }
                Some(prev_ag) => {
                    // Wilder's smoothing
                    let period_dec = Decimal::from(self.len);
                    self.avg_gain = Some((prev_ag * (period_dec - Decimal::ONE) + gain) / period_dec);
                    self.avg_loss = Some(
                        (self.avg_loss.unwrap() * (period_dec - Decimal::ONE) + loss) / period_dec,
                    );
                }
            }
        }

        self.prev_value = Some(value);
        self.value()
    }

    fn reset(&mut self) {
        self.prev_value = None;
        self.gains.clear();
        self.losses.clear();
        self.avg_gain = None;
        self.avg_loss = None;
        self.count = 0;
    }

    fn period(&self) -> usize {
        self.len + 1 // need one extra data point for the first change
    }

    fn is_ready(&self) -> bool {
        self.avg_gain.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_rsi_basic() {
        let mut rsi = Rsi::new(14);
        // Feed some increasing/decreasing values
        let values = [
            dec!(44), dec!(44.34), dec!(44.09), dec!(43.61), dec!(44.33),
            dec!(44.83), dec!(45.10), dec!(45.42), dec!(45.84), dec!(46.08),
            dec!(45.89), dec!(46.03), dec!(45.61), dec!(46.28), dec!(46.28),
        ];
        let mut result = None;
        for v in &values {
            result = rsi.next(*v);
        }
        assert!(result.is_some());
        let rsi_val = result.unwrap();
        // RSI should be between 0 and 100
        assert!(rsi_val > Decimal::ZERO && rsi_val < dec!(100));
    }
}
