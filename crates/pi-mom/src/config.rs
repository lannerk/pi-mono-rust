use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackConfig {
    pub bot_token: String,
    pub app_token: String,
    pub signing_secret: String,
    pub channels: Vec<String>,
    pub allowed_users: Option<Vec<String>>,
}

impl SlackConfig {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            bot_token: std::env::var("SLACK_BOT_TOKEN")?,
            app_token: std::env::var("SLACK_APP_TOKEN")?,
            signing_secret: std::env::var("SLACK_SIGNING_SECRET")?,
            channels: std::env::var("SLACK_CHANNELS")
                .unwrap_or_default()
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            allowed_users: std::env::var("SLACK_ALLOWED_USERS")
                .ok()
                .map(|s| s.split(',').map(|s| s.trim().to_string()).collect()),
        })
    }
}
