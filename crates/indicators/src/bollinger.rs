use crate::sma::Sma;
use crate::Indicator;
use rust_decimal::Decimal;
use std::collections::VecDeque;

/// Bollinger Bands.
///
/// Returns the middle band (SMA). Use `upper()` and `lower()` for the bands.
#[derive(Debug, Clone)]
pub struct BollingerBands {
    len: usize,
    num_std: Decimal,
    sma: Sma,
    buffer: VecDeque<Decimal>,
    upper: Option<Decimal>,
    lower: Option<Decimal>,
    middle: Option<Decimal>,
}

/// Bollinger Bands output.
#[derive(Debug, Clone, Copy)]
pub struct BollingerOutput {
    pub upper: Decimal,
    pub middle: Decimal,
    pub lower: Decimal,
    pub bandwidth: Decimal,
}

impl BollingerBands {
    pub fn new(period: usize, num_std_dev: Decimal) -> Self {
        Self {
            len: period,
            num_std: num_std_dev,
            sma: Sma::new(period),
            buffer: VecDeque::with_capacity(period),
            upper: None,
            lower: None,
            middle: None,
        }
    }

    /// Standard Bollinger Bands (20, 2).
    pub fn default_periods() -> Self {
        Self::new(20, Decimal::TWO)
    }

    /// Compute standard deviation of values in the buffer.
    fn std_dev(&self, mean: Decimal) -> Decimal {
        if self.buffer.len() < 2 {
            return Decimal::ZERO;
        }
        let variance: Decimal = self
            .buffer
            .iter()
            .map(|v| {
                let diff = *v - mean;
                diff * diff
            })
            .sum::<Decimal>()
            / Decimal::from(self.buffer.len());

        // Decimal sqrt approximation using Newton's method
        decimal_sqrt(variance)
    }

    pub fn output(&self) -> Option<BollingerOutput> {
        match (self.upper, self.middle, self.lower) {
            (Some(u), Some(m), Some(l)) => Some(BollingerOutput {
                upper: u,
                middle: m,
                lower: l,
                bandwidth: u - l,
            }),
            _ => None,
        }
    }

    pub fn next_output(&mut self, value: Decimal) -> Option<BollingerOutput> {
        self.buffer.push_back(value);
        if self.buffer.len() > self.len {
            self.buffer.pop_front();
        }

        if let Some(mid) = self.sma.next(value) {
            let sd = self.std_dev(mid);
            self.middle = Some(mid);
            self.upper = Some(mid + self.num_std * sd);
            self.lower = Some(mid - self.num_std * sd);
        }

        self.output()
    }
}

impl Indicator for BollingerBands {
    fn next(&mut self, value: Decimal) -> Option<Decimal> {
        self.next_output(value).map(|o| o.middle)
    }

    fn reset(&mut self) {
        self.sma.reset();
        self.buffer.clear();
        self.upper = None;
        self.lower = None;
        self.middle = None;
    }

    fn period(&self) -> usize {
        self.len
    }

    fn is_ready(&self) -> bool {
        self.middle.is_some()
    }
}

/// Newton's method square root for Decimal.
pub fn decimal_sqrt(value: Decimal) -> Decimal {
    if value.is_zero() || value < Decimal::ZERO {
        return Decimal::ZERO;
    }
    let mut guess = value / Decimal::TWO;
    let epsilon = Decimal::new(1, 10); // 0.0000000001
    for _ in 0..100 {
        let next_guess = (guess + value / guess) / Decimal::TWO;
        let diff = (next_guess - guess).abs();
        guess = next_guess;
        if diff < epsilon {
            break;
        }
    }
    guess
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_bollinger_basic() {
        let mut bb = BollingerBands::new(3, Decimal::TWO);
        assert!(bb.next_output(dec!(10)).is_none());
        assert!(bb.next_output(dec!(11)).is_none());
        let out = bb.next_output(dec!(12)).unwrap();
        assert_eq!(out.middle, dec!(11));
        assert!(out.upper > out.middle);
        assert!(out.lower < out.middle);
    }

    #[test]
    fn test_decimal_sqrt() {
        let result = decimal_sqrt(dec!(4));
        assert!((result - dec!(2)).abs() < dec!(0.0001));

        let result = decimal_sqrt(dec!(9));
        assert!((result - dec!(3)).abs() < dec!(0.0001));
    }
}
