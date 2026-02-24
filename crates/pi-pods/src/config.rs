use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodConfig {
    pub default_gpu_type: String,
    pub default_gpu_count: u32,
    pub default_replicas: u32,
    pub namespace: String,
    pub registry: String,
    pub vllm_image: String,
    pub api_server: String,
    pub api_token: String,
}

impl Default for PodConfig {
    fn default() -> Self {
        Self {
            default_gpu_type: "nvidia-tesla-v100".to_string(),
            default_gpu_count: 1,
            default_replicas: 1,
            namespace: "pi-pods".to_string(),
            registry: "ghcr.io".to_string(),
            vllm_image: "vllm/vllm-openai:latest".to_string(),
            api_server: std::env::var("KUBERNETES_API_SERVER")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            api_token: std::env::var("KUBERNETES_API_TOKEN")
                .unwrap_or_default(),
        }
    }
}

impl PodConfig {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self::default())
    }
}
