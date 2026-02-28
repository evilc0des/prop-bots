use chrono::{DateTime, Utc};
use propbot_core::{Bar, BacktestResult, Tick};
use rust_decimal::Decimal;
use sqlx::{PgPool, Row};

/// Run embedded migrations.
pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("../../migrations").run(pool).await?;
    Ok(())
}

/// Load bars from the database.
pub async fn load_bars(
    pool: &PgPool,
    instrument: &str,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<Vec<Bar>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT instrument, timestamp, open, high, low, close, volume
         FROM bars
         WHERE instrument = $1 AND timestamp >= $2 AND timestamp <= $3
         ORDER BY timestamp ASC",
    )
    .bind(instrument)
    .bind(start)
    .bind(end)
    .fetch_all(pool)
    .await?;

    let bars = rows
        .iter()
        .map(|r| Bar {
            instrument: r.get("instrument"),
            timestamp: r.get("timestamp"),
            open: r.get("open"),
            high: r.get("high"),
            low: r.get("low"),
            close: r.get("close"),
            volume: r.get("volume"),
        })
        .collect();

    Ok(bars)
}

/// Load ticks from the database.
pub async fn load_ticks(
    pool: &PgPool,
    instrument: &str,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<Vec<Tick>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT instrument, timestamp, bid, ask, last_price, volume
         FROM ticks
         WHERE instrument = $1 AND timestamp >= $2 AND timestamp <= $3
         ORDER BY timestamp ASC",
    )
    .bind(instrument)
    .bind(start)
    .bind(end)
    .fetch_all(pool)
    .await?;

    let ticks = rows
        .iter()
        .map(|r| Tick {
            instrument: r.get("instrument"),
            timestamp: r.get("timestamp"),
            bid: r.get("bid"),
            ask: r.get("ask"),
            last: r.get::<Decimal, _>("last_price"),
            volume: r.get("volume"),
        })
        .collect();

    Ok(ticks)
}

/// Insert bars into the database.
pub async fn insert_bars(pool: &PgPool, bars: &[Bar]) -> Result<u64, sqlx::Error> {
    let mut count = 0u64;
    for bar in bars {
        sqlx::query(
            "INSERT INTO bars (instrument, timestamp, open, high, low, close, volume)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             ON CONFLICT (instrument, timestamp) DO UPDATE
             SET open = EXCLUDED.open, high = EXCLUDED.high,
                 low = EXCLUDED.low, close = EXCLUDED.close, volume = EXCLUDED.volume",
        )
        .bind(&bar.instrument)
        .bind(bar.timestamp)
        .bind(bar.open)
        .bind(bar.high)
        .bind(bar.low)
        .bind(bar.close)
        .bind(bar.volume)
        .execute(pool)
        .await?;
        count += 1;
    }
    Ok(count)
}

/// Insert ticks into the database.
pub async fn insert_ticks(pool: &PgPool, ticks: &[Tick]) -> Result<u64, sqlx::Error> {
    let mut count = 0u64;
    for tick in ticks {
        sqlx::query(
            "INSERT INTO ticks (instrument, timestamp, bid, ask, last_price, volume)
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(&tick.instrument)
        .bind(tick.timestamp)
        .bind(tick.bid)
        .bind(tick.ask)
        .bind(tick.last)
        .bind(tick.volume)
        .execute(pool)
        .await?;
        count += 1;
    }
    Ok(count)
}

/// Get available instruments from the bars table.
pub async fn available_instruments(pool: &PgPool) -> Result<Vec<String>, sqlx::Error> {
    let rows = sqlx::query("SELECT DISTINCT instrument FROM bars ORDER BY instrument")
        .fetch_all(pool)
        .await?;
    Ok(rows.iter().map(|r| r.get("instrument")).collect())
}

/// Save a backtest result.
pub async fn save_backtest_result(
    pool: &PgPool,
    result: &BacktestResult,
) -> Result<(), sqlx::Error> {
    let trades_json = serde_json::to_value(&result.trades).unwrap_or_default();
    let equity_json = serde_json::to_value(&result.equity_curve).unwrap_or_default();

    sqlx::query(
        "INSERT INTO backtest_results (
            id, strategy_id, instrument, start_date, end_date,
            initial_balance, final_balance, total_trades, winning_trades, losing_trades,
            gross_profit, gross_loss, net_profit, max_drawdown, max_drawdown_percent,
            win_rate, profit_factor, sharpe_ratio, sortino_ratio,
            avg_trade_pnl, avg_winner, avg_loser, total_commission,
            equity_curve, trades
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
            $11, $12, $13, $14, $15, $16, $17, $18, $19,
            $20, $21, $22, $23, $24, $25
        )",
    )
    .bind(result.id)
    .bind(&result.strategy_id)
    .bind(&result.instrument)
    .bind(result.start_date)
    .bind(result.end_date)
    .bind(result.initial_balance)
    .bind(result.final_balance)
    .bind(result.total_trades as i64)
    .bind(result.winning_trades as i64)
    .bind(result.losing_trades as i64)
    .bind(result.gross_profit)
    .bind(result.gross_loss)
    .bind(result.net_profit)
    .bind(result.max_drawdown)
    .bind(result.max_drawdown_percent)
    .bind(result.win_rate)
    .bind(result.profit_factor)
    .bind(result.sharpe_ratio)
    .bind(result.sortino_ratio)
    .bind(result.avg_trade_pnl)
    .bind(result.avg_winner)
    .bind(result.avg_loser)
    .bind(result.total_commission)
    .bind(&equity_json)
    .bind(&trades_json)
    .execute(pool)
    .await?;

    Ok(())
}
