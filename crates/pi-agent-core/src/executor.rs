use crate::context::Context;
use crate::error::{AgentError, AgentResult};
use crate::state::StateStore;
use crate::tool_registry::ToolRegistry;
use pi_ai::{Client, Message};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::pin::Pin;
use futures::StreamExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutorConfig {
    pub max_iterations: usize,
    pub timeout_secs: u64,
    pub enable_tools: bool,
    pub auto_tool_execution: bool,
    pub stop_on_tool_error: bool,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            max_iterations: 10,
            timeout_secs: 300,
            enable_tools: true,
            auto_tool_execution: true,
            stop_on_tool_error: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub messages: Vec<Message>,
    pub tool_calls: Vec<ToolCallInfo>,
    pub iterations: usize,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallInfo {
    pub id: String,
    pub name: String,
    pub arguments: String,
    pub result: Option<String>,
    pub success: bool,
}

#[derive(Debug, Clone)]
pub enum StreamEvent {
    Token(String),
    ToolCall { id: String, name: String, arguments: String },
    Done,
    Error(String),
}

pub struct Executor {
    config: ExecutorConfig,
    llm_client: Arc<Client>,
    tool_registry: Arc<ToolRegistry>,
    state_store: Arc<StateStore>,
}

impl Executor {
    pub fn new(
        config: ExecutorConfig,
        llm_client: Arc<Client>,
        tool_registry: Arc<ToolRegistry>,
        state_store: Arc<StateStore>,
    ) -> Self {
        Self {
            config,
            llm_client,
            tool_registry,
            state_store,
        }
    }

    pub async fn execute(&self, context: &mut Context) -> AgentResult<ExecutionResult> {
        let mut iterations = 0;
        let mut executed_tool_calls = Vec::new();
        let mut success = true;
        let mut error = None;

        while iterations < self.config.max_iterations {
            iterations += 1;

            let messages = context.get_messages();
            let request = pi_ai::models::ChatCompletionRequest {
                model: "gpt-4".to_string(),
                messages: messages.clone(),
                tools: if self.config.enable_tools {
                    Some(self.tool_registry.get_tool_definitions().await)
                } else {
                    None
                },
                tool_choice: None,
                temperature: None,
                top_p: None,
                max_tokens: None,
                stream: Some(false),
                stop: None,
                presence_penalty: None,
                frequency_penalty: None,
                user: None,
            };

            let provider_name = "openai";
            let response = self.llm_client.chat(provider_name, request).await?;

            if let Some(choice) = response.choices.first() {
                let message = &choice.message;
                context.add_message(message.clone());

                if let Some(tool_calls) = &message.tool_calls {
                    for tool_call in tool_calls {
                        let tool_call_info = ToolCallInfo {
                            id: tool_call.id.clone(),
                            name: tool_call.function.name.clone(),
                            arguments: tool_call.function.arguments.clone(),
                            result: None,
                            success: false,
                        };

                        if self.config.auto_tool_execution {
                            match self.execute_tool(tool_call).await {
                                Ok(result) => {
                                    context.add_tool_message(
                                        result.clone(),
                                        tool_call.id.clone(),
                                    );
                                    executed_tool_calls.push(ToolCallInfo {
                                        result: Some(result),
                                        success: true,
                                        ..tool_call_info
                                    });
                                }
                                Err(e) => {
                                    if self.config.stop_on_tool_error {
                                        success = false;
                                        error = Some(e.to_string());
                                        break;
                                    }
                                    context.add_tool_message(
                                        format!("Error: {}", e),
                                        tool_call.id.clone(),
                                    );
                                    executed_tool_calls.push(ToolCallInfo {
                                        result: Some(format!("Error: {}", e)),
                                        success: false,
                                        ..tool_call_info
                                    });
                                }
                            }
                        } else {
                            executed_tool_calls.push(tool_call_info);
                        }
                    }

                    if self.config.auto_tool_execution && executed_tool_calls.iter().any(|t| t.success) {
                        continue;
                    }
                }
            }

            break;
        }

        Ok(ExecutionResult {
            messages: context.get_messages(),
            tool_calls: executed_tool_calls,
            iterations,
            success,
            error,
        })
    }

    pub async fn execute_stream(
        &self,
        context: &mut Context,
    ) -> AgentResult<Pin<Box<dyn futures::Stream<Item = AgentResult<StreamEvent>> + Send>>> {
        let messages = context.get_messages();
        let request = pi_ai::models::ChatCompletionRequest {
            model: "gpt-4".to_string(),
            messages: messages.clone(),
            tools: if self.config.enable_tools {
                Some(self.tool_registry.get_tool_definitions().await)
            } else {
                None
            },
            tool_choice: None,
            temperature: None,
            top_p: None,
            max_tokens: None,
            stream: Some(true),
            stop: None,
            presence_penalty: None,
            frequency_penalty: None,
            user: None,
        };

        let provider_name = "openai";
        let stream = self.llm_client.chat_stream(provider_name, request).await?;

        let mapped_stream = stream.map(|result| {
            result.map_err(|e| AgentError::Llm(e)).and_then(|event| match event {
                pi_ai::stream::StreamEvent::Token(token) => Ok(StreamEvent::Token(token)),
                pi_ai::stream::StreamEvent::ToolCall { id, name, arguments } => {
                    Ok(StreamEvent::ToolCall { id, name, arguments })
                }
                pi_ai::stream::StreamEvent::Done => Ok(StreamEvent::Done),
                pi_ai::stream::StreamEvent::Error(err) => Ok(StreamEvent::Error(err)),
            })
        });

        Ok(Box::pin(mapped_stream))
    }

    async fn execute_tool(&self, tool_call: &pi_ai::message::ToolCall) -> AgentResult<String> {
        let arguments: serde_json::Value = serde_json::from_str(&tool_call.function.arguments)
            .map_err(|e| AgentError::InvalidToolArguments(e.to_string()))?;

        let result = self.tool_registry.execute(&tool_call.function.name, arguments).await?;

        if result.success {
            Ok(result.output)
        } else {
            Err(AgentError::ToolExecution(
                result.error.unwrap_or_else(|| "Unknown error".to_string()),
            ))
        }
    }

    pub fn config(&self) -> &ExecutorConfig {
        &self.config
    }
}
