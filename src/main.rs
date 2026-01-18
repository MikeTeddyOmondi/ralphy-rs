use anyhow::Result;
use clap::Parser;
use ralphy_rs::{cli::Cli, config::Config, run_autonomous_loop};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    // Parse CLI arguments
    let cli = Cli::parse();

    // Convert CLI to Config
    let config = Config::from_cli(cli)?;

    // Show banner
    config.show_banner();

    // Run the autonomous loop
    run_autonomous_loop(config).await?;

    Ok(())
}
