use crate::Indicator;
use rust_decimal::Decimal;
use std::collections::VecDeque;

/// Simple Moving Average (SMA).
#[derive(Debug, Clone)]
pub struct Sma {
    len: usize,
    buffer: VecDeque<Decimal>,
    sum: Decimal,
}

impl Sma {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "SMA period must be > 0");
        Self {
            len: period,
            buffer: VecDeque::with_capacity(period),
            sum: Decimal::ZERO,
        }
    }

    /// Get the current SMA value without feeding new data.
    pub fn value(&self) -> Option<Decimal> {
        if self.buffer.len() == self.len {
            Some(self.sum / Decimal::from(self.len))
        } else {
            None
        }
    }
}

impl Indicator for Sma {
    fn next(&mut self, value: Decimal) -> Option<Decimal> {
        self.sum += value;
        self.buffer.push_back(value);

        if self.buffer.len() > self.len {
            if let Some(removed) = self.buffer.pop_front() {
                self.sum -= removed;
            }
        }

        self.value()
    }

    fn reset(&mut self) {
        self.buffer.clear();
        self.sum = Decimal::ZERO;
    }

    fn period(&self) -> usize {
        self.len
    }

    fn is_ready(&self) -> bool {
        self.buffer.len() == self.len
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_sma_basic() {
        let mut sma = Sma::new(3);
        assert_eq!(sma.next(dec!(1)), None);
        assert_eq!(sma.next(dec!(2)), None);
        assert_eq!(sma.next(dec!(3)), Some(dec!(2)));
        assert_eq!(sma.next(dec!(4)), Some(dec!(3)));
        assert_eq!(sma.next(dec!(5)), Some(dec!(4)));
    }

    #[test]
    fn test_sma_reset() {
        let mut sma = Sma::new(2);
        sma.next(dec!(10));
        sma.next(dec!(20));
        sma.reset();
        assert!(!sma.is_ready());
        assert_eq!(sma.next(dec!(5)), None);
        assert_eq!(sma.next(dec!(15)), Some(dec!(10)));
    }
}
