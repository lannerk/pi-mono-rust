use async_trait::async_trait;
use pi_agent_core::{Tool, ToolExecutionResult, ToolHandler};
use pi_agent_core::tool::ToolParameters;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::result::Result;
use std::sync::Arc;

pub struct FileReadTool;

#[async_trait]
impl ToolHandler for FileReadTool {
    fn name(&self) -> &str {
        "file_read"
    }

    fn description(&self) -> &str {
        "Reads contents of a file"
    }

    fn parameters(&self) -> ToolParameters {
        let mut properties = HashMap::new();
        properties.insert(
            "path".to_string(),
            pi_agent_core::tool::Property {
                prop_type: "string".to_string(),
                description: Some("Path to file to read".to_string()),
                enum_values: None,
            },
        );

        ToolParameters {
            param_type: "object".to_string(),
            properties,
            required: Some(vec!["path".to_string()]),
        }
    }

    async fn execute(&self, arguments: Value) -> Result<ToolExecutionResult, pi_agent_core::AgentError> {
        let path = arguments
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| pi_agent_core::AgentError::InvalidToolArguments("Missing 'path' argument".to_string()))?;

        match fs::read_to_string(path) {
            Ok(content) => Ok(ToolExecutionResult::success(content)),
            Err(e) => Ok(ToolExecutionResult::failure(format!("Failed to read file: {}", e))),
        }
    }
}

pub struct FileWriteTool;

#[async_trait]
impl ToolHandler for FileWriteTool {
    fn name(&self) -> &str {
        "file_write"
    }

    fn description(&self) -> &str {
        "Writes content to a file"
    }

    fn parameters(&self) -> ToolParameters {
        let mut properties = HashMap::new();
        properties.insert(
            "path".to_string(),
            pi_agent_core::tool::Property {
                prop_type: "string".to_string(),
                description: Some("Path to file to write".to_string()),
                enum_values: None,
            },
        );
        properties.insert(
            "content".to_string(),
            pi_agent_core::tool::Property {
                prop_type: "string".to_string(),
                description: Some("Content to write to the file".to_string()),
                enum_values: None,
            },
        );

        ToolParameters {
            param_type: "object".to_string(),
            properties,
            required: Some(vec!["path".to_string(), "content".to_string()]),
        }
    }

    async fn execute(&self, arguments: Value) -> Result<ToolExecutionResult, pi_agent_core::AgentError> {
        let path = arguments
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| pi_agent_core::AgentError::InvalidToolArguments("Missing 'path' argument".to_string()))?;

        let content = arguments
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| pi_agent_core::AgentError::InvalidToolArguments("Missing 'content' argument".to_string()))?;

        if let Some(parent) = Path::new(path).parent() {
            fs::create_dir_all(parent)
                .map_err(|e| pi_agent_core::AgentError::ToolExecution(format!("Failed to create directory: {}", e)))?;
        }

        match fs::write(path, content) {
            Ok(_) => Ok(ToolExecutionResult::success("File written successfully".to_string())),
            Err(e) => Ok(ToolExecutionResult::failure(format!("Failed to write file: {}", e))),
        }
    }
}

pub struct FileSearchTool;

#[async_trait]
impl ToolHandler for FileSearchTool {
    fn name(&self) -> &str {
        "file_search"
    }

    fn description(&self) -> &str {
        "Searches for files matching a pattern"
    }

    fn parameters(&self) -> ToolParameters {
        let mut properties = HashMap::new();
        properties.insert(
            "pattern".to_string(),
            pi_agent_core::tool::Property {
                prop_type: "string".to_string(),
                description: Some("Pattern to search for (supports glob patterns)".to_string()),
                enum_values: None,
            },
        );
        properties.insert(
            "path".to_string(),
            pi_agent_core::tool::Property {
                prop_type: "string".to_string(),
                description: Some("Path to search in (default: current directory)".to_string()),
                enum_values: None,
            },
        );

        ToolParameters {
            param_type: "object".to_string(),
            properties,
            required: Some(vec!["pattern".to_string()]),
        }
    }

    async fn execute(&self, arguments: Value) -> Result<ToolExecutionResult, pi_agent_core::AgentError> {
        let pattern = arguments
            .get("pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| pi_agent_core::AgentError::InvalidToolArguments("Missing 'pattern' argument".to_string()))?;

        let search_path = arguments
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or(".");

        let mut results = Vec::new();

        for entry in walkdir::WalkDir::new(search_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() {
                if let Some(file_name) = path.file_name() {
                    if let Some(name_str) = file_name.to_str() {
                        if glob::Pattern::new(pattern)
                            .map(|p| p.matches(name_str))
                            .unwrap_or(false)
                        {
                            results.push(path.display().to_string());
                        }
                    }
                }
            }
        }

        let output = if results.is_empty() {
            "No files found".to_string()
        } else {
            results.join("\n")
        };

        Ok(ToolExecutionResult::success(output))
    }
}

pub struct ExecuteCommandTool;

#[async_trait]
impl ToolHandler for ExecuteCommandTool {
    fn name(&self) -> &str {
        "execute_command"
    }

    fn description(&self) -> &str {
        "Executes a shell command"
    }

    fn parameters(&self) -> ToolParameters {
        let mut properties = HashMap::new();
        properties.insert(
            "command".to_string(),
            pi_agent_core::tool::Property {
                prop_type: "string".to_string(),
                description: Some("Command to execute".to_string()),
                enum_values: None,
            },
        );
        properties.insert(
            "working_dir".to_string(),
            pi_agent_core::tool::Property {
                prop_type: "string".to_string(),
                description: Some("Working directory for the command".to_string()),
                enum_values: None,
            },
        );

        ToolParameters {
            param_type: "object".to_string(),
            properties,
            required: Some(vec!["command".to_string()]),
        }
    }

    async fn execute(&self, arguments: Value) -> Result<ToolExecutionResult, pi_agent_core::AgentError> {
        let command = arguments
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| pi_agent_core::AgentError::InvalidToolArguments("Missing 'command' argument".to_string()))?;

        let working_dir = arguments.get("working_dir").and_then(|v| v.as_str());

        let output = if let Some(dir) = working_dir {
            tokio::process::Command::new("sh")
                .args(["-c", command])
                .current_dir(dir)
                .output()
                .await
        } else {
            tokio::process::Command::new("sh")
                .args(["-c", command])
                .output()
                .await
        };

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                if output.status.success() {
                    let result = if !stderr.is_empty() {
                        format!("{}\n{}", stdout, stderr)
                    } else {
                        stdout
                    };
                    Ok(ToolExecutionResult::success(result))
                } else {
                    Ok(ToolExecutionResult::failure(format!(
                        "Command failed with exit code {:?}: {}",
                        output.status.code(),
                        if !stderr.is_empty() { stderr } else { stdout }
                    )))
                }
            }
            Err(e) => Ok(ToolExecutionResult::failure(format!("Failed to execute command: {}", e))),
        }
    }
}

pub fn get_default_tools() -> Vec<Tool> {
    vec![
        Tool::new(Arc::new(FileReadTool)),
        Tool::new(Arc::new(FileWriteTool)),
        Tool::new(Arc::new(FileSearchTool)),
        Tool::new(Arc::new(ExecuteCommandTool)),
    ]
}
