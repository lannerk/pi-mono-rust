use crate::cli::{Cli, Commands};
use crate::config::Config;
use crate::ui::run_chat_ui;
use anyhow::Result;
use futures::StreamExt;
use pi_ai::{Client, Config as LLMConfig};
use pi_agent_core::{Agent, AgentConfig, ToolRegistry};
use std::sync::Arc;
use tracing::info;

pub async fn handle_command(cli: Cli) -> Result<()> {
    let config = Config::load(cli.config.as_deref())?;

    match cli.command {
        Commands::Chat {
            model,
            temperature,
            max_tokens,
            stream,
            ui,
        } => {
            handle_chat_command(config, model, temperature, max_tokens, stream, ui).await
        }
        Commands::Code {
            file,
            edit,
            review,
            explain,
            test,
        } => {
            handle_code_command(config, file, edit, review, explain, test).await
        }
        Commands::File {
            path,
            search,
            analyze,
            summarize,
        } => {
            handle_file_command(config, path, search, analyze, summarize).await
        }
        Commands::Project {
            path,
            analyze,
            document,
            refactor,
        } => {
            handle_project_command(config, path, analyze, document, refactor).await
        }
        Commands::Agent {
            list,
            create,
            delete,
            run,
        } => {
            handle_agent_command(config, list, create, delete, run).await
        }
        Commands::Tool {
            list,
            add,
            remove,
        } => {
            handle_tool_command(config, list, add, remove).await
        }
        Commands::Config {
            show,
            set,
            get,
        } => {
            handle_config_command(config, show, set, get).await
        }
    }
}

async fn handle_chat_command(
    config: Config,
    model: Option<String>,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
    stream: bool,
    ui: bool,
) -> Result<()> {
    info!("Starting chat mode");

    let llm_config = LLMConfig::default();
    let llm_client = Arc::new(Client::new(llm_config)?);
    let tool_registry = Arc::new(ToolRegistry::new());

    let agent_config = AgentConfig::new(
        "default".to_string(),
        "Pi Coding Agent".to_string(),
        "You are a helpful AI coding assistant. Help users with their programming tasks, provide code examples, explain concepts, and assist with debugging.".to_string(),
    )
    .with_model(model.unwrap_or_else(|| config.default_model.clone()))
    .with_temperature(temperature.unwrap_or(config.default_temperature))
    .with_max_tokens(max_tokens.unwrap_or(config.default_max_tokens));

    let agent = Arc::new(Agent::new(agent_config, llm_client, tool_registry));
    agent.initialize().await?;

    if ui {
        run_chat_ui(agent).await?;
    } else {
        run_chat_cli(agent, stream).await?;
    }

    Ok(())
}

async fn run_chat_cli(agent: Arc<Agent>, stream: bool) -> Result<()> {
    println!("Pi Coding Agent - Chat Mode");
    println!("Type 'quit' or 'exit' to end the conversation\n");

    loop {
        print!(" > ");
        std::io::Write::flush(&mut std::io::stdout())?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        if input == "quit" || input == "exit" {
            println!("Goodbye!");
            break;
        }

        if stream {
            let mut stream_response = agent.chat_stream(input.to_string()).await?;
            print!("Assistant: ");
            std::io::Write::flush(&mut std::io::stdout())?;

            while let Some(chunk) = stream_response.next().await {
                match chunk {
                    Ok(text) => {
                        print!("{}", text);
                        std::io::Write::flush(&mut std::io::stdout())?;
                    }
                    Err(e) => {
                        eprintln!("\nError: {}", e);
                        break;
                    }
                }
            }
            println!();
        } else {
            let result = agent.chat(input.to_string()).await?;
            if let Some(last_message) = result.messages.last() {
                println!("Assistant: {}", last_message.content);
            } else {
                println!("Assistant: No response");
            }
        }

        println!();
    }

    Ok(())
}

async fn handle_code_command(
    _config: Config,
    file: Option<String>,
    _edit: bool,
    _review: bool,
    _explain: bool,
    _test: bool,
) -> Result<()> {
    info!("Handling code command for file: {:?}", file);
    println!("Code command - Feature coming soon!");
    Ok(())
}

async fn handle_file_command(
    _config: Config,
    path: String,
    _search: Option<String>,
    _analyze: bool,
    _summarize: bool,
) -> Result<()> {
    info!("Handling file command for path: {}", path);
    println!("File command - Feature coming soon!");
    Ok(())
}

async fn handle_project_command(
    _config: Config,
    path: Option<String>,
    _analyze: bool,
    _document: bool,
    _refactor: bool,
) -> Result<()> {
    info!("Handling project command for path: {:?}", path);
    println!("Project command - Feature coming soon!");
    Ok(())
}

async fn handle_agent_command(
    _config: Config,
    _list: bool,
    _create: Option<String>,
    _delete: Option<String>,
    _run: Option<String>,
) -> Result<()> {
    println!("Agent command - Feature coming soon!");
    Ok(())
}

async fn handle_tool_command(
    _config: Config,
    _list: bool,
    _add: Option<String>,
    _remove: Option<String>,
) -> Result<()> {
    println!("Tool command - Feature coming soon!");
    Ok(())
}

async fn handle_config_command(
    _config: Config,
    _show: bool,
    _set: Option<String>,
    _get: Option<String>,
) -> Result<()> {
    println!("Config command - Feature coming soon!");
    Ok(())
}
