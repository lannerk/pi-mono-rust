use crate::pod::{Pod, PodStatus};
use crate::PodConfig;
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

#[derive(Debug, Serialize)]
struct VllmCreateRequest {
    model: String,
    tensor_parallel_size: u32,
    gpu_memory_utilization: f32,
    max_model_len: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct VllmModelInfo {
    id: String,
    object: String,
    created: u64,
    owned_by: String,
}

#[derive(Debug, Deserialize)]
struct VllmHealthResponse {
    status: String,
}

pub struct VllmManager {
    config: PodConfig,
    http_client: HttpClient,
}

impl VllmManager {
    pub fn new(config: PodConfig) -> Self {
        Self {
            config,
            http_client: HttpClient::new(),
        }
    }

    pub async fn deploy_model(&self, pod: &Pod) -> Result<String, anyhow::Error> {
        info!("Deploying model {} for pod {}", pod.model, pod.name);

        let endpoint = self.get_endpoint(pod);
        let request = VllmCreateRequest {
            model: pod.model.clone(),
            tensor_parallel_size: pod.gpu_count,
            gpu_memory_utilization: 0.9,
            max_model_len: Some(4096),
        };

        let response = self
            .http_client
            .post(&format!("{}/v1/models", endpoint))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Failed to deploy model: {}", error_text);
        }

        info!("Model {} deployed successfully for pod {}", pod.model, pod.name);
        Ok(endpoint)
    }

    pub async fn check_health(&self, pod: &Pod) -> Result<bool, anyhow::Error> {
        let endpoint = self.get_endpoint(pod);
        let url = format!("{}/health", endpoint);

        debug!("Checking health for pod {} at {}", pod.name, url);

        let response = self.http_client.get(&url).send().await?;

        if response.status().is_success() {
            let health: VllmHealthResponse = response.json().await?;
            Ok(health.status == "ok")
        } else {
            Ok(false)
        }
    }

    pub async fn list_models(&self, pod: &Pod) -> Result<Vec<VllmModelInfo>, anyhow::Error> {
        let endpoint = self.get_endpoint(pod);
        let url = format!("{}/v1/models", endpoint);

        let response = self.http_client.get(&url).send().await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to list models: {}", response.status());
        }

        let json: serde_json::Value = response.json().await?;
        let models: Vec<VllmModelInfo> = serde_json::from_value(json["data"].clone())?;

        Ok(models)
    }

    pub async fn remove_model(&self, pod: &Pod) -> Result<(), anyhow::Error> {
        info!("Removing model {} for pod {}", pod.model, pod.name);

        let endpoint = self.get_endpoint(pod);
        let url = format!("{}/v1/models/{}", endpoint, pod.model);

        let response = self.http_client.delete(&url).send().await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to remove model: {}", response.status());
        }

        Ok(())
    }

    pub async fn get_metrics(&self, pod: &Pod) -> Result<serde_json::Value, anyhow::Error> {
        let endpoint = self.get_endpoint(pod);
        let url = format!("{}/metrics", endpoint);

        let response = self.http_client.get(&url).send().await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to get metrics: {}", response.status());
        }

        let metrics: serde_json::Value = response.json().await?;
        Ok(metrics)
    }

    fn get_endpoint(&self, pod: &Pod) -> String {
        if let Some(endpoint) = &pod.endpoint {
            endpoint.clone()
        } else {
            format!("http://{}.{}.svc.cluster.local:8000", pod.name, pod.namespace)
        }
    }
}
