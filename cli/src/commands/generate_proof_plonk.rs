use anyhow::{Context, Result};
use colored::Colorize;
use std::path::PathBuf;
use tracing::info;

use crate::config::Config;
use crate::plonk_prover::generate_plonk_proof;

#[allow(clippy::too_many_arguments)]
pub async fn execute(
    private_key_opt: Option<String>,
    private_key_file: Option<PathBuf>,
    private_key_stdin: bool,
    recipient: String,
    merkle_tree: String,
    output: Option<PathBuf>,
    _format: String,
    proof_system: String,
    config: &Config,
) -> Result<()> {
    info!("Generating {} proof...", proof_system);

    let use_plonk = proof_system.to_lowercase() == "plonk";

    let private_key_wrapper =
        crate::crypto::read_private_key(private_key_opt, private_key_file, private_key_stdin)?;
    let private_key: [u8; 32] = private_key_wrapper
        .try_into_array()
        .map_err(|_| anyhow::anyhow!("Invalid private key length: expected 32 bytes"))?;

    let address = crate::crypto::derive_address(&private_key)
        .context("Failed to derive address from private key")?;

    let recipient_address: ethers::types::Address = recipient
        .parse()
        .context("Failed to parse recipient address")?;

    println!("{} {}", "Address:".green(), address);
    println!("{} {}", "Recipient:".green(), recipient);
    println!("{} Proof System: {}", "Using:".blue().bold(), proof_system);

    let path = PathBuf::from(&merkle_tree);

    let canonical_path = path
        .canonicalize()
        .with_context(|| format!("Failed to canonicalize path: {merkle_tree}"))?;

    if !canonical_path.exists() {
        anyhow::bail!(
            "Merkle tree file does not exist: {}",
            canonical_path.display()
        );
    }

    if !canonical_path.is_file() {
        anyhow::bail!(
            "Merkle tree path is not a file: {}",
            canonical_path.display()
        );
    }

    let merkle_tree = crate::tree::MerkleTree::from_file(&canonical_path)
        .context("Failed to load Merkle tree file")?;

    let pb = indicatif::ProgressBar::new(100);
    pb.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .expect("Failed to create progress style")
            .progress_chars("=>-"),
    );
    pb.set_message("Deriving public key...");
    pb.set_position(10);

    let plonk_proof_data = if use_plonk {
        generate_plonk_proof(&private_key, recipient_address, &merkle_tree)
            .context("Failed to generate PLONK proof")?
    } else {
        anyhow::bail!(
            "Unsupported proof system: '{proof_system}'. Only PLONK is supported. Use --proof-system plonk or the generate-proof-plonk subcommand."
        )
    };

    pb.finish_with_message("Proof generated!");

    let proof_data: crate::types_plonk::ProofData = plonk_proof_data.into();

    let output_path = output.unwrap_or_else(|| PathBuf::from("proof.json"));
    let json =
        serde_json::to_string_pretty(&proof_data).context("Failed to serialize proof data")?;

    std::fs::write(&output_path, json).context("Failed to write proof file")?;

    println!("\n{} {}", "Proof saved to:".cyan(), output_path.display());
    println!("\n{proof_data}");
    println!("\n{}", "Next steps:".yellow().bold());
    println!("  Submit via relayer:");
    println!(
        "    zkp-airdrop submit --proof {} --relayer-url {} --recipient {}",
        output_path.display(),
        config
            .get_relayer_url()
            .unwrap_or_else(|_| "<RELAYER_URL>".to_string()),
        recipient
    );
    println!();

    Ok(())
}
