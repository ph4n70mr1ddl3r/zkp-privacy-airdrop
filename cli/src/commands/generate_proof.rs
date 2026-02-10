use anyhow::{Context, Result};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use tracing::info;

use crate::config::Config;
use crate::crypto::{derive_address, generate_nullifier, read_private_key};
use crate::types_plonk::{Proof, ProofData};

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

    let tree = crate::tree::MerkleTree::load_from_csv(&merkle_tree)
        .await
        .context("Failed to load Merkle tree")?;

    let merkle_root = tree.root.clone();
    let merkle_root_hex = format!("0x{}", hex::encode(&merkle_root));

    return Err(anyhow::anyhow!(
        "PLONK proof generation is not yet implemented. \
         Please use the Groth16 proof system or set up PLONK proving infrastructure as documented in PLONK-README.md.\n\n\
         To enable PLONK proof generation:\n\
         1) Compile the circom circuit to generate PLONK proving key (.ptau)\n\
         2) Integrate with a PLONK proving library (snarkjs or arkworks-plonk)\n\
         3) Use the proving key to generate proofs from witness computation"
    ));

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
        config.relayer_url.as_deref().unwrap_or("<RELAYER_URL>"),
        recipient
    );

    Ok(())
}
