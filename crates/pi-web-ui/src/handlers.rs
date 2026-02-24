use axum::{
    extract::{State, WebSocketUpgrade},
    response::{IntoResponse, Json},
};
use pi_agent_core::Agent;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

pub async fn health_handler() -> impl IntoResponse {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub message: String,
    pub context_id: Option<String>,
    pub stream: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub message: String,
    pub context_id: String,
    pub tokens_used: u32,
}

pub async fn chat_handler(
    State(agent): State<Arc<Agent>>,
    Json(request): Json<ChatRequest>,
) -> impl IntoResponse {
    let context_id = request.context_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    match agent.chat_with_context(context_id.clone(), request.message).await {
        Ok(result) => {
            let message = if let Some(last_message) = result.messages.last() {
                last_message.content.clone()
            } else {
                "No response".to_string()
            };
            Json(ChatResponse {
                message,
                context_id,
                tokens_used: 0,
            })
        }
        Err(e) => {
            tracing::error!("Chat error: {}", e);
            Json(ChatResponse {
                message: format!("Error: {}", e),
                context_id,
                tokens_used: 0,
            })
        }
    }
}

pub async fn ws_handler(
    State(agent): State<Arc<Agent>>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| crate::websocket::handle_websocket(socket, agent))
}
