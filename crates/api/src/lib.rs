pub mod routes;
pub mod state;

use axum::Router;
use sqlx::PgPool;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

/// Build the Axum application router.
pub fn build_router(pool: PgPool) -> Router {
    let app_state = Arc::new(state::AppState::new(pool));

    Router::new()
        .nest("/api", routes::api_routes())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(app_state)
}

/// Start the API server.
pub async fn start_server(pool: PgPool, bind_addr: &str) -> anyhow::Result<()> {
    let app = build_router(pool);
    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    tracing::info!("API server listening on {}", bind_addr);
    axum::serve(listener, app).await?;
    Ok(())
}
