mod cli;
mod commands;
mod config;
mod tools;
mod ui;

use anyhow::Result;
use clap::Parser;
use cli::Cli;
use commands::handle_command;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");

    let cli = Cli::parse();

    if let Err(e) = handle_command(cli).await {
        error!("Error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}
