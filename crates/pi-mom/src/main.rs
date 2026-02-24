use anyhow::Result;
use clap::Parser;
use pi_ai::Client;
use pi_agent_core::{Agent, AgentConfig, ToolRegistry};
use pi_mom::{SlackBot, SlackConfig};
use std::sync::Arc;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Parser, Debug)]
#[command(name = "pi-mom")]
#[command(about = "Slack bot for Pi coding agent", long_about = None)]
struct Cli {
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let subscriber = FmtSubscriber::builder()
        .with_max_level(if cli.verbose { Level::DEBUG } else { Level::INFO })
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");

    info!("Starting Pi Mom - Slack Bot");

    let config = SlackConfig::from_env()?;
    let llm_client = Arc::new(Client::from_env()?);
    let tool_registry = Arc::new(ToolRegistry::new());

    let agent_config = AgentConfig::new(
        "slack-bot".to_string(),
        "Pi Slack Bot".to_string(),
        "You are a helpful AI assistant integrated with Slack. Provide concise and helpful responses to user questions.",
    );

    let agent = Arc::new(Agent::new(agent_config, llm_client, tool_registry));
    agent.initialize().await?;

    let bot = SlackBot::new(config, agent);
    bot.start().await?;

    Ok(())
}
