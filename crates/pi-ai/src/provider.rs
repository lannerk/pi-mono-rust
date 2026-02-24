use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProviderType {
    OpenAI,
    Anthropic,
    Google,
    Azure,
    Custom,
}

impl From<&str> for ProviderType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "openai" => ProviderType::OpenAI,
            "anthropic" => ProviderType::Anthropic,
            "google" => ProviderType::Google,
            "azure" => ProviderType::Azure,
            _ => ProviderType::Custom,
        }
    }
}

impl std::fmt::Display for ProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderType::OpenAI => write!(f, "openai"),
            ProviderType::Anthropic => write!(f, "anthropic"),
            ProviderType::Google => write!(f, "google"),
            ProviderType::Azure => write!(f, "azure"),
            ProviderType::Custom => write!(f, "custom"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Provider {
    pub name: String,
    pub provider_type: ProviderType,
    pub config: super::ProviderConfig,
}

impl Provider {
    pub fn new(name: impl Into<String>, provider_type: ProviderType, config: super::ProviderConfig) -> Self {
        Self {
            name: name.into(),
            provider_type,
            config,
        }
    }

    pub fn get_endpoint(&self, path: &str) -> String {
        format!("{}/{}", self.config.base_url.trim_end_matches('/'), path.trim_start_matches('/'))
    }

    pub fn get_headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        
        match self.provider_type {
            ProviderType::OpenAI => {
                headers.insert("Authorization".to_string(), format!("Bearer {}", self.config.api_key));
                if let Some(org) = &self.config.organization {
                    headers.insert("OpenAI-Organization".to_string(), org.clone());
                }
            }
            ProviderType::Anthropic => {
                headers.insert("x-api-key".to_string(), self.config.api_key.clone());
                headers.insert("anthropic-version".to_string(), "2023-06-01".to_string());
            }
            ProviderType::Google => {
                headers.insert("x-goog-api-key".to_string(), self.config.api_key.clone());
            }
            ProviderType::Azure => {
                headers.insert("api-key".to_string(), self.config.api_key.clone());
            }
            ProviderType::Custom => {
                headers.insert("Authorization".to_string(), format!("Bearer {}", self.config.api_key));
            }
        }

        for (key, value) in &self.config.headers {
            headers.insert(key.clone(), value.clone());
        }

        headers
    }

    pub fn supports_streaming(&self) -> bool {
        matches!(
            self.provider_type,
            ProviderType::OpenAI | ProviderType::Anthropic | ProviderType::Google
        )
    }

    pub fn supports_tools(&self) -> bool {
        matches!(
            self.provider_type,
            ProviderType::OpenAI | ProviderType::Anthropic
        )
    }
}
