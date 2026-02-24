use crate::tool::{Tool, ToolHandler};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct ToolRegistry {
    tools: Arc<RwLock<HashMap<String, Tool>>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register(&self, tool: Tool) -> Result<(), crate::AgentError> {
        let name = tool.name().to_string();
        let mut tools = self.tools.write().await;
        tools.insert(name, tool);
        Ok(())
    }

    pub async fn unregister(&self, name: &str) -> Result<(), crate::AgentError> {
        let mut tools = self.tools.write().await;
        if tools.remove(name).is_none() {
            return Err(crate::AgentError::ToolNotFound(name.to_string()));
        }
        Ok(())
    }

    pub async fn get(&self, name: &str) -> Option<Tool> {
        let tools = self.tools.read().await;
        tools.get(name).cloned()
    }

    pub async fn list(&self) -> Vec<String> {
        let tools = self.tools.read().await;
        tools.keys().cloned().collect()
    }

    pub async fn get_tool_definitions(&self) -> Vec<pi_ai::models::ToolDefinition> {
        let tools = self.tools.read().await;
        tools.values().map(|t| {
            let def = t.to_definition();
            pi_ai::models::ToolDefinition {
                tool_type: "function".to_string(),
                function: pi_ai::models::FunctionDefinition {
                    name: def.name,
                    description: Some(def.description),
                    parameters: serde_json::to_value(&def.parameters).unwrap_or_default(),
                },
            }
        }).collect()
    }

    pub async fn execute(
        &self,
        name: &str,
        arguments: serde_json::Value,
    ) -> Result<crate::tool::ToolExecutionResult, crate::AgentError> {
        let tools = self.tools.read().await;
        let tool = tools
            .get(name)
            .ok_or_else(|| crate::AgentError::ToolNotFound(name.to_string()))?;
        tool.execute(arguments).await
    }

    pub async fn clear(&self) {
        let mut tools = self.tools.write().await;
        tools.clear();
    }

    pub async fn count(&self) -> usize {
        let tools = self.tools.read().await;
        tools.len()
    }

    pub async fn contains(&self, name: &str) -> bool {
        let tools = self.tools.read().await;
        tools.contains_key(name)
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
