use crate::context::{Context, ContextManager};
use crate::error::{AgentError, AgentResult};
use crate::executor::{Executor, ExecutorConfig, ExecutionResult};
use crate::state::{AgentState, StateStore};
use crate::tool_registry::ToolRegistry;
use pi_ai::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use futures::StreamExt;
use std::pin::Pin;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub system_prompt: String,
    pub model: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub executor_config: ExecutorConfig,
    pub enabled_tools: Vec<String>,
}

impl AgentConfig {
    pub fn new(id: String, name: String, system_prompt: String) -> Self {
        Self {
            id,
            name,
            description: None,
            system_prompt,
            model: "gpt-4".to_string(),
            temperature: None,
            max_tokens: None,
            executor_config: ExecutorConfig::default(),
            enabled_tools: Vec::new(),
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_model(mut self, model: String) -> Self {
        self.model = model;
        self
    }

    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    pub fn with_executor_config(mut self, config: ExecutorConfig) -> Self {
        self.executor_config = config;
        self
    }

    pub fn with_enabled_tools(mut self, tools: Vec<String>) -> Self {
        self.enabled_tools = tools;
        self
    }
}

#[derive(Clone)]
pub struct Agent {
    config: AgentConfig,
    executor: Arc<Executor>,
    context_manager: Arc<ContextManager>,
    state_store: Arc<StateStore>,
    tool_registry: Arc<ToolRegistry>,
    initialized: Arc<RwLock<bool>>,
}

impl Agent {
    pub fn new(
        config: AgentConfig,
        llm_client: Arc<Client>,
        tool_registry: Arc<ToolRegistry>,
    ) -> Self {
        let state_store = Arc::new(StateStore::new());
        let context_manager = Arc::new(ContextManager::new());
        let executor = Arc::new(Executor::new(
            config.executor_config.clone(),
            llm_client,
            tool_registry.clone(),
            state_store.clone(),
        ));

        Self {
            config,
            executor,
            context_manager,
            state_store,
            tool_registry,
            initialized: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn initialize(&self) -> AgentResult<()> {
        let mut initialized = self.initialized.write().await;
        *initialized = true;
        Ok(())
    }

    pub async fn is_initialized(&self) -> bool {
        *self.initialized.read().await
    }

    pub async fn chat(&self, user_message: String) -> AgentResult<ExecutionResult> {
        self.ensure_initialized().await?;

        let context_id = format!("{}:default", self.config.id);
        let mut context = self
            .context_manager
            .get(&context_id)
            .await
            .unwrap_or_else(|| Context::new(context_id.clone()));

        if context.message_count() == 0 {
            context.add_system_message(self.config.system_prompt.clone());
        }

        context.add_user_message(user_message);

        let result = self.executor.execute(&mut context).await?;

        self.context_manager.update(&context_id, |ctx| {
            *ctx = context;
        }).await?;

        Ok(result)
    }

    pub async fn chat_with_context(
        &self,
        context_id: String,
        user_message: String,
    ) -> AgentResult<ExecutionResult> {
        self.ensure_initialized().await?;

        let mut context = self
            .context_manager
            .get(&context_id)
            .await
            .unwrap_or_else(|| Context::new(context_id.clone()));

        if context.message_count() == 0 {
            context.add_system_message(self.config.system_prompt.clone());
        }

        context.add_user_message(user_message);

        let result = self.executor.execute(&mut context).await?;

        self.context_manager.update(&context_id, |ctx| {
            *ctx = context;
        }).await?;

        Ok(result)
    }

    pub async fn chat_stream(
        &self,
        user_message: String,
    ) -> AgentResult<Pin<Box<dyn futures::Stream<Item = AgentResult<String>> + Send>>> {
        self.ensure_initialized().await?;

        let context_id = format!("{}:default", self.config.id);
        let mut context = self
            .context_manager
            .get(&context_id)
            .await
            .unwrap_or_else(|| Context::new(context_id.clone()));

        if context.message_count() == 0 {
            context.add_system_message(self.config.system_prompt.clone());
        }

        context.add_user_message(user_message);

        let stream = self.executor.execute_stream(&mut context).await?;
        let stream = stream.map(|result| {
            result.map(|event| match event {
                crate::executor::StreamEvent::Token(token) => token,
                crate::executor::StreamEvent::ToolCall { id, name, arguments } => {
                    format!("[ToolCall: {}({}) args={}]", name, id, arguments)
                }
                crate::executor::StreamEvent::Done => "[Done]".to_string(),
                crate::executor::StreamEvent::Error(err) => format!("[Error: {}]", err),
            })
        });

        Ok(Box::pin(stream))
    }

    pub async fn create_context(&self, context_id: String) -> AgentResult<Context> {
        let context = self.context_manager.create(context_id).await;
        Ok(context)
    }

    pub async fn get_context(&self, context_id: &str) -> AgentResult<Option<Context>> {
        Ok(self.context_manager.get(context_id).await)
    }

    pub async fn delete_context(&self, context_id: &str) -> AgentResult<()> {
        self.context_manager.delete(context_id).await;
        Ok(())
    }

    pub async fn list_contexts(&self) -> AgentResult<Vec<Context>> {
        Ok(self.context_manager.list().await)
    }

    pub async fn get_state(&self) -> AgentResult<Option<AgentState>> {
        Ok(self.state_store.get(&self.config.id).await)
    }

    pub async fn set_state(&self, state: AgentState) -> AgentResult<()> {
        self.state_store.set(state).await;
        Ok(())
    }

    pub async fn update_state<F>(&self, f: F) -> AgentResult<()>
    where
        F: FnOnce(&mut AgentState),
    {
        let mut state = self
            .state_store
            .get(&self.config.id)
            .await
            .unwrap_or_else(|| AgentState::new(self.config.id.clone()));
        f(&mut state);
        self.state_store.set(state).await;
        Ok(())
    }

    pub async fn register_tool(&self, tool: crate::Tool) -> AgentResult<()> {
        self.tool_registry.register(tool).await
    }

    pub async fn unregister_tool(&self, name: &str) -> AgentResult<()> {
        self.tool_registry.unregister(name).await
    }

    pub async fn list_tools(&self) -> AgentResult<Vec<String>> {
        Ok(self.tool_registry.list().await)
    }

    pub async fn reset(&self) -> AgentResult<()> {
        let context_id = format!("{}:default", self.config.id);
        self.context_manager.delete(&context_id).await;
        self.state_store.remove(&self.config.id).await;
        Ok(())
    }

    pub fn config(&self) -> &AgentConfig {
        &self.config
    }

    pub fn executor(&self) -> &Arc<Executor> {
        &self.executor
    }

    pub fn tool_registry(&self) -> &Arc<ToolRegistry> {
        &self.tool_registry
    }

    async fn ensure_initialized(&self) -> AgentResult<()> {
        if !self.is_initialized().await {
            return Err(AgentError::NotInitialized);
        }
        Ok(())
    }
}
