use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;
use tokio::signal;
use tracing::info;

mod commands;
mod config;
mod crypto;
mod plonk_prover;
mod tree;
mod types;
mod types_plonk;

use config::Config;

fn validate_private_key_source(
    private_key: &Option<String>,
    private_key_file: &Option<PathBuf>,
    private_key_stdin: bool,
) -> Result<()> {
    let sources = [
        private_key.is_some(),
        private_key_file.is_some(),
        private_key_stdin,
        std::env::var("ZKP_AIRDROP_PRIVATE_KEY").is_ok(),
    ];

    let active_count = sources.iter().filter(|&&x| x).count();

    if active_count == 0 {
        return Err(anyhow::anyhow!(
            "No private key source provided. Use one of:\n\
             --private-key <KEY>\n\
             --private-key-file <PATH>\n\
             --private-key-stdin\n\
             ZKP_AIRDROP_PRIVATE_KEY environment variable"
        ));
    }

    if active_count > 1 {
        return Err(anyhow::anyhow!(
            "Multiple private key sources provided. Please use exactly one of:\n\
             --private-key <KEY>\n\
             --private-key-file <PATH>\n\
             --private-key-stdin\n\
             ZKP_AIRDROP_PRIVATE_KEY environment variable"
        ));
    }

    Ok(())
}

fn validate_format_parameter(format: &str) -> Result<()> {
    if format != "json" {
        return Err(anyhow::anyhow!(
            "Invalid format '{}'. Only 'json' format is currently supported.",
            format
        ));
    }
    Ok(())
}

#[derive(Parser)]
#[command(name = "zkp-airdrop")]
#[command(about = "ZKP Privacy Airdrop CLI", long_about = None)]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true)]
    verbose: bool,

    #[arg(short, long, global = true)]
    quiet: bool,

    #[arg(short, long, global = true)]
    config: Option<PathBuf>,
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
        #[arg(long, default_value = "120")]
        timeout: u64,
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
        #[arg(long, default_value = "120")]
        timeout: u64,
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
        #[command(subcommand)]
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

    if cli.verbose && cli.quiet {
        return Err(anyhow::anyhow!(
            "Cannot specify both --verbose and --quiet. Use one or neither."
        ));
    }

    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(if cli.verbose {
            tracing::Level::DEBUG
        } else if cli.quiet {
            tracing::Level::WARN
        } else {
            tracing::Level::INFO
        })
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .context("Failed to set tracing subscriber")?;

    let config_path = if let Some(path) = cli.config {
        path
    } else {
        dirs::home_dir()
            .map(|home_dir| home_dir.join(".zkp-airdrop").join("config.toml"))
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Could not determine home directory. Please specify config file with --config."
                )
            })?
    };

    let config = Config::load_or_default(&config_path)?;

    tokio::select! {
        result = async {
            match cli.command {
                Commands::GenerateProof {
                    private_key,
                    private_key_file,
                    private_key_stdin,
                    recipient,
                    merkle_tree,
                    output,
                    format,
                    timeout,
                } => {
                    validate_private_key_source(&private_key, &private_key_file, private_key_stdin)?;
                    validate_format_parameter(&format)?;
                    commands::generate_proof::execute(
                        private_key,
                        private_key_file,
                        private_key_stdin,
                        recipient,
                        merkle_tree,
                        output,
                        format,
                        timeout,
                        &config,
                    )
                    .await
                }
                Commands::GenerateProofPlonk {
                    private_key,
                    private_key_file,
                    private_key_stdin,
                    recipient,
                    merkle_tree,
                    output,
                    format,
                    timeout,
                } => {
                    validate_private_key_source(&private_key, &private_key_file, private_key_stdin)?;
                    validate_format_parameter(&format)?;
                    commands::generate_proof_plonk::execute(
                        private_key,
                        private_key_file,
                        private_key_stdin,
                        recipient,
                        merkle_tree,
                        output,
                        format,
                        timeout,
                        "plonk".to_string(),
                        &config,
                    )
                    .await
                }
                Commands::VerifyProof {
                    proof,
                    merkle_root,
                    verification_key,
                } => commands::verify_proof::execute(proof, merkle_root, verification_key),

                Commands::Submit {
                    proof,
                    relayer_url,
                    recipient,
                    wait,
                    timeout,
                } => commands::submit::execute(proof, relayer_url, recipient, wait, timeout, &config).await,

                Commands::CheckStatus {
                    nullifier,
                    relayer_url,
                    rpc_url,
                } => commands::check_status::execute(nullifier, relayer_url, rpc_url, &config).await,

                Commands::DownloadTree {
                    source,
                    output,
                    format,
                    verify,
                    resume,
                    chunk_size,
                } => {
                    commands::download_tree::execute(source, output, format, verify, resume, chunk_size)
                        .await
                }

                Commands::Config { action } => commands::config_cmd::execute(action, &config_path, &config),
            }
        } => {
            if let Err(e) = &result {
                if !cli.quiet {
                    eprintln!("{} {}", "Error:".red(), sanitize_error_message(&e.to_string()));
                }
                std::process::exit(1);
            }
            Ok(())
        }
        _ = signal::ctrl_c() => {
            info!("Received interrupt signal, shutting down...");
            std::process::exit(130);
        }
    }
}

fn sanitize_error_message(msg: &str) -> String {
    let mut sanitized = String::new();

    for c in msg.chars() {
        match c {
            '0'..='9' | 'a'..='z' | 'A'..='Z' => sanitized.push(c),
            ' ' | '.' | ',' | ':' | '-' | '_' | '(' | ')' | '[' | ']' | '{' | '}' => {
                sanitized.push(c)
            }
            _ => {}
        }
    }

    if sanitized.is_empty() || sanitized.trim().is_empty() {
        "An error occurred. Please check your inputs.".to_string()
    } else {
        sanitized
    }
}
