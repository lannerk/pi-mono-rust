use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    pub id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub data: HashMap<String, serde_json::Value>,
}

impl AgentState {
    pub fn new(id: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            id,
            created_at: now,
            updated_at: now,
            data: HashMap::new(),
        }
    }

    pub fn get<T>(&self, key: &str) -> Option<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.data.get(key).and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    pub fn set<T>(&mut self, key: String, value: T)
    where
        T: Serialize,
    {
        self.data.insert(key, serde_json::to_value(value).unwrap());
        self.updated_at = chrono::Utc::now();
    }

    pub fn remove(&mut self, key: &str) {
        self.data.remove(key);
        self.updated_at = chrono::Utc::now();
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }
}

#[derive(Clone)]
pub struct StateStore {
    states: Arc<RwLock<HashMap<String, AgentState>>>,
}

impl StateStore {
    pub fn new() -> Self {
        Self {
            states: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get(&self, id: &str) -> Option<AgentState> {
        let states = self.states.read().await;
        states.get(id).cloned()
    }

    pub async fn set(&self, state: AgentState) {
        let mut states = self.states.write().await;
        states.insert(state.id.clone(), state);
    }

    pub async fn remove(&self, id: &str) {
        let mut states = self.states.write().await;
        states.remove(id);
    }

    pub async fn list(&self) -> Vec<AgentState> {
        let states = self.states.read().await;
        states.values().cloned().collect()
    }

    pub async fn clear(&self) {
        let mut states = self.states.write().await;
        states.clear();
    }
}

impl Default for StateStore {
    fn default() -> Self {
        Self::new()
    }
}
