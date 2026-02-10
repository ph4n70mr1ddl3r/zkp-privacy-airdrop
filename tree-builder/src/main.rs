use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use tracing::{error, info};

mod io;
mod poseidon;
mod tree;

#[derive(Parser)]
#[command(name = "tree-builder")]
#[command(about = "Merkle tree builder for ZKP Privacy Airdrop", long_about = None)]
struct Cli {
    #[arg(long)]
    accounts_file: PathBuf,

    #[arg(long)]
    output: PathBuf,

    #[arg(long, default_value_t = 26)]
    height: u8,

    #[arg(long)]
    verify: bool,

    #[arg(long, default_value_t = 4)]
    threads: usize,
}

fn main() -> Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    let cli = Cli::parse();

    rayon::ThreadPoolBuilder::new()
        .num_threads(cli.threads)
        .build_global()
        .context("Failed to create thread pool")?;

    info!("Building Merkle tree from {}", cli.accounts_file.display());

    let pb = ProgressBar::new(100);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_bar())
            .progress_chars("=>-"),
    );

    pb.set_message("Reading addresses...");
    pb.set_position(5);

    let addresses =
        io::read_addresses(&cli.accounts_file).context("Failed to read addresses file")?;

    pb.set_message(format!("Found {} addresses", addresses.len()));
    pb.set_position(10);

    let tree = tree::build_merkle_tree(&addresses, cli.height)
        .map_err(|e| anyhow::anyhow!("Failed to build Merkle tree: {}", e))?;

    pb.set_message("Writing tree to file...");
    pb.set_position(90);

    io::write_tree(&tree, &cli.output).context("Failed to write tree file")?;

    pb.finish_with_message("Tree built successfully!");

    println!("\n{}", "Merkle Tree Summary:".green().bold());
    println!("  {} {}", "Height:".cyan(), tree.height);
    println!("  {} {}", "Leaves:".cyan(), tree.leaves.len());
    println!("  {} {}", "Root:".cyan(), hex::encode(tree.root));
    println!("  {} {}", "Output:".cyan(), cli.output.display());

    if cli.verify {
        println!("\n{}", "Verifying tree integrity...".yellow());
        verify_tree(&tree, &addresses)?;
        println!("{} Tree verification passed", "✓".green());
    }

    Ok(())
}

fn verify_tree(tree: &tree::MerkleTree, addresses: &[[u8; 20]]) -> Result<()> {
    for (i, &address) in addresses.iter().enumerate() {
        let leaf = poseidon::hash_address(address);
        let path = tree
            .get_path(i)
            .map_err(|e| anyhow::anyhow!("Failed to get path for leaf {}: {}", i, e))?;
        if !tree.verify_path(&leaf, &path) {
            error!("Verification failed for leaf {}", i);
            anyhow::bail!("Tree verification failed");
        }
    }
    Ok(())
}
