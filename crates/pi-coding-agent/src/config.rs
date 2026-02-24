use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub default_model: String,
    pub default_temperature: f32,
    pub default_max_tokens: u32,
    pub api_keys: HashMap<String, String>,
    pub base_urls: HashMap<String, String>,
    pub ui_config: UiConfig,
    pub tool_config: ToolConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub theme: String,
    pub font_size: u32,
    pub show_line_numbers: bool,
    pub auto_save: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
    pub enabled_tools: Vec<String>,
    pub tool_timeout_secs: u64,
    pub max_concurrent_tools: usize,
}

impl Default for Config {
    fn default() -> Self {
        let mut api_keys = HashMap::new();
        if let Ok(key) = std::env::var("OPENAI_API_KEY") {
            api_keys.insert("openai".to_string(), key);
        }
        if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
            api_keys.insert("anthropic".to_string(), key);
        }

        let mut base_urls = HashMap::new();
        base_urls.insert(
            "openai".to_string(),
            "https://api.openai.com/v1".to_string(),
        );
        base_urls.insert(
            "anthropic".to_string(),
            "https://api.anthropic.com/v1".to_string(),
        );

        Self {
            default_model: "gpt-4".to_string(),
            default_temperature: 0.7,
            default_max_tokens: 4096,
            api_keys,
            base_urls,
            ui_config: UiConfig {
                theme: "dark".to_string(),
                font_size: 14,
                show_line_numbers: true,
                auto_save: true,
            },
            tool_config: ToolConfig {
                enabled_tools: vec![
                    "file_read".to_string(),
                    "file_write".to_string(),
                    "file_search".to_string(),
                    "execute_command".to_string(),
                ],
                tool_timeout_secs: 30,
                max_concurrent_tools: 3,
            },
        }
    }
}

impl Config {
    pub fn load(path: Option<&str>) -> Result<Self> {
        if let Some(path) = path {
            let content = std::fs::read_to_string(path)?;
            let config: Config = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            let config_path = Self::default_config_path();
            if config_path.exists() {
                let content = std::fs::read_to_string(&config_path)?;
                let config: Config = serde_json::from_str(&content)?;
                Ok(config)
            } else {
                Ok(Self::default())
            }
        }
    }

    pub fn save(&self, path: Option<&str>) -> Result<()> {
        let config_path = if let Some(path) = path {
            PathBuf::from(path)
        } else {
            Self::default_config_path()
        };

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        Ok(())
    }

    fn default_config_path() -> PathBuf {
        let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push(".pi");
        path.push("config.json");
        path
    }

    pub fn get_api_key(&self, provider: &str) -> Option<&String> {
        self.api_keys.get(provider)
    }

    pub fn get_base_url(&self, provider: &str) -> Option<&String> {
        self.base_urls.get(provider)
    }
}
