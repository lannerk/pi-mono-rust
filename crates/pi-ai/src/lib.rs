pub mod client;
pub mod config;
pub mod error;
pub mod message;
pub mod models;
pub mod provider;
pub mod stream;
pub mod tool;

#[cfg(test)]
mod tests;

pub use client::Client;
pub use config::{Config, ProviderConfig};
pub use error::{Error, Result};
pub use message::{Message, MessageRole, ToolCall, ToolResult};
pub use models::{ChatCompletionRequest, ChatCompletionResponse, CompletionChoice, ToolDefinition, FunctionDefinition};
pub use provider::{Provider, ProviderType};
pub use stream::{StreamChunk, StreamEvent};
pub use tool::{Tool, ToolInputSchema};
