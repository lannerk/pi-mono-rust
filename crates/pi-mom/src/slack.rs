use crate::config::SlackConfig;
use pi_agent_core::Agent;
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};

#[derive(Debug, Serialize)]
struct SlackMessage {
    channel: String,
    text: String,
    thread_ts: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SlackEvent {
    #[serde(rename = "type")]
    event_type: String,
    user: String,
    text: String,
    channel: String,
    ts: String,
    thread_ts: Option<String>,
}

pub struct SlackBot {
    config: SlackConfig,
    agent: Arc<Agent>,
    http_client: HttpClient,
}

impl SlackBot {
    pub fn new(config: SlackConfig, agent: Arc<Agent>) -> Self {
        Self {
            config,
            agent,
            http_client: HttpClient::new(),
        }
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        info!("Starting Slack bot");

        self.verify_bot_access().await?;

        info!("Slack bot started successfully");
        info!("Listening on channels: {:?}", self.config.channels);

        Ok(())
    }

    async fn verify_bot_access(&self) -> anyhow::Result<()> {
        let url = "https://slack.com/api/auth.test";
        let response = self
            .http_client
            .get(url)
            .header("Authorization", format!("Bearer {}", self.config.bot_token))
            .send()
            .await?;

        let json: serde_json::Value = response.json().await?;
        if !json.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
            anyhow::bail!("Failed to verify bot access: {:?}", json);
        }

        Ok(())
    }

    async fn handle_message(&self, event: SlackEvent) -> anyhow::Result<()> {
        if !self.config.channels.contains(&event.channel) {
            return Ok(());
        }

        if let Some(allowed_users) = &self.config.allowed_users {
            if !allowed_users.contains(&event.user) {
                return Ok(());
            }
        }

        info!("Received message from {}: {}", event.user, event.text);

        match self.agent.chat(event.text.clone()).await {
            Ok(result) => {
                let response = if let Some(last_message) = result.messages.last() {
                    last_message.content.clone()
                } else {
                    "No response".to_string()
                };
                self.send_message(
                    event.channel,
                    response,
                    event.thread_ts,
                )
                .await?;
            }
            Err(e) => {
                error!("Failed to process message: {}", e);
                self.send_message(
                    event.channel,
                    format!("Error: {}", e),
                    event.thread_ts,
                )
                .await?;
            }
        }

        Ok(())
    }

    async fn send_message(
        &self,
        channel: String,
        text: String,
        thread_ts: Option<String>,
    ) -> anyhow::Result<()> {
        let url = "https://slack.com/api/chat.postMessage";
        let message = SlackMessage {
            channel,
            text,
            thread_ts,
        };

        let response = self
            .http_client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.config.bot_token))
            .json(&message)
            .send()
            .await?;

        let json: serde_json::Value = response.json().await?;
        if !json.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
            error!("Failed to send message: {:?}", json);
        }

        Ok(())
    }
}
