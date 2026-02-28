use crate::Indicator;
use rust_decimal::Decimal;
use std::collections::VecDeque;

/// Donchian Channel.
///
/// Upper band = highest high over N periods.
/// Lower band = lowest low over N periods.
/// Middle band = (upper + lower) / 2.
#[derive(Debug, Clone)]
pub struct DonchianChannel {
    len: usize,
    highs: VecDeque<Decimal>,
    lows: VecDeque<Decimal>,
    upper: Option<Decimal>,
    lower: Option<Decimal>,
}

#[derive(Debug, Clone, Copy)]
pub struct DonchianOutput {
    pub upper: Decimal,
    pub middle: Decimal,
    pub lower: Decimal,
}

impl DonchianChannel {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "Donchian period must be > 0");
        Self {
            len: period,
            highs: VecDeque::with_capacity(period),
            lows: VecDeque::with_capacity(period),
            upper: None,
            lower: None,
        }
    }

    pub fn next_hl(&mut self, high: Decimal, low: Decimal) -> Option<DonchianOutput> {
        self.highs.push_back(high);
        self.lows.push_back(low);

        if self.highs.len() > self.len {
            self.highs.pop_front();
            self.lows.pop_front();
        }

        if self.highs.len() < self.len {
            return None;
        }

        let upper = self.highs.iter().max().copied().unwrap();
        let lower = self.lows.iter().min().copied().unwrap();
        self.upper = Some(upper);
        self.lower = Some(lower);

        Some(DonchianOutput {
            upper,
            middle: (upper + lower) / Decimal::TWO,
            lower,
        })
    }

    pub fn output(&self) -> Option<DonchianOutput> {
        match (self.upper, self.lower) {
            (Some(u), Some(l)) => Some(DonchianOutput {
                upper: u,
                middle: (u + l) / Decimal::TWO,
                lower: l,
            }),
            _ => None,
        }
    }
}

impl Indicator for DonchianChannel {
    fn next(&mut self, value: Decimal) -> Option<Decimal> {
        self.next_hl(value, value).map(|o| o.middle)
    }

    fn reset(&mut self) {
        self.highs.clear();
        self.lows.clear();
        self.upper = None;
        self.lower = None;
    }

    fn period(&self) -> usize {
        self.len
    }

    fn is_ready(&self) -> bool {
        self.upper.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_donchian_basic() {
        let mut dc = DonchianChannel::new(3);
        assert!(dc.next_hl(dec!(10), dec!(8)).is_none());
        assert!(dc.next_hl(dec!(12), dec!(9)).is_none());
        let out = dc.next_hl(dec!(11), dec!(7)).unwrap();
        assert_eq!(out.upper, dec!(12));
        assert_eq!(out.lower, dec!(7));
    }
}
