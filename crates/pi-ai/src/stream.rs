use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<StreamChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChoice {
    pub index: u32,
    pub delta: StreamDelta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamDelta {
    pub role: Option<String>,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<StreamToolCall>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamToolCall {
    pub index: u32,
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub tool_type: Option<String>,
    pub function: Option<StreamFunction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamFunction {
    pub name: Option<String>,
    pub arguments: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamEvent {
    Token(String),
    ToolCall { id: String, name: String, arguments: String },
    Done,
    Error(String),
}

impl fmt::Display for StreamEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StreamEvent::Token(token) => write!(f, "{}", token),
            StreamEvent::ToolCall { id, name, arguments } => {
                write!(f, "[ToolCall: {}({}) args={}]", name, id, arguments)
            }
            StreamEvent::Done => write!(f, "[Done]"),
            StreamEvent::Error(err) => write!(f, "[Error: {}]", err),
        }
    }
}

pub fn parse_sse_line(line: &str) -> Result<Option<StreamChunk>, String> {
    let line = line.trim();
    
    if line.is_empty() {
        return Ok(None);
    }

    if !line.starts_with("data: ") {
        return Ok(None);
    }

    let data = &line[6..];

    if data == "[DONE]" {
        return Ok(None);
    }

    serde_json::from_str(data)
        .map(Some)
        .map_err(|e| format!("Failed to parse SSE data: {}", e))
}

pub fn chunk_to_event(chunk: &StreamChunk) -> Vec<StreamEvent> {
    let mut events = Vec::new();

    for choice in &chunk.choices {
        if let Some(content) = &choice.delta.content {
            if !content.is_empty() {
                events.push(StreamEvent::Token(content.clone()));
            }
        }

        if let Some(tool_calls) = &choice.delta.tool_calls {
            for tool_call in tool_calls {
                if let (Some(id), Some(name), Some(args)) = (
                    &tool_call.id,
                    tool_call.function.as_ref().and_then(|f| f.name.as_ref()),
                    tool_call.function.as_ref().and_then(|f| f.arguments.as_ref()),
                ) {
                    events.push(StreamEvent::ToolCall {
                        id: id.clone(),
                        name: name.clone(),
                        arguments: args.clone(),
                    });
                }
            }
        }

        if choice.finish_reason.is_some() {
            events.push(StreamEvent::Done);
        }
    }

    events
}
