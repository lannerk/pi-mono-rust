use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PodStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Unknown,
}

impl std::fmt::Display for PodStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PodStatus::Pending => write!(f, "Pending"),
            PodStatus::Running => write!(f, "Running"),
            PodStatus::Succeeded => write!(f, "Succeeded"),
            PodStatus::Failed => write!(f, "Failed"),
            PodStatus::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pod {
    pub name: String,
    pub namespace: String,
    pub model: String,
    pub gpu_type: String,
    pub gpu_count: u32,
    pub replicas: u32,
    pub status: PodStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub endpoint: Option<String>,
    pub labels: HashMap<String, String>,
    pub annotations: HashMap<String, String>,
}

impl Pod {
    pub fn new(
        name: String,
        namespace: String,
        model: String,
        gpu_type: String,
        gpu_count: u32,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            name,
            namespace,
            model,
            gpu_type,
            gpu_count,
            replicas: 1,
            status: PodStatus::Pending,
            created_at: now,
            updated_at: now,
            endpoint: None,
            labels: HashMap::new(),
            annotations: HashMap::new(),
        }
    }

    pub fn with_replicas(mut self, replicas: u32) -> Self {
        self.replicas = replicas;
        self
    }

    pub fn with_label(mut self, key: String, value: String) -> Self {
        self.labels.insert(key, value);
        self
    }

    pub fn with_annotation(mut self, key: String, value: String) -> Self {
        self.annotations.insert(key, value);
        self
    }

    pub fn update_status(&mut self, status: PodStatus) {
        self.status = status;
        self.updated_at = chrono::Utc::now();
    }

    pub fn set_endpoint(&mut self, endpoint: String) {
        self.endpoint = Some(endpoint);
        self.updated_at = chrono::Utc::now();
    }

    pub fn age(&self) -> chrono::Duration {
        chrono::Utc::now() - self.created_at
    }

    pub fn is_running(&self) -> bool {
        self.status == PodStatus::Running
    }

    pub fn is_healthy(&self) -> bool {
        self.is_running() && self.endpoint.is_some()
    }
}

pub struct PodManager {
    pub config: super::PodConfig,
    pods: HashMap<String, Pod>,
}

impl PodManager {
    pub fn new(config: super::PodConfig) -> Self {
        Self {
            pods: HashMap::new(),
            config,
        }
    }

    pub async fn create(&mut self, pod: Pod) -> Result<(), anyhow::Error> {
        if self.pods.contains_key(&pod.name) {
            anyhow::bail!("Pod {} already exists", pod.name);
        }

        self.pods.insert(pod.name.clone(), pod);
        Ok(())
    }

    pub async fn delete(&mut self, name: &str) -> Result<Option<Pod>, anyhow::Error> {
        Ok(self.pods.remove(name))
    }

    pub async fn get(&self, name: &str) -> Option<&Pod> {
        self.pods.get(name)
    }

    pub async fn get_mut(&mut self, name: &str) -> Option<&mut Pod> {
        self.pods.get_mut(name)
    }

    pub async fn list(&self, all: bool) -> Vec<&Pod> {
        let mut pods: Vec<_> = self.pods.values().collect();
        pods.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        if !all {
            pods.retain(|p| p.status != PodStatus::Succeeded);
        }

        pods
    }

    pub async fn start(&mut self, name: &str) -> Result<(), anyhow::Error> {
        let pod = self
            .pods
            .get_mut(name)
            .ok_or_else(|| anyhow::anyhow!("Pod {} not found", name))?;

        pod.update_status(PodStatus::Running);
        Ok(())
    }

    pub async fn stop(&mut self, name: &str) -> Result<(), anyhow::Error> {
        let pod = self
            .pods
            .get_mut(name)
            .ok_or_else(|| anyhow::anyhow!("Pod {} not found", name))?;

        pod.update_status(PodStatus::Succeeded);
        Ok(())
    }

    pub async fn scale(&mut self, name: &str, replicas: u32) -> Result<(), anyhow::Error> {
        let pod = self
            .pods
            .get_mut(name)
            .ok_or_else(|| anyhow::anyhow!("Pod {} not found", name))?;

        pod.replicas = replicas;
        pod.updated_at = chrono::Utc::now();
        Ok(())
    }

    pub async fn update(&mut self, name: &str, updates: PodUpdate) -> Result<(), anyhow::Error> {
        let pod = self
            .pods
            .get_mut(name)
            .ok_or_else(|| anyhow::anyhow!("Pod {} not found", name))?;

        if let Some(model) = updates.model {
            pod.model = model;
        }

        if let Some(gpu_count) = updates.gpu_count {
            pod.gpu_count = gpu_count;
        }

        pod.updated_at = chrono::Utc::now();
        Ok(())
    }

    pub async fn status(&self, name: &str) -> Result<&Pod, anyhow::Error> {
        self.pods
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Pod {} not found", name))
    }

    pub async fn logs(&self, name: &str) -> Result<String, anyhow::Error> {
        let _pod = self
            .pods
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Pod {} not found", name))?;

        Ok("No logs available".to_string())
    }
}

#[derive(Debug, Clone)]
pub struct PodUpdate {
    pub model: Option<String>,
    pub gpu_count: Option<u32>,
}
