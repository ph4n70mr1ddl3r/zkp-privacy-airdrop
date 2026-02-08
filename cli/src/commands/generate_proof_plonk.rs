use anyhow::{Context, Result};
use colored::Colorize;
use std::path::PathBuf;
use tracing::info;

use crate::config::Config;
use crate::plonk_prover::generate_plonk_proof;
use crate::types::{PLONKProof, PLONKProofData};

pub async fn execute(
    private_key_opt: Option<String>,
    private_key_file: Option<PathBuf>,
    private_key_stdin: bool,
    recipient: String,
    merkle_tree: String,
    output: Option<PathBuf>,
    format: String,
    proof_system: String,
    config: &Config,
) -> Result<()> {
    let proof_system_lower = proof_system.to_lowercase();
    let use_plonk = proof_system_lower == "plonk";

    info!("Generating {} proof...", proof_system);

    let private_key_bytes =
        crate::crypto::read_private_key(private_key_opt, private_key_file, private_key_stdin)?;
    let mut private_key = [0u8; 32];
    private_key.copy_from_slice(&private_key_bytes);
    private_key_bytes.zeroize();

    let address = crate::crypto::derive_address(&private_key)
        .context("Failed to derive address from private key")?;

    println!("{} {}", "Address:".green(), address);
    println!("{} {}", "Recipient:".green(), recipient);
    println!("{} Proof System: {}", "Using:".blue().bold(), proof_system);

    let merkle_tree = match merkle_tree.as_str() {
        path if path.starts_with('.') || path.starts_with('/') => {
            crate::tree::MerkleTree::from_file(&PathBuf::from(path))?
                .context("Failed to load Merkle tree file")?
        }
        _ => anyhow::bail!("Merkle tree must be a valid file path"),
    };

    let pb = indicatif::ProgressBar::new(100);
    pb.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .progress_chars("=>-"),
    );
    pb.set_message("Deriving public key...");
    pb.set_position(10);

    let proof_data = if use_plonk {
        generate_plonk_proof(&private_key, &recipient, &merkle_tree)?
    } else {
        // Fall back to Groth16 (original implementation)
        crate::crypto::generate_nullifier(&private_key);
        // The rest would use the old proof generation logic
        // For now, return an error to use the original command
        anyhow::bail!("PLONK proof generation not fully implemented yet. Please use --proof-system groth16 for now, or complete Week 2 CLI integration tasks.")
    };

    private_key.zeroize();

    pb.finish_with_message("Proof generated!");

    let output_path = output.unwrap_or_else(|| PathBuf::from("proof.json"));
    let json =
        serde_json::to_string_pretty(&proof_data).context("Failed to serialize proof data")?;

    std::fs::write(&output_path, json).context("Failed to write proof file")?;

    println!("\n{} {}", "Proof saved to:".cyan(), output_path.display());
    println!("\n{}", proof_data);
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
