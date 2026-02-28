use anyhow::Result;
use clap::{Parser, Subcommand};
use rust_decimal::Decimal;
use std::path::PathBuf;
use tracing_subscriber::{fmt, EnvFilter};

#[derive(Parser)]
#[command(name = "propbot")]
#[command(about = "Prop trading bot system — backtest, deploy, and manage trading strategies")]
#[command(version)]
struct Cli {
    /// Log level (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "info")]
    log_level: String,

    /// Database URL
    #[arg(long, env = "DATABASE_URL")]
    database_url: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a backtest
    Backtest {
        /// Strategy name (e.g. "ma_crossover", "donchian_breakout")
        #[arg(short, long)]
        strategy: String,

        /// Instrument symbol (e.g. "ES", "NQ")
        #[arg(short, long)]
        instrument: String,

        /// Path to CSV data file
        #[arg(short, long)]
        data: PathBuf,

        /// Initial account balance
        #[arg(long, default_value = "50000")]
        balance: f64,

        /// Fast MA period (for ma_crossover)
        #[arg(long, default_value = "10")]
        fast_period: usize,

        /// Slow MA period (for ma_crossover)
        #[arg(long, default_value = "20")]
        slow_period: usize,

        /// Quantity per trade
        #[arg(long, default_value = "1")]
        quantity: f64,

        /// Prop firm risk profile (optional: topstep_50k, topstep_100k, mffu_100k, funding_pips_100k)
        #[arg(long)]
        risk_profile: Option<String>,
    },

    /// Start the API server
    Server {
        /// Bind address
        #[arg(short, long, default_value = "0.0.0.0:3000")]
        bind: String,
    },

    /// Import CSV data into the database
    #[command(name = "data")]
    Data {
        #[command(subcommand)]
        command: DataCommands,
    },

    /// List available strategies
    Strategies,

    /// List available prop firm risk profiles
    RiskProfiles,
}

#[derive(Subcommand)]
enum DataCommands {
    /// Import bars from a CSV file
    Import {
        /// Path to CSV file
        #[arg(short, long)]
        file: PathBuf,

        /// Instrument symbol to assign
        #[arg(short, long)]
        instrument: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&cli.log_level));
    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();

    match cli.command {
        Commands::Backtest {
            strategy,
            instrument,
            data,
            balance,
            fast_period,
            slow_period,
            quantity,
            risk_profile,
        } => {
            run_backtest(
                strategy,
                instrument,
                data,
                balance,
                fast_period,
                slow_period,
                quantity,
                risk_profile,
            )
            .await?;
        }
        Commands::Server { bind } => {
            let database_url = cli
                .database_url
                .unwrap_or_else(|| "postgres://propbot:propbot@localhost:5432/propbot".to_string());
            let pool = sqlx::PgPool::connect(&database_url).await?;
            propbot_data::db::run_migrations(&pool).await
                .map_err(|e| anyhow::anyhow!("Migration failed: {}", e))?;
            propbot_api::start_server(pool, &bind).await?;
        }
        Commands::Data { command } => match command {
            DataCommands::Import { file, instrument } => {
                import_data(file, instrument, cli.database_url).await?;
            }
        },
        Commands::Strategies => {
            println!("Available strategies:");
            println!("  ma_crossover     - Moving Average Crossover (fast/slow EMA or SMA)");
            println!("  donchian_breakout - Donchian Channel Breakout with ATR trailing stop");
        }
        Commands::RiskProfiles => {
            println!("Built-in prop firm risk profiles:");
            println!("  topstep_50k       - TopStep $50K   (daily: $1,000  | drawdown: $2,000 trailing)");
            println!("  topstep_100k      - TopStep $100K  (daily: $2,000  | drawdown: $3,000 trailing)");
            println!("  topstep_150k      - TopStep $150K  (daily: $3,000  | drawdown: $4,500 trailing)");
            println!("  mffu_100k         - MFFU $100K     (daily: $2,000  | drawdown: $3,000 trailing + consistency)");
            println!("  funding_pips_100k - FundingPips $100K (daily: $4,000 | drawdown: $8,000 fixed)");
        }
    }

    Ok(())
}

async fn run_backtest(
    strategy_name: String,
    instrument_symbol: String,
    data_path: PathBuf,
    balance: f64,
    fast_period: usize,
    slow_period: usize,
    quantity: f64,
    risk_profile_name: Option<String>,
) -> Result<()> {
    use propbot_brokers_common::simulated::SimulatedBrokerConfig;
    use propbot_core::*;
    use propbot_data::csv_loader;
    use propbot_engine::run_backtest;
    use propbot_risk::{PropFirmProfile, PropFirmRiskManager};
    use propbot_strategies::donchian_breakout::{DonchianBreakoutConfig, DonchianBreakoutStrategy};
    use propbot_strategies::ma_crossover::{MaCrossoverConfig, MaCrossoverStrategy};

    tracing::info!(
        strategy = %strategy_name,
        instrument = %instrument_symbol,
        data = %data_path.display(),
        "Starting backtest"
    );

    // Load data
    let bars = csv_loader::load_bars_from_csv(&data_path)?;
    tracing::info!(bars = bars.len(), "Loaded historical data");

    if bars.is_empty() {
        anyhow::bail!("No bars loaded from CSV file");
    }

    // Create instrument
    let instrument = Instrument {
        symbol: instrument_symbol.clone(),
        asset_class: AssetClass::Futures,
        tick_size: Decimal::new(25, 2),
        tick_value: Decimal::new(1250, 2),
        contract_size: Decimal::ONE,
        currency: "USD".to_string(),
        exchange: None,
    };

    let qty = Decimal::try_from(quantity).unwrap_or(Decimal::ONE);
    let initial_balance = Decimal::try_from(balance).unwrap_or(Decimal::new(50000, 0));

    // Create strategy
    let mut strategy: Box<dyn Strategy> = match strategy_name.as_str() {
        "donchian_breakout" => Box::new(DonchianBreakoutStrategy::new(DonchianBreakoutConfig {
            instrument: instrument_symbol.clone(),
            quantity: qty,
            ..Default::default()
        })),
        _ => Box::new(MaCrossoverStrategy::new(MaCrossoverConfig {
            instrument: instrument_symbol.clone(),
            fast_period,
            slow_period,
            quantity: qty,
            ma_type: "ema".to_string(),
        })),
    };

    // Create risk manager
    let mut risk_manager = risk_profile_name.map(|name| {
        let profile = match name.as_str() {
            "topstep_50k" => PropFirmProfile::topstep_50k(),
            "topstep_100k" => PropFirmProfile::topstep_100k(),
            "topstep_150k" => PropFirmProfile::topstep_150k(),
            "mffu_100k" => PropFirmProfile::mffu_100k(),
            "funding_pips_100k" => PropFirmProfile::funding_pips_100k(),
            _ => {
                tracing::warn!(profile = %name, "Unknown risk profile, using TopStep 50K");
                PropFirmProfile::topstep_50k()
            }
        };
        PropFirmRiskManager::new(profile)
    });

    let broker_config = SimulatedBrokerConfig {
        initial_balance,
        ..Default::default()
    };

    let config = propbot_engine::BacktestConfig {
        instrument,
        broker_config,
    };

    // Run backtest
    let result = run_backtest(
        bars,
        strategy.as_mut(),
        risk_manager.as_mut(),
        config,
    )
    .await;

    // Print results
    let sep = "=".repeat(60);
    println!("\n{sep}");
    println!("  BACKTEST RESULTS");
    println!("{sep}");
    println!("  Strategy:        {}", result.strategy_id);
    println!("  Instrument:      {}", result.instrument);
    println!("  Period:          {} → {}", result.start_date.format("%Y-%m-%d"), result.end_date.format("%Y-%m-%d"));
    println!("  Initial Balance: ${:.2}", result.initial_balance);
    println!("  Final Balance:   ${:.2}", result.final_balance);
    println!("  Net Profit:      ${:.2}", result.net_profit);
    println!("  Total Trades:    {}", result.total_trades);
    println!("  Win Rate:        {:.1}%", result.win_rate);
    println!("  Profit Factor:   {:.2}", result.profit_factor);
    println!("  Max Drawdown:    ${:.2} ({:.1}%)", result.max_drawdown, result.max_drawdown_percent);
    println!("  Sharpe Ratio:    {:.2}", result.sharpe_ratio);
    println!("  Sortino Ratio:   {:.2}", result.sortino_ratio);
    println!("  Avg Winner:      ${:.2}", result.avg_winner);
    println!("  Avg Loser:       ${:.2}", result.avg_loser);
    println!("  Commission:      ${:.2}", result.total_commission);
    println!("{sep}\n");

    Ok(())
}

async fn import_data(
    file: PathBuf,
    instrument: String,
    database_url: Option<String>,
) -> Result<()> {
    let database_url =
        database_url.unwrap_or_else(|| "postgres://propbot:propbot@localhost:5432/propbot".to_string());
    let pool = sqlx::PgPool::connect(&database_url).await?;
    propbot_data::db::run_migrations(&pool).await
        .map_err(|e| anyhow::anyhow!("Migration failed: {}", e))?;

    tracing::info!(file = %file.display(), instrument = %instrument, "Importing CSV data");

    let mut bars = propbot_data::csv_loader::load_bars_from_csv(&file)?;

    // Override instrument name
    for bar in &mut bars {
        bar.instrument = instrument.clone();
    }

    let count = propbot_data::db::insert_bars(&pool, &bars).await
        .map_err(|e| anyhow::anyhow!("Insert failed: {}", e))?;

    tracing::info!(count = count, "Data import complete");
    println!("Imported {} bars for {}", count, instrument);

    Ok(())
}
