use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub providers: HashMap<String, ProviderConfig>,
    pub default_provider: String,
    pub timeout_secs: u64,
    pub max_retries: u32,
}

impl Default for Config {
    fn default() -> Self {
        let mut providers = HashMap::new();
        providers.insert(
            "openai".to_string(),
            ProviderConfig {
                api_key: std::env::var("OPENAI_API_KEY").unwrap_or_default(),
                base_url: "https://api.openai.com/v1".to_string(),
                model: "gpt-4".to_string(),
                ..Default::default()
            },
        );

        Self {
            providers,
            default_provider: "openai".to_string(),
            timeout_secs: 120,
            max_retries: 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub organization: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub headers: HashMap<String, String>,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: String::new(),
            model: String::new(),
            organization: None,
            max_tokens: None,
            temperature: None,
            top_p: None,
            headers: HashMap::new(),
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_provider(mut self, name: String, config: ProviderConfig) -> Self {
        self.providers.insert(name, config);
        self
    }

    pub fn with_default_provider(mut self, provider: String) -> Self {
        self.default_provider = provider;
        self
    }

    pub fn get_provider(&self, name: &str) -> Option<&ProviderConfig> {
        self.providers.get(name)
    }

    pub fn from_env() -> Result<Self, crate::Error> {
        let mut config = Self::default();

        if let Ok(openai_key) = std::env::var("OPENAI_API_KEY") {
            config.providers.insert(
                "openai".to_string(),
                ProviderConfig {
                    api_key: openai_key,
                    base_url: std::env::var("OPENAI_BASE_URL")
                        .unwrap_or_else(|_| "https://api.openai.com/v1".to_string()),
                    model: std::env::var("OPENAI_MODEL")
                        .unwrap_or_else(|_| "gpt-4".to_string()),
                    ..Default::default()
                },
            );
        }

        if let Ok(anthropic_key) = std::env::var("ANTHROPIC_API_KEY") {
            config.providers.insert(
                "anthropic".to_string(),
                ProviderConfig {
                    api_key: anthropic_key,
                    base_url: "https://api.anthropic.com/v1".to_string(),
                    model: "claude-3-opus-20240229".to_string(),
                    ..Default::default()
                },
            );
        }

        if let Ok(google_key) = std::env::var("GOOGLE_API_KEY") {
            config.providers.insert(
                "google".to_string(),
                ProviderConfig {
                    api_key: google_key,
                    base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
                    model: "gemini-pro".to_string(),
                    ..Default::default()
                },
            );
        }

        Ok(config)
    }
}
