use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;
use tracing::{info, error, warn};
use tracing_subscriber;

mod commands;
mod config;
mod crypto;
mod types;
mod plonk_prover;
mod tree;

use config::Config;

#[derive(Parser)]
#[command(name = "zkp-airdrop")]
#[command(about = "ZKP Privacy Airdrop CLI", long_about = None)]
#[command(version = "1.0.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true)]
    verbose: bool,

    #[arg(short, long, global = true)]
    quiet: bool,

    #[arg(short, long, global = true)]
    config: Option<PathBuf>,

    #[arg(short, long, global = true)]
    proof_system: Option<String>,
}

#[derive(Subcommand)]
    enum Commands {
        GenerateProof {
            #[arg(long)]
            private_key: Option<String>,
            #[arg(long)]
            private_key_file: Option<PathBuf>,
            #[arg(long)]
            private_key_stdin: bool,
            #[arg(long)]
            recipient: String,
            #[arg(long)]
            merkle_tree: String,
            #[arg(long)]
            output: Option<PathBuf>,
            #[arg(long, default_value = "json")]
            format: String,
        },
        GenerateProofPlonk {
            #[arg(long)]
            private_key: Option<String>,
            #[arg(long)]
            private_key_file: Option<PathBuf>,
            #[arg(long)]
            private_key_stdin: bool,
            #[arg(long)]
            recipient: String,
            #[arg(long)]
            merkle_tree: String,
            #[arg(long)]
            output: Option<PathBuf>,
            #[arg(long, default_value = "json")]
            format: String,
        },
    VerifyProof {
        #[arg(long)]
        proof: PathBuf,
        #[arg(long)]
        merkle_root: String,
        #[arg(long)]
        verification_key: Option<PathBuf>,
    },
    Submit {
        #[arg(long)]
        proof: PathBuf,
        #[arg(long)]
        relayer_url: Option<String>,
        #[arg(long)]
        recipient: String,
        #[arg(long)]
        wait: bool,
        #[arg(long, default_value = "120")]
        timeout: u64,
    },
    CheckStatus {
        #[arg(long)]
        nullifier: String,
        #[arg(long)]
        relayer_url: Option<String>,
        #[arg(long)]
        rpc_url: Option<String>,
    },
    DownloadTree {
        #[arg(long)]
        source: String,
        #[arg(long)]
        output: PathBuf,
        #[arg(long, default_value = "binary")]
        format: String,
        #[arg(long)]
        verify: bool,
        #[arg(long)]
        resume: bool,
        #[arg(long, default_value = "100")]
        chunk_size: u64,
    },
    Config {
        #[arg(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    Show,
    Set { key: String, value: String },
    Reset,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(if cli.verbose {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        })
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .context("Failed to set tracing subscriber")?;

    let config_path = cli.config.unwrap_or_else(|| {
        dirs::home_dir()
            .unwrap_or_else(|| {
                eprintln!("Error: Could not determine home directory. Please specify config file with --config.");
                std::process::exit(1);
            })
            .join(".zkp-airdrop")
            .join("config.toml")
    });

    let config = Config::load_or_default(&config_path)?;

    let result = match cli.command {
        Commands::GenerateProof {
            private_key,
            private_key_file,
            private_key_stdin,
            recipient,
            merkle_tree,
            output,
            format,
        } => commands::generate_proof::execute(
            private_key,
            private_key_file,
            private_key_stdin,
            recipient,
            merkle_tree,
            output,
            format,
            &config,
        ).await,
        Commands::GenerateProofPlonk {
            private_key,
            private_key_file,
            private_key_stdin,
            recipient,
            merkle_tree,
            output,
            format,
        } => commands::generate_proof_plonk::execute(
            private_key,
            private_key_file,
            private_key_stdin,
            recipient,
            merkle_tree,
            output,
            format,
            cli.proof_system.unwrap_or_else(|| "groth16".to_string()),
            &config,
        ).await,
        Commands::VerifyProof {
            proof,
            merkle_root,
            verification_key,
        } => commands::verify_proof::execute(
            proof,
            merkle_root,
            verification_key,
        ),

        Commands::Submit {
            proof,
            relayer_url,
            recipient,
            wait,
            timeout,
        } => commands::submit::execute(
            proof,
            relayer_url,
            recipient,
            wait,
            timeout,
            &config,
        ).await,

        Commands::CheckStatus {
            nullifier,
            relayer_url,
            rpc_url,
        } => commands::check_status::execute(
            nullifier,
            relayer_url,
            rpc_url,
            &config,
        ).await,

        Commands::DownloadTree {
            source,
            output,
            format,
            verify,
            resume,
            chunk_size,
        } => commands::download_tree::execute(
            source,
            output,
            format,
            verify,
            resume,
            chunk_size,
        ).await,

        Commands::Config { action } => commands::config_cmd::execute(
            action,
            &config_path,
            &config,
        ),
    };

    if let Err(e) = &result {
        if !cli.quiet {
            eprintln!("{} {}", "Error:".red(), e);
        }
        std::process::exit(1);
    }

    Ok(())
}
