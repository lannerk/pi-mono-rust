use thiserror::Error;

#[derive(Debug, Error)]
pub enum AgentError {
    #[error("LLM error: {0}")]
    Llm(#[from] pi_ai::Error),

    #[error("Tool execution failed: {0}")]
    ToolExecution(String),

    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Invalid tool arguments: {0}")]
    InvalidToolArguments(String),

    #[error("State error: {0}")]
    State(String),

    #[error("Context error: {0}")]
    Context(String),

    #[error("Execution timeout")]
    Timeout,

    #[error("Maximum iterations exceeded")]
    MaxIterationsExceeded,

    #[error("Agent not initialized")]
    NotInitialized,

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Other error: {0}")]
    Other(String),
}

pub type AgentResult<T> = std::result::Result<T, AgentError>;
