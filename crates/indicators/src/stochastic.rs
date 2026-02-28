use crate::Indicator;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Stochastic Oscillator (%K and %D).
///
/// %K = (Close - Lowest Low) / (Highest High - Lowest Low) * 100
/// %D = SMA(%K, d_period)
#[derive(Debug, Clone)]
pub struct Stochastic {
    k_period: usize,
    d_period: usize,
    highs: Vec<Decimal>,
    lows: Vec<Decimal>,
    k_values: Vec<Decimal>,
    current_k: Option<Decimal>,
    current_d: Option<Decimal>,
}

#[derive(Debug, Clone, Copy)]
pub struct StochasticOutput {
    pub k: Decimal,
    pub d: Decimal,
}

impl Stochastic {
    pub fn new(k_period: usize, d_period: usize) -> Self {
        Self {
            k_period,
            d_period,
            highs: Vec::new(),
            lows: Vec::new(),
            k_values: Vec::new(),
            current_k: None,
            current_d: None,
        }
    }

    /// Standard Stochastic (14, 3).
    pub fn default_periods() -> Self {
        Self::new(14, 3)
    }

    pub fn next_hlc(&mut self, high: Decimal, low: Decimal, close: Decimal) -> Option<StochasticOutput> {
        self.highs.push(high);
        self.lows.push(low);

        if self.highs.len() > self.k_period {
            self.highs.remove(0);
            self.lows.remove(0);
        }

        if self.highs.len() < self.k_period {
            return None;
        }

        let highest = self.highs.iter().max().copied().unwrap();
        let lowest = self.lows.iter().min().copied().unwrap();

        let range = highest - lowest;
        let k = if range.is_zero() {
            dec!(50)
        } else {
            ((close - lowest) / range) * dec!(100)
        };

        self.current_k = Some(k);
        self.k_values.push(k);

        if self.k_values.len() >= self.d_period {
            let d_values: Decimal = self.k_values[self.k_values.len() - self.d_period..].iter().sum();
            let d = d_values / Decimal::from(self.d_period);
            self.current_d = Some(d);

            Some(StochasticOutput { k, d })
        } else {
            None
        }
    }

    pub fn output(&self) -> Option<StochasticOutput> {
        match (self.current_k, self.current_d) {
            (Some(k), Some(d)) => Some(StochasticOutput { k, d }),
            _ => None,
        }
    }
}

impl Indicator for Stochastic {
    fn next(&mut self, value: Decimal) -> Option<Decimal> {
        // Simplified: use value as high, low, and close
        self.next_hlc(value, value, value).map(|o| o.k)
    }

    fn reset(&mut self) {
        self.highs.clear();
        self.lows.clear();
        self.k_values.clear();
        self.current_k = None;
        self.current_d = None;
    }

    fn period(&self) -> usize {
        self.k_period
    }

    fn is_ready(&self) -> bool {
        self.current_d.is_some()
    }
}
