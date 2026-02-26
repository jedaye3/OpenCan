mod agent;
mod client;
mod config;
mod model;
mod session;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::config::Config;

#[derive(Debug, Parser)]
#[command(
    name = "opencan",
    version,
    about = "OpenCan: a local-first Rust coding assistant for terminal workflows"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Create or overwrite ~/.opencan/config.toml
    Onboard {
        #[arg(long)]
        force: bool,
        #[arg(long)]
        model: Option<String>,
        #[arg(long)]
        base_url: Option<String>,
        #[arg(long)]
        api_key_env: Option<String>,
    },
    /// Validate local setup and config
    Doctor,
    /// Start an interactive agent session
    Agent,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command.unwrap_or(Commands::Agent) {
        Commands::Onboard {
            force,
            model,
            base_url,
            api_key_env,
        } => {
            let path = config::write_default_config(force, model, base_url, api_key_env)?;
            println!("Wrote config: {}", path.display());
            println!(
                "Next: export OPEN_API_KEY (or OPENAI_API_KEY) and run `opencan doctor`."
            );
        }
        Commands::Doctor => run_doctor()?,
        Commands::Agent => agent::run_agent()?,
    }

    Ok(())
}

fn run_doctor() -> Result<()> {
    let path = config::config_path()?;
    println!("Config path: {}", path.display());

    let config = Config::load()?;
    config::ensure_layout(&config)?;

    println!("Model: {}", config.model);
    println!("Base URL: {}", config.base_url);
    println!("Configured API key env var: {}", config.api_key_env);

    match config.resolve_api_key() {
        Ok((source, _)) => println!("API key: present ({})", source),
        Err(err) => println!("API key: missing ({})", err),
    }

    let memory_path = config.memory_path()?;
    println!("Memory file: {}", memory_path.display());

    Ok(())
}
