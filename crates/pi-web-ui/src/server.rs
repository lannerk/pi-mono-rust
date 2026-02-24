use axum::{
    extract::{State, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use pi_agent_core::Agent;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing::info;

use crate::handlers::{chat_handler, health_handler, ws_handler};

pub struct Server {
    agent: Arc<Agent>,
    port: u16,
}

impl Server {
    pub fn new(agent: Arc<Agent>, port: u16) -> Self {
        Self { agent, port }
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let app = self.create_router();

        let addr = format!("0.0.0.0:{}", self.port);
        info!("Starting web UI server on {}", addr);

        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }

    fn create_router(&self) -> Router {
        Router::new()
            .route("/health", get(health_handler))
            .route("/api/chat", get(chat_handler))
            .route("/ws", get(ws_handler))
            .with_state(self.agent.clone())
            .layer(CorsLayer::permissive())
    }
}
