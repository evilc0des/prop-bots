pub mod atr;
pub mod bollinger;
pub mod donchian;
pub mod ema;
pub mod macd;
pub mod rsi;
pub mod sma;
pub mod stochastic;
pub mod vwap;

use rust_decimal::Decimal;

/// Trait for streaming (incremental) indicators.
/// Feed one value at a time; the indicator maintains internal state.
pub trait Indicator: Send + Sync {
    /// Process the next value and return the indicator output (if ready).
    fn next(&mut self, value: Decimal) -> Option<Decimal>;

    /// Reset the indicator to its initial state.
    fn reset(&mut self);

    /// The minimum number of data points needed before the indicator produces output.
    fn period(&self) -> usize;

    /// Whether the indicator has enough data to produce output.
    fn is_ready(&self) -> bool;
}
