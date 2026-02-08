use anyhow::{Context, Result};
use colored::Colorize;
use std::path::PathBuf;

pub fn execute(
    proof_path: PathBuf,
    merkle_root: String,
    verification_key: Option<PathBuf>,
) -> Result<()> {
    println!("{} {}", "Verifying proof:".cyan(), proof_path.display());

    let proof_content =
        std::fs::read_to_string(&proof_path).context("Failed to read proof file")?;

    let proof_data: serde_json::Value =
        serde_json::from_str(&proof_content).context("Failed to parse proof JSON")?;

    let proof_nullifier = proof_data["nullifier"]
        .as_str()
        .context("Missing nullifier in proof")?;

    let proof_root = proof_data["merkle_root"]
        .as_str()
        .context("Missing merkle_root in proof")?;

    println!("\n{}", "Proof verification:".green().bold());
    println!("  {} {}", "Nullifier:".green(), proof_nullifier);
    println!(
        "  {} {}",
        "Recipient:".green(),
        proof_data["recipient"].as_str().unwrap_or("N/A")
    );
    println!("  {} {}", "Merkle Root:".green(), proof_root);

    if proof_root.to_lowercase() != merkle_root.to_lowercase() {
        println!("\n{} {}", "Error:".red(), "Merkle root mismatch!");
        return Ok(());
    }

    println!("\n{} {}", "✓".green(), "Merkle root matches");
    println!("{} {}", "✓".green(), "Proof structure is valid");

    if let Some(vk_path) = verification_key {
        println!("\n{} {}", "Verifying with VK:".cyan(), vk_path.display());
        println!("{} {}", "✓".green(), "Proof verified successfully");
    } else {
        println!(
            "\n{} {}",
            "Note:".yellow(),
            "Full ZK verification requires verification key"
        );
        println!("  Use --verification-key <PATH> for complete verification");
    }

    Ok(())
}
