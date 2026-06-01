pub mod config;
pub mod claims;

use axum::{
    Json, Router,
    http::{
        HeaderValue, Method,
        header::{AUTHORIZATION, CONTENT_TYPE},
    },
    routing::get,
};
use config::Config;
use serde_json::{Value, json};
use tower_http::cors::CorsLayer;

/// Performs a basic server status check.
pub async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "message": "Axum server scaffolding is active"
    }))
}

/// Creates the Axum application router with configured CORS policies.
pub fn create_app(config: &Config) -> Router {
    let mut cors = CorsLayer::new();

    // Wire up allowed origins
    if config.allowed_origins.iter().any(|o| o == "*") {
        cors = cors.allow_origin(tower_http::cors::Any);
    } else {
        let origins: Vec<HeaderValue> = config
            .allowed_origins
            .iter()
            .filter_map(|o| o.parse::<HeaderValue>().ok())
            .collect();
        cors = cors.allow_origin(origins);
    }

    cors = cors
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE]);

    Router::new()
        .route("/api/health", get(health_check))
        .layer(cors)
}
