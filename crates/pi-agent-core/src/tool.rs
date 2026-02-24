use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: ToolParameters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameters {
    #[serde(rename = "type")]
    pub param_type: String,
    pub properties: HashMap<String, Property>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Property {
    #[serde(rename = "type")]
    pub prop_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub metadata: HashMap<String, Value>,
}

impl ToolExecutionResult {
    pub fn success(output: impl Into<String>) -> Self {
        Self {
            success: true,
            output: output.into(),
            error: None,
            metadata: HashMap::new(),
        }
    }

    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            success: false,
            output: String::new(),
            error: Some(error.into()),
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: String, value: Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

#[async_trait]
pub trait ToolHandler: Send + Sync {
    fn name(&self) -> &str;

    fn description(&self) -> &str;

    fn parameters(&self) -> ToolParameters;

    async fn execute(&self, arguments: Value) -> Result<ToolExecutionResult, crate::AgentError>;
}

#[derive(Clone)]
pub struct Tool {
    handler: Arc<dyn ToolHandler>,
}

impl Tool {
    pub fn new(handler: Arc<dyn ToolHandler>) -> Self {
        Self { handler }
    }

    pub fn name(&self) -> &str {
        self.handler.name()
    }

    pub fn description(&self) -> &str {
        self.handler.description()
    }

    pub fn parameters(&self) -> ToolParameters {
        self.handler.parameters()
    }

    pub async fn execute(&self, arguments: Value) -> Result<ToolExecutionResult, crate::AgentError> {
        self.handler.execute(arguments).await
    }

    pub fn to_definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.name().to_string(),
            description: self.description().to_string(),
            parameters: self.parameters(),
        }
    }
}
