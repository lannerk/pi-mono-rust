use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "pi-pods")]
#[command(about = "CLI for managing vLLM deployments on GPU pods", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[arg(short, long, global = true)]
    pub config: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    List {
        #[arg(short, long)]
        all: bool,
    },

    Create {
        #[arg(short, long)]
        name: String,

        #[arg(short, long)]
        model: String,

        #[arg(short, long)]
        gpu_type: Option<String>,

        #[arg(short, long)]
        gpu_count: Option<u32>,

        #[arg(short, long)]
        replicas: Option<u32>,
    },

    Delete {
        #[arg(short, long)]
        name: String,

        #[arg(short, long)]
        force: bool,
    },

    Start {
        #[arg(short, long)]
        name: String,
    },

    Stop {
        #[arg(short, long)]
        name: String,

        #[arg(short, long)]
        force: bool,
    },

    Status {
        #[arg(short, long)]
        name: String,
    },

    Logs {
        #[arg(short, long)]
        name: String,

        #[arg(short, long)]
        follow: bool,

        #[arg(short, long)]
        tail: Option<usize>,
    },

    Scale {
        #[arg(short, long)]
        name: String,

        #[arg(short, long)]
        replicas: u32,
    },

    Update {
        #[arg(short, long)]
        name: String,

        #[arg(short, long)]
        model: Option<String>,

        #[arg(short, long)]
        gpu_count: Option<u32>,
    },
}
