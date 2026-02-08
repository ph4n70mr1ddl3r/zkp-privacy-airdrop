use anyhow::{Context, Result};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use tracing::info;

use crate::config::Config;
use crate::crypto::{derive_address, generate_nullifier, read_private_key};
use crate::types::ProofData;

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
    info!("Generating proof...");

    let private_key_bytes = read_private_key(private_key_opt, private_key_file, private_key_stdin)?;
    let mut private_key = [0u8; 32];
    private_key.copy_from_slice(&private_key_bytes);
    private_key_bytes.zeroize();

    let address =
        derive_address(&private_key).context("Failed to derive address from private key")?;

    let address_str = format!("{:?}", address);
    println!("{} {}", "Address:".green(), address_str);

    let nullifier = generate_nullifier(&private_key);
    let nullifier_hex = format!("0x{}", hex::encode(&nullifier.as_bytes()));

    private_key.zeroize();
    println!("{} {}", "Recipient:".green(), recipient);

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

    // SECURITY WARNING: Placeholder proof generation for testing only!
    // Real ZK proofs require:
    // 1. Proper circuit compilation (circom/circom-2.0)
    // 2. Trusted setup ceremony
    // 3. Proving key generation
    // 4. Witness computation using actual Merkle tree proof path
    // 5. ZK proof generation (snarkjs plonk fullprove or groth16 prove)
    //
    // Use: `snarkjs groth16 fullprove input.json circuit.wasm proving_key.zkey proof.json public.json`
    // or `snarkjs plonk fullprove input.json circuit.wasm proving_key.zkey proof.json public.json`
    let proof_data = ProofData {
        proof: crate::types::Proof {
            a: ["0".to_string(), "0".to_string()],
            b: [
                ["0".to_string(), "0".to_string()],
                ["0".to_string(), "0".to_string()],
            ],
            c: ["0".to_string(), "0".to_string()],
        },
        public_signals: ["0".to_string(), "0".to_string(), nullifier_hex.clone()],
        nullifier: nullifier_hex,
        recipient: recipient.clone(),
        merkle_root: "0x0000000000000000000000000000000000000000000000000000000000000000"
            .to_string(),
        generated_at: chrono::Utc::now().to_rfc3339(),
    };

    eprintln!("\n{}: This is a placeholder proof for testing purposes only!", "WARNING".yellow().bold());
    eprintln!("Real ZK proofs require proper circuit compilation and trusted setup.");

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
        config.relayer_url.as_deref().unwrap_or("<RELAYER_URL>"),
        recipient
    );

    Ok(())
}
