use clap::Parser;
use anyhow::{anyhow, Result};
use tracing::{info, Level};
use tracing_subscriber;

use pi_pods::{Cli, Commands, Pod, PodConfig, PodManager, PodStatus, PodUpdate, VllmManager};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // 设置日志
    let level = if cli.verbose {
        Level::DEBUG
    } else {
        Level::INFO
    };

    tracing_subscriber::fmt()
        .with_max_level(level)
        .init();

    info!("Starting pi-pods...");

    // 加载配置
    let config = match PodConfig::from_env() {
        Ok(config) => config,
        Err(e) => {
            return Err(anyhow!("Failed to load config: {}", e));
        }
    };

    info!("Loaded config: {:?}", config);

    // 初始化 PodManager
    let mut manager = PodManager::new(config.clone());

    // 初始化 VllmManager
    let vllm_manager = VllmManager::new(config);

    // 处理命令
    handle_command(cli, &mut manager, &vllm_manager).await
}

async fn handle_command(
    cli: Cli,
    manager: &mut PodManager,
    vllm_manager: &VllmManager,
) -> Result<()> {
    match cli.command {
        Commands::List { all } => {
            let pods = manager.list(all).await;
            println!("{:40} {:20} {:10} {:10} {:10} {:20} {:10}",
                     "NAME", "MODEL", "GPU COUNT", "REPLICAS", "STATUS", "ENDPOINT", "AGE");
            println!("{:40} {:20} {:10} {:10} {:10} {:20} {:10}",
                     "-".repeat(40), "-".repeat(20), "-".repeat(10), "-".repeat(10), "-".repeat(10), "-".repeat(20), "-".repeat(10));
            for pod in pods {
                let endpoint = pod.endpoint.as_deref().unwrap_or("");
                let age = format!("{:.0?}", pod.age());
                println!("{:40} {:20} {:10} {:10} {:10} {:20} {:10}",
                         pod.name, pod.model, pod.gpu_count, pod.replicas, pod.status, endpoint, age);
            }
        }

        Commands::Create {
            name,
            model,
            gpu_type,
            gpu_count,
            replicas,
        } => {
            let gpu_type = gpu_type.unwrap_or_else(|| "a100".to_string());
            let gpu_count = gpu_count.unwrap_or(1);
            let replicas = replicas.unwrap_or(1);

            let pod = Pod::new(
                name.clone(),
                manager.config.namespace.clone(),
                model,
                gpu_type,
                gpu_count,
            )
            .with_replicas(replicas);

            manager.create(pod).await?;
            println!("Created pod: {}", name);
        }

        Commands::Delete { name, force: _ } => {
            let _ = manager.delete(&name).await?;
            println!("Deleted pod: {}", name);
        }

        Commands::Start { name } => {
            manager.start(&name).await?;
            println!("Started pod: {}", name);
        }

        Commands::Stop { name, force: _ } => {
            manager.stop(&name).await?;
            println!("Stopped pod: {}", name);
        }

        Commands::Status { name } => {
            let pod = manager.status(&name).await?;
            println!("Name: {}", pod.name);
            println!("Model: {}", pod.model);
            println!("GPU Type: {}", pod.gpu_type);
            println!("GPU Count: {}", pod.gpu_count);
            println!("Replicas: {}", pod.replicas);
            println!("Status: {}", pod.status);
            println!("Created At: {}", pod.created_at);
            println!("Updated At: {}", pod.updated_at);
            println!("Endpoint: {:?}", pod.endpoint);
            println!("Labels: {:?}", pod.labels);
            println!("Annotations: {:?}", pod.annotations);
        }

        Commands::Logs { name, follow: _, tail: _ } => {
            let logs = manager.logs(&name).await?;
            println!("{}", logs);
        }

        Commands::Scale { name, replicas } => {
            manager.scale(&name, replicas).await?;
            println!("Scaled pod {} to {} replicas", name, replicas);
        }

        Commands::Update { name, model, gpu_count } => {
            manager.update(&name, PodUpdate { model, gpu_count }).await?;
            println!("Updated pod: {}", name);
        }
    }

    Ok(())
}
