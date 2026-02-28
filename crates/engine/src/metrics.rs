use chrono::{DateTime, Utc};
use propbot_core::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use uuid::Uuid;


/// Compute aggregate backtest results from trade log and equity curve.
pub fn compute_backtest_result(
    strategy_id: String,
    instrument: String,
    initial_balance: Decimal,
    final_account: AccountState,
    trades: Vec<Trade>,
    equity_curve: Vec<EquityPoint>,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
) -> BacktestResult {
    let total_trades = trades.len();
    let winning_trades = trades.iter().filter(|t| t.net_pnl() > Decimal::ZERO).count();
    let losing_trades = trades.iter().filter(|t| t.net_pnl() < Decimal::ZERO).count();

    let gross_profit: Decimal = trades
        .iter()
        .filter(|t| t.pnl > Decimal::ZERO)
        .map(|t| t.pnl)
        .sum();

    let gross_loss: Decimal = trades
        .iter()
        .filter(|t| t.pnl < Decimal::ZERO)
        .map(|t| t.pnl.abs())
        .sum();

    let net_profit: Decimal = trades.iter().map(|t| t.net_pnl()).sum();

    let total_commission: Decimal = trades.iter().map(|t| t.commission).sum();

    let max_drawdown = equity_curve
        .iter()
        .map(|e| e.drawdown)
        .max()
        .unwrap_or(Decimal::ZERO);

    let max_drawdown_percent = if initial_balance.is_zero() {
        Decimal::ZERO
    } else {
        (max_drawdown / initial_balance) * dec!(100)
    };

    let win_rate = if total_trades == 0 {
        Decimal::ZERO
    } else {
        Decimal::from(winning_trades) / Decimal::from(total_trades) * dec!(100)
    };

    let profit_factor = if gross_loss.is_zero() {
        if gross_profit > Decimal::ZERO {
            dec!(999.99) // Infinite profit factor capped
        } else {
            Decimal::ZERO
        }
    } else {
        gross_profit / gross_loss
    };

    let avg_trade_pnl = if total_trades == 0 {
        Decimal::ZERO
    } else {
        net_profit / Decimal::from(total_trades)
    };

    let avg_winner = if winning_trades == 0 {
        Decimal::ZERO
    } else {
        gross_profit / Decimal::from(winning_trades)
    };

    let avg_loser = if losing_trades == 0 {
        Decimal::ZERO
    } else {
        gross_loss / Decimal::from(losing_trades)
    };

    let sharpe_ratio = compute_sharpe(&equity_curve);
    let sortino_ratio = compute_sortino(&equity_curve);

    BacktestResult {
        id: Uuid::new_v4(),
        strategy_id,
        instrument,
        start_date,
        end_date,
        initial_balance,
        final_balance: final_account.equity,
        total_trades,
        winning_trades,
        losing_trades,
        gross_profit,
        gross_loss,
        net_profit,
        max_drawdown,
        max_drawdown_percent,
        win_rate,
        profit_factor,
        sharpe_ratio,
        sortino_ratio,
        avg_trade_pnl,
        avg_winner,
        avg_loser,
        total_commission,
        equity_curve,
        trades,
    }
}

/// Compute annualized Sharpe ratio from equity curve.
fn compute_sharpe(equity_curve: &[EquityPoint]) -> Decimal {
    if equity_curve.len() < 2 {
        return Decimal::ZERO;
    }

    let returns: Vec<Decimal> = equity_curve
        .windows(2)
        .map(|w| {
            if w[0].equity.is_zero() {
                Decimal::ZERO
            } else {
                (w[1].equity - w[0].equity) / w[0].equity
            }
        })
        .collect();

    let n = Decimal::from(returns.len());
    let mean: Decimal = returns.iter().sum::<Decimal>() / n;

    let variance: Decimal = returns
        .iter()
        .map(|r| {
            let diff = *r - mean;
            diff * diff
        })
        .sum::<Decimal>()
        / n;

    let std_dev = propbot_indicators::bollinger::decimal_sqrt(variance);

    if std_dev.is_zero() {
        return Decimal::ZERO;
    }

    // Annualize (assuming daily bars, ~252 trading days)
    let annualization = propbot_indicators::bollinger::decimal_sqrt(dec!(252));
    (mean / std_dev) * annualization
}

/// Compute annualized Sortino ratio (only downside deviation).
fn compute_sortino(equity_curve: &[EquityPoint]) -> Decimal {
    if equity_curve.len() < 2 {
        return Decimal::ZERO;
    }

    let returns: Vec<Decimal> = equity_curve
        .windows(2)
        .map(|w| {
            if w[0].equity.is_zero() {
                Decimal::ZERO
            } else {
                (w[1].equity - w[0].equity) / w[0].equity
            }
        })
        .collect();

    let n = Decimal::from(returns.len());
    let mean: Decimal = returns.iter().sum::<Decimal>() / n;

    let downside_variance: Decimal = returns
        .iter()
        .filter(|r| **r < Decimal::ZERO)
        .map(|r| r * r)
        .sum::<Decimal>()
        / n;

    let downside_dev = propbot_indicators::bollinger::decimal_sqrt(downside_variance);

    if downside_dev.is_zero() {
        return Decimal::ZERO;
    }

    let annualization = propbot_indicators::bollinger::decimal_sqrt(dec!(252));
    (mean / downside_dev) * annualization
}
