use anyhow::{Context, Result};
use colored::Colorize;
use reqwest::Client;
use std::path::PathBuf;
use tokio::time::{sleep, Duration};
use tracing::info;

use crate::config::Config;
use crate::crypto::{validate_address, validate_merkle_root, validate_nullifier};
use crate::types_plonk::{Proof, ProofData, SubmitClaimRequest, SubmitClaimResponse};

pub async fn execute(
    proof_path: PathBuf,
    relayer_url_opt: Option<String>,
    recipient: String,
    wait: bool,
    timeout: u64,
    config: &Config,
) -> Result<()> {
    info!("Submitting proof to relayer...");

    let relayer_url = relayer_url_opt.unwrap_or_else(|| {
        config
            .get_relayer_url()
            .unwrap_or_else(|_| "https://relayer.zkp-airdrop.io".to_string())
    });

    let recipient_addr = validate_address(&recipient).context("Invalid recipient address")?;

    let proof_content = std::fs::read_to_string(&proof_path)
        .with_context(|| format!("Failed to read proof file from {}", proof_path.display()))?;

    let proof_data: ProofData = serde_json::from_str(&proof_content).with_context(|| {
        format!(
            "Failed to parse proof JSON from file {}",
            proof_path.display()
        )
    })?;

    validate_proof_structure(&proof_data).context("Invalid proof structure")?;

    if proof_data.recipient.to_lowercase() != recipient.to_lowercase() {
        return Err(anyhow::anyhow!(
            "Proof recipient mismatch: proof is for {}, but submitting for {}",
            proof_data.recipient,
            recipient
        ));
    }

    validate_nullifier(&proof_data.nullifier).context("Invalid nullifier in proof file")?;

    validate_merkle_root(&proof_data.merkle_root).context("Invalid merkle_root in proof file")?;

    println!("{} {}", "Relayer:".cyan(), relayer_url);
    println!("{} {}", "Proof:".cyan(), proof_path.display());
    println!("{} {}", "Recipient:".cyan(), recipient);
    println!("{} {}", "Proof Type:".cyan(), proof_data.proof.type_name());
    println!(
        "{} {} bytes",
        "Proof Size:".cyan(),
        proof_data.proof.estimated_size_bytes()
    );

    let proof_type = proof_data.proof.type_name();

    let request = SubmitClaimRequest {
        proof: proof_data.proof,
        recipient: recipient.to_string(),
        nullifier: proof_data.nullifier,
        merkle_root: proof_data.merkle_root,
    };

    println!(
        "\n{} {} proof with {}-element structure",
        "Proof Type:".cyan(),
        proof_type,
        match proof_type {
            "Plonk" => "8-field",
            _ => "3-field",
        }
    );

    let client = Client::new();
    let url = format!("{}/api/v1/submit-claim", relayer_url);

    println!("\n{}", "Submitting claim...".yellow());

    let response = client
        .post(&url)
        .json(&request)
        .send()
        .await
        .context("Failed to send HTTP request to relayer")?;

    let status = response.status();
    let response_text = response
        .text()
        .await
        .context("Failed to read response body from relayer")?;

    let submit_response: SubmitClaimResponse =
        serde_json::from_str(&response_text).context("Failed to parse response JSON")?;

    if !status.is_success() {
        println!(
            "\n{} {}",
            "Error:".red(),
            submit_response
                .error
                .unwrap_or_else(|| "Unknown error".to_string())
        );

        if let Some(code) = &submit_response.code {
            match code.as_str() {
                "RATE_LIMITED" => {
                    if let Some(retry_after) = response.headers().get("Retry-After") {
                        if let Ok(seconds_str) = retry_after.to_str() {
                            if let Ok(secs) = seconds_str.parse::<u64>() {
                                const MAX_RETRY_AFTER: u64 = 86400;
                                if secs > MAX_RETRY_AFTER {
                                    tracing::warn!("Suspicious Retry-After value: {}", secs);
                                } else {
                                    println!("{} Try again in {} seconds.", "Note:".yellow(), secs);
                                }
                            }
                        }
                    }
                }
                "ALREADY_CLAIMED" => {
                    println!("{} This nullifier has already been used.", "Info:".blue());
                }
                "INVALID_PROOF" => {
                    println!("{} Please regenerate the proof.", "Info:".blue());
                    if proof_type == "Plonk" {
                        println!(
                            "{} PLONK proofs must use 8-field element structure.",
                            "Note:".yellow()
                        );
                    }
                }
                "INSUFFICIENT_FUNDS" => {
                    println!(
                        "{} The relayer is temporarily out of funds.",
                        "Info:".blue()
                    );
                    println!(
                        "{} Consider submitting directly to the contract or try another relayer.",
                        "Info:".blue()
                    );
                }
                "PLONK_FORMAT_ERROR" => {
                    println!("{} PLONK proof format is invalid.", "Error:".red());
                    println!(
                        "{} Expected 8 field elements: A, B, C, Z, T1, T2, T3, WXi",
                        "Info:".blue()
                    );
                }
                _ => {}
            }
        }
        return Ok(());
    }

    println!(
        "\n{} {}",
        "✓ Claim submitted successfully!".green(),
        submit_response.tx_hash.unwrap_or_else(|| "N/A".to_string())
    );

    if proof_type == "Plonk" {
        println!(
            "{} Note: PLONK verification uses ~1.3M gas (higher than Groth16)",
            "Info:".blue()
        );
    }

    if wait {
        if let Some(tx_hash) = submit_response.tx_hash {
            println!("\n{}", "Waiting for confirmation...".yellow());

            let rpc_url = config.get_rpc_url().context("Failed to get RPC URL")?;

            let start = std::time::Instant::now();
            let mut confirmed = false;

            while start.elapsed().as_secs() < timeout {
                sleep(Duration::from_secs(5)).await;
                print!(".");
                std::io::stdout().flush()?;

                if check_transaction_status(&rpc_url, &tx_hash).await {
                    confirmed = true;
                    break;
                }
            }
            println!();

            if confirmed {
                println!("{} {}", "✓".green(), "Transaction confirmed successfully!");
            } else {
                println!(
                    "\n{} {}",
                    "Timeout:".yellow(),
                    "Transaction not confirmed within timeout. Check manually:"
                );
                println!(
                    "  {}",
                    format!("https://optimism.etherscan.io/tx/{}", tx_hash).cyan()
                );
            }
        }
    }

    Ok(())
}

async fn check_transaction_status(rpc_url: &str, tx_hash: &str) -> bool {
    use ethers::providers::{Http, Provider};

    if let Ok(provider) = Provider::<Http>::try_from(rpc_url) {
        if let Ok(Some(_receipt)) = provider.get_transaction_receipt(tx_hash).await {
            return true;
        }
    }
    false
}

fn validate_proof_structure(proof_data: &ProofData) -> Result<()> {
    match &proof_data.proof {
        Proof::Plonk(plonk_proof) => {
            if plonk_proof.A.len() != 2 {
                return Err(anyhow::anyhow!(
                    "Invalid PLONK proof: A field must have 2 elements, found {}",
                    plonk_proof.A.len()
                ));
            }
            if plonk_proof.B.len() != 2
                || plonk_proof.B[0].len() != 2
                || plonk_proof.B[1].len() != 2
            {
                return Err(anyhow::anyhow!(
                    "Invalid PLONK proof: B field must be 2x2 array"
                ));
            }
            if plonk_proof.C.len() != 2 {
                return Err(anyhow::anyhow!(
                    "Invalid PLONK proof: C field must have 2 elements, found {}",
                    plonk_proof.C.len()
                ));
            }
            if plonk_proof.Z.len() != 3 {
                return Err(anyhow::anyhow!(
                    "Invalid PLONK proof: Z field must have 3 elements, found {}",
                    plonk_proof.Z.len()
                ));
            }
            if plonk_proof.T1.len() != 2 || plonk_proof.T2.len() != 2 || plonk_proof.T3.len() != 2 {
                return Err(anyhow::anyhow!(
                    "Invalid PLONK proof: T1, T2, T3 fields must each have 2 elements"
                ));
            }
            if plonk_proof.WXi.is_empty() {
                return Err(anyhow::anyhow!(
                    "Invalid PLONK proof: WXi field cannot be empty"
                ));
            }
        }
        Proof::Groth16(groth16_proof) => {
            if groth16_proof.a.len() != 2 {
                return Err(anyhow::anyhow!(
                    "Invalid Groth16 proof: a field must have 2 elements, found {}",
                    groth16_proof.a.len()
                ));
            }
            if groth16_proof.b.len() != 2
                || groth16_proof.b[0].len() != 2
                || groth16_proof.b[1].len() != 2
            {
                return Err(anyhow::anyhow!(
                    "Invalid Groth16 proof: b field must be 2x2 array"
                ));
            }
            if groth16_proof.c.len() != 2 {
                return Err(anyhow::anyhow!(
                    "Invalid Groth16 proof: c field must have 2 elements, found {}",
                    groth16_proof.c.len()
                ));
            }
        }
    }

    if proof_data.public_signals.len() != 3 {
        return Err(anyhow::anyhow!(
            "Invalid proof: public_signals must have 3 elements, found {}",
            proof_data.public_signals.len()
        ));
    }

    Ok(())
}
