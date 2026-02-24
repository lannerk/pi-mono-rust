use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "pi")]
#[command(about = "AI-powered coding agent", long_about = None)]
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
    Chat {
        #[arg(short, long)]
        model: Option<String>,

        #[arg(short, long)]
        temperature: Option<f32>,

        #[arg(short, long)]
        max_tokens: Option<u32>,

        #[arg(short, long)]
        stream: bool,

        #[arg(short, long)]
        ui: bool,
    },

    Code {
        #[arg(short, long)]
        file: Option<String>,

        #[arg(short, long)]
        edit: bool,

        #[arg(short, long)]
        review: bool,

        #[arg(short, long)]
        explain: bool,

        #[arg(short, long)]
        test: bool,
    },

    File {
        #[arg(short, long)]
        path: String,

        #[arg(short, long)]
        search: Option<String>,

        #[arg(short, long)]
        analyze: bool,

        #[arg(short, long)]
        summarize: bool,
    },

    Project {
        #[arg(short, long)]
        path: Option<String>,

        #[arg(short, long)]
        analyze: bool,

        #[arg(short, long)]
        document: bool,

        #[arg(short, long)]
        refactor: bool,
    },

    Agent {
        #[arg(short, long)]
        list: bool,

        #[arg(short, long)]
        create: Option<String>,

        #[arg(short, long)]
        delete: Option<String>,

        #[arg(short, long)]
        run: Option<String>,
    },

    Tool {
        #[arg(short, long)]
        list: bool,

        #[arg(short, long)]
        add: Option<String>,

        #[arg(short, long)]
        remove: Option<String>,
    },

    Config {
        #[arg(short, long)]
        show: bool,

        #[arg(short, long)]
        set: Option<String>,

        #[arg(short, long)]
        get: Option<String>,
    },
}
