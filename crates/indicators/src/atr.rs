use crate::Indicator;
use rust_decimal::Decimal;
use std::collections::VecDeque;

/// Average True Range (ATR).
///
/// Requires high, low, close data. Feed via `next_hlc()` or use `next()` with close
/// (in which case ATR acts as a simple moving average of absolute changes — less accurate).
#[derive(Debug, Clone)]
pub struct Atr {
    len: usize,
    prev_close: Option<Decimal>,
    tr_values: VecDeque<Decimal>,
    current_atr: Option<Decimal>,
    count: usize,
}

impl Atr {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "ATR period must be > 0");
        Self {
            len: period,
            prev_close: None,
            tr_values: VecDeque::with_capacity(period),
            current_atr: None,
            count: 0,
        }
    }

    /// Feed high, low, close and compute ATR (preferred method).
    pub fn next_hlc(&mut self, high: Decimal, low: Decimal, close: Decimal) -> Option<Decimal> {
        let tr = match self.prev_close {
            Some(prev_c) => {
                let hl = high - low;
                let hc = (high - prev_c).abs();
                let lc = (low - prev_c).abs();
                hl.max(hc).max(lc)
            }
            None => high - low,
        };
        self.prev_close = Some(close);
        self.count += 1;

        match self.current_atr {
            None => {
                self.tr_values.push_back(tr);
                if self.count >= self.len {
                    let sum: Decimal = self.tr_values.iter().sum();
                    self.current_atr = Some(sum / Decimal::from(self.len));
                }
            }
            Some(prev_atr) => {
                // Wilder's smoothing
                let period_dec = Decimal::from(self.len);
                self.current_atr = Some((prev_atr * (period_dec - Decimal::ONE) + tr) / period_dec);
            }
        }

        self.current_atr
    }

    pub fn value(&self) -> Option<Decimal> {
        self.current_atr
    }
}

impl Indicator for Atr {
    fn next(&mut self, value: Decimal) -> Option<Decimal> {
        // Simplified: treat each value as a "close" and compute TR as abs(change)
        let tr = match self.prev_close {
            Some(prev) => (value - prev).abs(),
            None => {
                self.prev_close = Some(value);
                return None;
            }
        };
        self.prev_close = Some(value);
        self.count += 1;

        match self.current_atr {
            None => {
                self.tr_values.push_back(tr);
                if self.tr_values.len() >= self.len {
                    let sum: Decimal = self.tr_values.iter().sum();
                    self.current_atr = Some(sum / Decimal::from(self.len));
                }
            }
            Some(prev_atr) => {
                let period_dec = Decimal::from(self.len);
                self.current_atr = Some((prev_atr * (period_dec - Decimal::ONE) + tr) / period_dec);
            }
        }

        self.current_atr
    }

    fn reset(&mut self) {
        self.prev_close = None;
        self.tr_values.clear();
        self.current_atr = None;
        self.count = 0;
    }

    fn period(&self) -> usize {
        self.len
    }

    fn is_ready(&self) -> bool {
        self.current_atr.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_atr_hlc() {
        let mut atr = Atr::new(3);
        assert!(atr.next_hlc(dec!(48.70), dec!(47.79), dec!(48.16)).is_none());
        assert!(atr.next_hlc(dec!(48.72), dec!(48.14), dec!(48.61)).is_none());
        let result = atr.next_hlc(dec!(48.90), dec!(48.39), dec!(48.75));
        assert!(result.is_some());
    }
}
