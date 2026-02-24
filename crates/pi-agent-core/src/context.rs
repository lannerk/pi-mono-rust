use pi_ai::{Message, MessageRole};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    pub id: String,
    pub messages: VecDeque<Message>,
    pub max_messages: usize,
    pub metadata: ContextMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMetadata {
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub session_id: Option<String>,
    pub user_id: Option<String>,
    pub tags: Vec<String>,
}

impl Default for ContextMetadata {
    fn default() -> Self {
        let now = chrono::Utc::now();
        Self {
            created_at: now,
            updated_at: now,
            session_id: None,
            user_id: None,
            tags: Vec::new(),
        }
    }
}

impl Context {
    pub fn new(id: String) -> Self {
        Self {
            id,
            messages: VecDeque::new(),
            max_messages: 100,
            metadata: ContextMetadata::default(),
        }
    }

    pub fn with_max_messages(mut self, max: usize) -> Self {
        self.max_messages = max;
        self
    }

    pub fn with_session_id(mut self, session_id: String) -> Self {
        self.metadata.session_id = Some(session_id);
        self
    }

    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.metadata.user_id = Some(user_id);
        self
    }

    pub fn add_message(&mut self, message: Message) {
        self.messages.push_back(message);
        self.trim();
        self.metadata.updated_at = chrono::Utc::now();
    }

    pub fn add_system_message(&mut self, content: String) {
        self.add_message(Message::system(content));
    }

    pub fn add_user_message(&mut self, content: String) {
        self.add_message(Message::user(content));
    }

    pub fn add_assistant_message(&mut self, content: String) {
        self.add_message(Message::assistant(content));
    }

    pub fn add_tool_message(&mut self, content: String, tool_call_id: String) {
        self.add_message(Message::tool(content, tool_call_id));
    }

    pub fn get_messages(&self) -> Vec<Message> {
        self.messages.iter().cloned().collect()
    }

    pub fn get_last_n_messages(&self, n: usize) -> Vec<Message> {
        self.messages
            .iter()
            .rev()
            .take(n)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    pub fn clear(&mut self) {
        self.messages.clear();
        self.metadata.updated_at = chrono::Utc::now();
    }

    pub fn reset(&mut self, system_message: Option<String>) {
        self.clear();
        if let Some(msg) = system_message {
            self.add_system_message(msg);
        }
    }

    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    pub fn token_estimate(&self) -> usize {
        self.messages
            .iter()
            .map(|m| m.content.len() / 4)
            .sum()
    }

    fn trim(&mut self) {
        while self.messages.len() > self.max_messages {
            if let Some(role) = self.messages.front().map(|m| m.role.clone()) {
                if role == MessageRole::System {
                    self.messages.pop_front();
                    continue;
                }
            }
            self.messages.pop_front();
        }
    }

    pub fn add_tag(&mut self, tag: String) {
        if !self.metadata.tags.contains(&tag) {
            self.metadata.tags.push(tag);
        }
    }

    pub fn remove_tag(&mut self, tag: &str) {
        self.metadata.tags.retain(|t| t != tag);
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        self.metadata.tags.contains(&tag.to_string())
    }
}

#[derive(Clone)]
pub struct ContextManager {
    contexts: Arc<RwLock<std::collections::HashMap<String, Context>>>,
}

impl ContextManager {
    pub fn new() -> Self {
        Self {
            contexts: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    pub async fn create(&self, id: String) -> Context {
        let context = Context::new(id.clone());
        let mut contexts = self.contexts.write().await;
        contexts.insert(id, context.clone());
        context
    }

    pub async fn get(&self, id: &str) -> Option<Context> {
        let contexts = self.contexts.read().await;
        contexts.get(id).cloned()
    }

    pub async fn update<F>(&self, id: &str, f: F) -> AgentResult<()>
    where
        F: FnOnce(&mut Context),
    {
        let mut contexts = self.contexts.write().await;
        let context = contexts
            .get_mut(id)
            .ok_or_else(|| AgentError::Context(format!("Context {} not found", id)))?;
        f(context);
        Ok(())
    }

    pub async fn delete(&self, id: &str) {
        let mut contexts = self.contexts.write().await;
        contexts.remove(id);
    }

    pub async fn list(&self) -> Vec<Context> {
        let contexts = self.contexts.read().await;
        contexts.values().cloned().collect()
    }

    pub async fn clear(&self) {
        let mut contexts = self.contexts.write().await;
        contexts.clear();
    }
}

impl Default for ContextManager {
    fn default() -> Self {
        Self::new()
    }
}

use crate::error::{AgentError, AgentResult};
