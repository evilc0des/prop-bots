# Rust Trading Bot System for Prop Firms

## Overview

Build a high-performance, event-driven trading system in Rust with a Cargo workspace of modular crates. The engine supports both backtesting (CSV/API data) and live execution through broker adapters (NinjaTrader 8 priority, then MT5, crypto, GUI automation). A prop-firm-aware risk manager enforces rules for TopStep, MFFU, FundingPips, etc. Strategies support both rule-based indicators and ML inference (ONNX Runtime). An Axum REST/WebSocket API exposes the system to a JS frontend dashboard. A companion C# addon bridges NinjaTrader 8 via TCP sockets.

## Repository Structure

```
prop-bots/
├── crates/
│   ├── core/              # Domain types and traits
│   ├── engine/            # Event-driven execution engine
│   ├── strategies/        # Trading strategies (rule-based + ML)
│   ├── indicators/        # Technical indicators library
│   ├── data/              # Data loading and storage
│   ├── brokers/           # Broker adapters
│   │   ├── common/        # Shared broker code
│   │   ├── ninjatrader/   # NinjaTrader 8 bridge
│   │   ├── metatrader/    # MetaTrader 5 bridge
│   │   ├── crypto/        # Crypto exchange APIs
│   │   └── gui/           # GUI automation fallback
│   ├── risk/              # Prop firm risk management
│   ├── api/               # REST/WebSocket API server
│   └── cli/               # Command-line interface
├── nt8-addon/             # NinjaTrader 8 C# addon
├── dashboard/             # Web frontend
├── migrations/            # Database migrations
├── config/                # Configuration files
├── data/sample/           # Sample data for testing
└── Cargo.toml             # Workspace manifest
```

## Implementation Steps

### Step 1 — Scaffold the Cargo workspace and crate skeletons

Create root `Cargo.toml` with `[workspace]` listing all crates. Initialize each crate under `crates/` with `cargo init --lib` (except `cli` which is `--bin`). Set up shared dependencies in `[workspace.dependencies]`: `tokio`, `serde`, `sqlx`, `axum`, `tracing`, `chrono`, `rust_decimal`. Add `.gitignore`, `rustfmt.toml`, `clippy.toml`.

### Step 2 — Define core domain types and traits (`crates/core`)

**Types:**
- `Instrument` — futures/CFD/crypto with symbol, tick size, contract size, asset class enum
- `Bar` — OHLCV + timestamp
- `Tick` — bid/ask/last + timestamp
- `Order` — market/limit/stop, side, qty, state machine
- `Position` — entry, qty, unrealized PnL
- `Trade` — closed position with PnL
- `AccountState` — balance, equity, margin

**Event enum:**
- `MarketData(Bar|Tick)`
- `OrderEvent(Fill|Reject|Cancel)`
- `Signal(BuyEntry|SellEntry|Exit)`
- `RiskEvent(Violation)`
- `SystemEvent(Start|Stop|Error)`

**Traits:**
- `Strategy` — receives events, emits signals
- `Broker` — submit/cancel orders, stream data
- `DataProvider` — load historical bars/ticks
- `RiskManager` — approve/reject orders

Use `rust_decimal::Decimal` for all monetary values to avoid floating-point errors.

### Step 3 — Build the technical indicator library (`crates/indicators`)

Implement an incremental (streaming) indicator framework — each indicator takes one new value at a time and returns the computed result without needing the full history. 

**Indicators:**
- SMA, EMA, RSI, MACD, Bollinger Bands, ATR, Stochastic, VWAP, Donchian Channel

Each implements an `Indicator` trait with `fn next(&mut self, value: Decimal) -> Option<Decimal>`. Composable: indicators can chain (e.g., RSI of ATR).

### Step 4 — Implement the data layer (`crates/data`)

**CSV loader:** Parse OHLCV CSVs with configurable column mapping and date formats. Support both bar and tick data.

**API loader:** Trait-based `HistoricalDataApi` with initial impls for free sources (e.g., Databento, Polygon.io, or exchange APIs for crypto).

**PostgreSQL storage:** Use `sqlx` with compile-time checked queries. 
- **Tables:** `instruments`, `bars`, `ticks`, `trades`, `orders`, `backtest_results`, `strategy_configs`, `prop_firm_rules`
- Write migration files in `migrations/`

**Normalization:** Uniform `Bar`/`Tick` format regardless of source.

### Step 5 — Build the event-driven engine (`crates/engine`)

**Unified event loop:** Both backtest and live modes use the same `Engine` struct parameterized by a `Broker` impl and a `DataProvider` impl.

**Backtest mode:** `SimulatedBroker` replays historical data bar-by-bar (or tick-by-tick), simulates fills with configurable slippage/commission models. Produces equity curve, trade log, performance metrics (Sharpe, Sortino, max drawdown, win rate, profit factor).

**Live mode:** Connects to a real `Broker` impl, processes streaming market data, routes signals through the risk manager before order submission.

**Concurrency:** Use tokio channels (`mpsc`, `broadcast`) for event routing between components. Each component (data feed, strategy, risk, broker) runs as a separate tokio task.

### Step 6 — Implement prop firm risk management (`crates/risk`)

**Rule profiles:** Define prop firm rules as config (TOML): daily loss limit, max drawdown (fixed or trailing), max position size, trading hours, max contracts, consistency rules.

**Pre-built profiles:** TopStep, MFFU, FundingPips with their specific rule sets.

**Pre-trade checks:** Before every order, validate against current account state + rules. Reject if violation would occur.

**Auto-flatten:** If approaching a limit (configurable threshold, e.g., 90% of daily loss), auto-close all positions.

**Tracking:** Real-time tracking of high-water mark for trailing drawdown, daily PnL reset, cumulative metrics.

### Step 7 — Create the strategy framework (`crates/strategies`)

**Rule-based:** Strategies compose indicators and emit signals. 
- **Example strategies:** MA crossover, breakout (Donchian), mean reversion (Bollinger bounce)

**ML inference:** Integrate `ort` (ONNX Runtime for Rust) crate. Strategies load a pre-trained ONNX model, feed feature vectors (from indicators/raw data), and interpret model output as signals. Training happens externally (Python/Jupyter), only inference in Rust.

**Configuration:** Each strategy has a TOML config for parameters (periods, thresholds, model paths). Hot-reloadable in live mode.

### Step 8 — Build the NinjaTrader 8 bridge (priority broker)

**C# addon (`nt8-addon/`):** A NinjaTrader addon (NinjaScript) that runs a TCP server inside NT8. It streams market data (bars, ticks, DOM) as JSON messages and accepts order commands (submit, cancel, modify). Handles connection lifecycle, reconnection, and heartbeats.

**Rust client (`crates/brokers/ninjatrader`):** Implements the `Broker` trait. Connects to the C# addon via TCP, deserializes market data events, serializes order commands. Manages connection state, buffering, and reconnection.

**Protocol:** JSON over TCP with length-prefixed framing. 
- **Message types:** `MarketData`, `OrderSubmit`, `OrderUpdate`, `AccountUpdate`, `Heartbeat`

### Step 9 — Implement additional broker adapters

**MetaTrader 5 (`crates/brokers/metatrader`):** TCP/WebSocket bridge similar to NT8 approach — a Python or MQL5 EA runs inside MT5, communicates with Rust. Alternatively, use ZeroMQ bridge pattern.

**Crypto (`crates/brokers/crypto`):** Direct REST + WebSocket to exchanges. Start with Binance Futures. Use `reqwest` for REST, `tokio-tungstenite` for WebSocket. Handle rate limits, authentication (HMAC signing).

**GUI automation (`crates/brokers/gui`):** Use `enigo` crate for input simulation, `xcap` or platform-specific screen capture for reading prices/positions. Define screen regions via config. This is the fallback for platforms with no API.

### Step 10 — Build the API server (`crates/api`)

**Axum REST endpoints:**
- `POST /bots` — create
- `GET /bots` — list
- `POST /bots/:id/start`
- `POST /bots/:id/stop`
- `GET /bots/:id/status`
- `POST /backtest` — run backtest
- `GET /backtest/:id/results`
- `GET /strategies` — list available
- CRUD `/config/prop-firms`

**WebSocket:** Real-time streaming of positions, PnL, equity curve, log messages, risk metrics.

**Auth:** Simple API key or JWT for dashboard access.

### Step 11 — Build the web dashboard (`dashboard/`)

**Framework:** React (Vite + TypeScript) or Svelte — pick based on preference.

**Pages:**
- Dashboard overview (active bots, daily PnL, risk status)
- Strategy configurator (parameter forms)
- Backtest runner (upload CSV / select data, configure strategy, view results with equity curve charts using Recharts or Chart.js)
- Live monitor (real-time positions, order book, trade log)
- Prop firm rules editor

**Charts:** Equity curves, drawdown plots, candlestick charts with trade markers.

### Step 12 — Build the CLI (`crates/cli`)

Use `clap` for argument parsing.

**Commands:**
- `backtest` — run backtest with strategy + data args
- `live` — start live bot
- `server` — start API server
- `data import` — load CSV into DB
- `data fetch` — download from API source
- `strategies list`
- `risk check` — validate config against prop firm rules

### Step 13 — Testing and CI

**Unit tests:** In each crate (indicator correctness, order state machines, risk rule logic).

**Integration tests:** End-to-end backtest with sample CSV data producing expected metrics.

**CI:** GitHub Actions — `cargo test`, `cargo clippy`, `cargo fmt --check`, build the C# addon, build the dashboard.

**Docker Compose:** Rust app + PostgreSQL + dashboard for local dev.

## Verification

### Backtest Correctness
Run MA crossover strategy on known sample data CSV, compare equity curve and trade log against manually calculated expected results.

### Risk Rules
Unit tests that simulate account states hitting daily loss limits, trailing drawdown, and verify auto-flatten triggers.

### NT8 Bridge
Integration test with a mock TCP server mimicking the C# addon protocol.

### API
HTTP integration tests using `axum::test` utilities.

### End-to-End
Docker Compose up → import CSV → run backtest via CLI → view results on dashboard.

## Key Decisions

| Decision | Rationale |
|----------|-----------|
| **Rust over Python** | Higher performance for backtesting and live execution, at the cost of fewer ready-made trading libraries (we build our own) |
| **PostgreSQL over SQLite** | Better suited for concurrent access, time-series queries, and future scaling |
| **ONNX for ML** | Train models externally in Python, deploy inference in Rust — clean separation of concerns |
| **Socket bridge for NT8** | NinjaTrader is .NET-only; TCP JSON protocol is the cleanest cross-language bridge |
| **Event-driven engine** | Same engine for backtest and live avoids "two codebases" divergence bug |

## Technology Stack

### Backend
- **Language:** Rust (stable)
- **Async runtime:** Tokio
- **Web framework:** Axum
- **Database:** PostgreSQL + SQLx
- **Serialization:** Serde
- **ML inference:** ONNX Runtime (ort crate)
- **Decimal arithmetic:** rust_decimal

### Frontend
- **Framework:** React (Vite) or Svelte
- **Language:** TypeScript
- **Charts:** Recharts / Chart.js
- **Real-time:** WebSocket

### Broker Bridges
- **NinjaTrader 8:** C# addon + TCP JSON protocol
- **MetaTrader 5:** Python/MQL5 bridge + TCP/ZeroMQ
- **Crypto:** Native REST + WebSocket clients
- **GUI automation:** enigo + xcap

### DevOps
- **CI/CD:** GitHub Actions
- **Containerization:** Docker + Docker Compose
- **Testing:** cargo test, integration tests
- **Linting:** clippy, rustfmt