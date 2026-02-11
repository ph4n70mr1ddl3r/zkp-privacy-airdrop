use anyhow::{Context, Result};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use tracing::info;

use crate::config::Config;
use crate::crypto::{derive_address, generate_nullifier, read_private_key};

pub async fn execute(
    private_key_opt: Option<String>,
    private_key_file: Option<PathBuf>,
    private_key_stdin: bool,
    recipient: String,
    merkle_tree: String,
    output: Option<PathBuf>,
    format: String,
    config: &Config,
) -> Result<()> {
    if private_key_opt.is_some() {
        eprintln!(
            "{}: Passing private key via --private-key is deprecated. Use --private-key-file or --private-key-stdin instead.",
            "WARNING".yellow().bold()
        );
    }

    info!("Generating proof...");

    let private_key_wrapper =
        read_private_key(private_key_opt, private_key_file, private_key_stdin)?;
    let private_key: [u8; 32] = private_key_wrapper
        .try_into_array()
        .map_err(|e| anyhow::anyhow!("Invalid private key length: expected 32 bytes",))?;

    let address =
        derive_address(&private_key).context("Failed to derive address from private key")?;

    let address_str = format!("{:#x}", address);
    tracing::debug!("Address: {}", address_str);

    let nullifier = generate_nullifier(&private_key).context("Failed to generate nullifier")?;
    let nullifier_hex = format!("0x{}", hex::encode(&nullifier.as_bytes()));

    tracing::debug!("Recipient: {}", recipient);

    let pb = ProgressBar::new(100);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .expect("Failed to create progress style")
            .progress_chars("=>-"),
    );
    pb.set_message("Deriving public key...");
    pb.set_position(10);

    pb.set_message("Computing leaf hash...");
    pb.set_position(30);

    pb.set_message("Generating nullifier...");
    pb.set_position(50);

    pb.set_message("Finding Merkle path...");
    pb.set_position(70);

    pb.set_message("Generating zero-knowledge proof...");
    pb.set_position(90);

    let tree = crate::tree::MerkleTree::from_file(&std::path::PathBuf::from(&merkle_tree))
        .context("Failed to load Merkle tree")?;

    let merkle_root = tree.root.clone();
    let merkle_root_hex = format!("0x{}", hex::encode(&merkle_root));

    pb.finish();

    Err(anyhow::anyhow!(
        "This command is deprecated. Groth16 proof system is no longer supported. \
         Please use the PLONK proof system instead with the 'generate-proof-plonk' command.\n\n\
         Example:\n\
         zkp-airdrop generate-proof-plonk --private-key $PRIVATE_KEY --recipient $RECIPIENT --merkle-tree $MERKLE_TREE"
    ))
}
