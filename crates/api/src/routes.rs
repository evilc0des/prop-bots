use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub fn api_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Health
        .route("/health", get(health_check))
        // Strategies
        .route("/strategies", get(list_strategies))
        // Backtests
        .route("/backtest", post(run_backtest))
        .route("/backtest/{id}", get(get_backtest_result))
        // Bots
        .route("/bots", get(list_bots))
        .route("/bots", post(create_bot))
        .route("/bots/{id}/start", post(start_bot))
        .route("/bots/{id}/stop", post(stop_bot))
        .route("/bots/{id}/status", get(bot_status))
        // Prop firm profiles
        .route("/risk/profiles", get(list_risk_profiles))
        .route("/risk/profiles", post(create_risk_profile))
}

// ---------------------------------------------------------------------------
// Health
// ---------------------------------------------------------------------------

async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct StrategyInfo {
    id: String,
    name: String,
    description: String,
    parameters: serde_json::Value,
}

async fn list_strategies() -> impl IntoResponse {
    let strategies = vec![
        StrategyInfo {
            id: "ma_crossover".to_string(),
            name: "MA Crossover".to_string(),
            description: "Moving average crossover strategy with configurable fast/slow periods"
                .to_string(),
            parameters: serde_json::json!({
                "instrument": "string",
                "fast_period": "integer",
                "slow_period": "integer",
                "quantity": "decimal",
                "ma_type": "sma | ema"
            }),
        },
        StrategyInfo {
            id: "donchian_breakout".to_string(),
            name: "Donchian Breakout".to_string(),
            description: "Donchian channel breakout with ATR trailing stop".to_string(),
            parameters: serde_json::json!({
                "instrument": "string",
                "channel_period": "integer",
                "atr_period": "integer",
                "atr_stop_multiplier": "decimal",
                "quantity": "decimal"
            }),
        },
    ];
    Json(strategies)
}

// ---------------------------------------------------------------------------
// Backtests
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
#[allow(dead_code)]
struct BacktestRequest {
    strategy: String,
    instrument: String,
    #[serde(default)]
    parameters: serde_json::Value,
    start_date: Option<String>,
    end_date: Option<String>,
    initial_balance: Option<f64>,
    risk_profile: Option<String>,
}

async fn run_backtest(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<BacktestRequest>,
) -> impl IntoResponse {
    // TODO: Implement backtest execution via engine
    (
        StatusCode::ACCEPTED,
        Json(serde_json::json!({
            "status": "queued",
            "message": "Backtest queued for execution",
            "strategy": req.strategy,
            "instrument": req.instrument,
        })),
    )
}

async fn get_backtest_result(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // TODO: Look up from database
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({
            "error": "Backtest result not found",
            "id": id,
        })),
    )
}

// ---------------------------------------------------------------------------
// Bots
// ---------------------------------------------------------------------------

async fn list_bots(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let bots = state.active_bots.read().await;
    let list: Vec<_> = bots.values().cloned().collect();
    Json(list)
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct CreateBotRequest {
    strategy: String,
    instrument: String,
    broker: String,
    #[serde(default)]
    parameters: serde_json::Value,
    risk_profile: Option<String>,
}

async fn create_bot(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateBotRequest>,
) -> impl IntoResponse {
    let bot_id = uuid::Uuid::new_v4().to_string();
    let bot = crate::state::BotStatus {
        id: bot_id.clone(),
        strategy: req.strategy,
        instrument: req.instrument,
        status: "created".to_string(),
        started_at: None,
    };
    state.active_bots.write().await.insert(bot_id.clone(), bot.clone());

    (StatusCode::CREATED, Json(bot))
}

async fn start_bot(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let mut bots = state.active_bots.write().await;
    if let Some(bot) = bots.get_mut(&id) {
        bot.status = "running".to_string();
        bot.started_at = Some(chrono::Utc::now());
        (StatusCode::OK, Json(serde_json::json!({"status": "started", "id": id})))
    } else {
        (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Bot not found"})))
    }
}

async fn stop_bot(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let mut bots = state.active_bots.write().await;
    if let Some(bot) = bots.get_mut(&id) {
        bot.status = "stopped".to_string();
        (StatusCode::OK, Json(serde_json::json!({"status": "stopped", "id": id})))
    } else {
        (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Bot not found"})))
    }
}

async fn bot_status(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let bots = state.active_bots.read().await;
    if let Some(bot) = bots.get(&id) {
        (StatusCode::OK, Json(serde_json::to_value(bot).unwrap()))
    } else {
        (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Bot not found"})))
    }
}

// ---------------------------------------------------------------------------
// Risk Profiles
// ---------------------------------------------------------------------------

async fn list_risk_profiles() -> impl IntoResponse {
    use propbot_risk::PropFirmProfile;

    let profiles = vec![
        PropFirmProfile::topstep_50k(),
        PropFirmProfile::topstep_100k(),
        PropFirmProfile::topstep_150k(),
        PropFirmProfile::mffu_100k(),
        PropFirmProfile::funding_pips_100k(),
    ];

    Json(profiles)
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct CreateRiskProfileRequest {
    name: String,
    daily_loss_limit: f64,
    max_drawdown: f64,
    trailing_drawdown: bool,
    initial_balance: f64,
    max_position_size: Option<f64>,
    max_contracts: Option<u32>,
}

async fn create_risk_profile(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<CreateRiskProfileRequest>,
) -> impl IntoResponse {
    // TODO: Save to database
    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "status": "created",
            "name": req.name,
        })),
    )
}
