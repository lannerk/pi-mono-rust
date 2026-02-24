use axum::extract::ws::{Message, WebSocket, WebSocketStream};
use futures::{SinkExt, StreamExt};
use pi_agent_core::Agent;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum WsMessage {
    #[serde(rename = "chat")]
    Chat { message: String, context_id: Option<String> },
    #[serde(rename = "ping")]
    Ping,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum WsResponse {
    #[serde(rename = "chat")]
    Chat { message: String, context_id: String },
    #[serde(rename = "chat_chunk")]
    ChatChunk { chunk: String, context_id: String },
    #[serde(rename = "error")]
    Error { error: String },
    #[serde(rename = "pong")]
    Pong,
}

pub async fn handle_websocket(socket: WebSocket, agent: Arc<Agent>) {
    let (mut sender, mut receiver) = socket.split();

    let agent_clone = agent.clone();

    let receive_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                        match ws_msg {
                            WsMessage::Chat { message, context_id } => {
                                let context_id = context_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

                                match agent_clone.chat(message).await {
                                    Ok(result) => {
                                        let response_message = if let Some(last_message) = result.messages.last() {
                                            last_message.content.clone()
                                        } else {
                                            "No response".to_string()
                                        };
                                        let response = WsResponse::Chat {
                                            message: response_message,
                                            context_id,
                                        };
                                        if let Ok(json) = serde_json::to_string(&response) {
                                            let _ = sender.send(Message::Text(json)).await;
                                        }
                                    }
                                    Err(e) => {
                                        let response = WsResponse::Error {
                                            error: e.to_string(),
                                        };
                                        if let Ok(json) = serde_json::to_string(&response) {
                                            let _ = sender.send(Message::Text(json)).await;
                                        }
                                    }
                                }
                            }
                            WsMessage::Ping => {
                                let response = WsResponse::Pong;
                                if let Ok(json) = serde_json::to_string(&response) {
                                    let _ = sender.send(Message::Text(json)).await;
                                }
                            }
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    break;
                }
                Err(e) => {
                    tracing::error!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }
    });

    if let Err(e) = receive_task.await {
        tracing::error!("WebSocket task error: {}", e);
    }
}
