pub mod config;
pub mod claims;
pub mod db;

use axum::{
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
    Json, Router,
    http::{
        HeaderValue, Method,
        header::{AUTHORIZATION, CONTENT_TYPE},
    },
    routing::get,
};
use config::Config;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tower_http::cors::CorsLayer;
use futures::StreamExt;
use tokio_util::codec::{FramedRead, LinesCodec};
use tokio_stream::wrappers::ReceiverStream;
use tracing::{info, warn};
use std::convert::Infallible;
use uuid::Uuid;
use sqlx::Row;


use claims::Claims;

/// Shared application state containing the database connection pool and configuration.
#[derive(Clone, Debug)]
pub struct AppState {
    pub db: Option<sqlx::PgPool>,
    pub config: Config,
}

/// JSON payload structure for the chat endpoint request.
#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub session_id: Option<Uuid>,
    pub prompt: String,
    pub model: String,
}

/// Structured SSE event data pushed when a session is auto-created.
#[derive(Debug, Serialize)]
pub struct SessionCreatedEvent {
    pub session_id: Uuid,
}

/// Structure representing historical message elements returned as context.
#[derive(Debug, Serialize)]
pub struct ContextMessage {
    pub role: String,
    pub content: String,
}

/// SSE token wrapper pushed during completion streaming.
#[derive(Debug, Serialize)]
pub struct TokenEvent {
    pub text: String,
}

/// SSE error details pushed on runtime failures.
#[derive(Debug, Serialize)]
pub struct ErrorEvent {
    pub error: String,
}

/// Performs a basic server status check.
pub async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "message": "Axum server scaffolding is active"
    }))
}

/// Returns secure user dashboard data.
pub async fn get_user_profile(claims: Claims) -> Json<Value> {
    Json(json!({
        "subject": claims.sub,
        "status": "authorized",
        "message": "Access granted to secure user dashboard API"
    }))
}

/// Returns secure protected mock data.
pub async fn get_protected_data(claims: Claims) -> Json<Value> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    Json(json!({
        "message": "Access granted to secure user dashboard API",
        "user_id": claims.sub,
        "timestamp": now
    }))
}

/// Verifies connectivity to the LiteLLM Proxy.
pub async fn check_litellm_connection(litellm_url: &str, litellm_api_key: &str) {
    let client = reqwest::Client::new();
    let url = format!("{}/v1/models", litellm_url.trim_end_matches('/'));
    
    info!("Checking LiteLLM connection at {}...", url);
    let mut request = client.get(&url);
    if !litellm_api_key.is_empty() {
        request = request.header("Authorization", format!("Bearer {}", litellm_api_key));
    }

    match request.send().await {
        Ok(res) => {
            if res.status().is_success() {
                info!("Successfully connected to LiteLLM Proxy at {}", litellm_url);
            } else {
                warn!("LiteLLM Proxy returned non-success status: {} at {}", res.status(), litellm_url);
            }
        }
        Err(e) => {
            warn!("Could not connect to LiteLLM Proxy at {}: {}", litellm_url, e);
        }
    }
}

/// Fetches vector embeddings for input text from the LiteLLM Proxy.
async fn fetch_embedding(
    client: &reqwest::Client,
    litellm_url: &str,
    litellm_api_key: &str,
    input: &str,
    llm_model: &str,
) -> Result<(Vec<f32>, bool), reqwest::Error> {
    // Select the appropriate embedding model based on selected provider
    let (embedding_model, is_1536) = if llm_model.starts_with("gpt") || llm_model.starts_with("o1") {
        ("text-embedding-3-small", true)
    } else {
        ("text-embedding-005", false)
    };

    let url = format!("{}/v1/embeddings", litellm_url.trim_end_matches('/'));
    let payload = json!({
        "model": embedding_model,
        "input": input
    });

    let mut request = client.post(&url).json(&payload);
    if !litellm_api_key.is_empty() {
        request = request.header("Authorization", format!("Bearer {}", litellm_api_key));
    }

    let response = request.send().await?;
    let body: Value = response.json().await?;

    let vector: Vec<f32> = body["data"][0]["embedding"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|val| val.as_f64().map(|v| v as f32))
        .collect();

    Ok((vector, is_1536))
}

/// Handles secure AI chat completion streaming and similarity context retrieval.
pub async fn chat_handler(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<ChatRequest>,
) -> Sse<impl futures::Stream<Item = Result<Event, Infallible>>> {
    let (tx, rx) = tokio::sync::mpsc::channel::<Result<Event, Infallible>>(100);

    let pool = state.db.clone();
    let config = state.config.clone();
    let user_id = claims.sub.clone();

    tokio::spawn(async move {
        let session_id = payload.session_id.unwrap_or_else(Uuid::new_v4);
        let is_new_session = payload.session_id.is_none();

        if let Some(ref pool) = pool {
            if is_new_session {
                if let Err(e) = db::create_session(pool, &user_id, "New Chat").await {
                    let err_evt = ErrorEvent { error: format!("Failed to create chat session: {}", e) };
                    let _ = tx.send(Ok(Event::default().event("error").data(serde_json::to_string(&err_evt).unwrap()))).await;
                    return;
                }
                // Notify client of new session
                let session_evt = SessionCreatedEvent { session_id };
                if tx.send(Ok(Event::default().event("session_created").data(serde_json::to_string(&session_evt).unwrap()))).await.is_err() {
                    return;
                }
            }
        }

        // 1. Fetch prompt embedding
        let client = reqwest::Client::new();
        let embedding_res = fetch_embedding(&client, &config.litellm_url, &config.litellm_api_key, &payload.prompt, &payload.model).await;

        // 2. Fetch context (similarity matches or chronological fallback)
        let mut context_messages = Vec::new();
        let mut used_fallback = false;

        match embedding_res {
            Ok((ref vector, is_1536)) => {
                if let Some(ref pool) = pool {
                    match db::get_semantic_context(pool, &user_id, vector, is_1536, 5).await {
                        Ok(ctx) => context_messages = ctx,
                        Err(_) => used_fallback = true,
                    }
                }
            }
            Err(_) => used_fallback = true,
        }

        if used_fallback {
            if let Some(ref pool) = pool {
                if let Ok(recent) = db::get_recent_messages(pool, session_id, 10).await {
                    context_messages = recent;
                }
            }
        }

        // 3. Stream context messages to the client
        let context_data: Vec<ContextMessage> = context_messages.iter().map(|(role, content)| ContextMessage {
            role: role.clone(),
            content: content.clone(),
        }).collect();

        if tx.send(Ok(Event::default().event("context").data(serde_json::to_string(&context_data).unwrap()))).await.is_err() {
            return;
        }

        // 4. Construct completions payload
        let mut messages = Vec::new();
        messages.push(json!({
            "role": "system",
            "content": "You are a helpful AI assistant. Use the provided relevant context messages to answer the user if applicable."
        }));

        for msg in context_messages {
            messages.push(json!({
                "role": msg.0,
                "content": msg.1
            }));
        }

        messages.push(json!({
            "role": "user",
            "content": payload.prompt
        }));

        let completion_payload = json!({
            "model": payload.model,
            "messages": messages,
            "stream": true
        });

        // 5. Query LiteLLM Completions Stream
        let url = format!("{}/v1/chat/completions", config.litellm_url.trim_end_matches('/'));
        let mut req = client.post(&url).json(&completion_payload);
        if !config.litellm_api_key.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", config.litellm_api_key));
        }

        let res = match req.send().await {
            Ok(r) => r,
            Err(e) => {
                let err_evt = ErrorEvent { error: format!("AI Gateway Connection error: {}", e) };
                let _ = tx.send(Ok(Event::default().event("error").data(serde_json::to_string(&err_evt).unwrap()))).await;
                return;
            }
        };

        // 6. Decode bytes stream and monitor client disconnection aborts
        let body_stream = res.bytes_stream().map(|r| r.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)));
        let reader = tokio_util::io::StreamReader::new(body_stream);
        let mut lines = FramedRead::new(reader, LinesCodec::new());
        let mut assistant_response = String::new();

        loop {
            tokio::select! {
                _ = tx.closed() => {
                    info!("Client disconnected. Aborting upstream LiteLLM streaming connection.");
                    break;
                }
                line = lines.next() => {
                    match line {
                        Some(Ok(text)) => {
                            let text = text.trim();
                            if text.is_empty() {
                                continue;
                            }
                            if text == "data: [DONE]" {
                                let _ = tx.send(Ok(Event::default().event("done").data("{}"))).await;
                                break;
                            }
                            if text.starts_with("data: ") {
                                let json_str = &text["data: ".len()..];
                                if let Ok(parsed) = serde_json::from_str::<Value>(json_str) {
                                    if let Some(content) = parsed["choices"][0]["delta"]["content"].as_str() {
                                        assistant_response.push_str(content);
                                        let token_evt = TokenEvent { text: content.to_string() };
                                        if tx.send(Ok(Event::default().event("token").data(serde_json::to_string(&token_evt).unwrap()))).await.is_err() {
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                        Some(Err(e)) => {
                            let err_evt = ErrorEvent { error: format!("Streaming chunk read error: {}", e) };
                            let _ = tx.send(Ok(Event::default().event("error").data(serde_json::to_string(&err_evt).unwrap()))).await;
                            break;
                        }
                        None => {
                            let _ = tx.send(Ok(Event::default().event("done").data("{}"))).await;
                            break;
                        }
                    }
                }
            }
        }

        // 7. Persist conversation prompt, embedding, and response log to database
        if let Some(ref pool) = pool {
            let count_sql = "SELECT COUNT(*) FROM chat_messages WHERE session_id = $1";
            let count: i64 = sqlx::query(count_sql)
                .bind(session_id)
                .fetch_one(pool)
                .await
                .map(|r| r.get(0))
                .unwrap_or(0);

            let user_idx = count as i32;
            let assistant_idx = user_idx + 1;

            if let Ok(user_msg_id) = db::save_message(pool, session_id, "user", &payload.prompt, user_idx).await {
                if let Ok((ref vector, is_1536)) = embedding_res {
                    let _ = db::save_embedding(pool, user_msg_id, vector, is_1536).await;
                }
            }

            let _ = db::save_message(pool, session_id, "assistant", &assistant_response, assistant_idx).await;
        }
    });

    let stream = ReceiverStream::new(rx);
    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// Creates the Axum application router with configured CORS policies.
pub fn create_app(config: &Config, pool: Option<sqlx::PgPool>) -> Router {
    let state = AppState {
        db: pool,
        config: config.clone(),
    };
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
        .route("/api/user", get(get_user_profile))
        .route("/api/protected", get(get_protected_data))
        .route("/api/chat", axum::routing::post(chat_handler))
        .with_state(state)
        .layer(cors)
}
