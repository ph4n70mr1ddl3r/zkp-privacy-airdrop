use anyhow::{Context, Result};
use colored::Colorize;
use ethers::providers::Middleware;
use reqwest::Client;
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;
use tracing::info;
use url::Url;
use zeroize::Zeroize;

use crate::config::Config;
use crate::crypto::{validate_address, validate_merkle_root, validate_nullifier};
use crate::types_plonk::{Proof, ProofData, SubmitClaimRequest, SubmitClaimResponse};

const HTTP_TIMEOUT_SECONDS: u64 = 30;
const MAX_RETRY_AFTER_SECONDS: u64 = 86400;
const TRANSACTION_CHECK_INTERVAL_SECONDS: u64 = 5;

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

    let parsed_url = Url::parse(&relayer_url)
        .with_context(|| format!("Invalid relayer URL format: {relayer_url}"))?;

    if !["http", "https"].contains(&parsed_url.scheme()) {
        return Err(anyhow::anyhow!(
            "Relayer URL must use http or https scheme, got: {}",
            parsed_url.scheme()
        ));
    }

    if parsed_url
        .host_str()
        .is_some_and(|h| h.contains("localhost"))
    {
        tracing::warn!("Connecting to localhost - ensure this is intentional");
    }

    if relayer_url.len() > 2048 {
        return Err(anyhow::anyhow!(
            "Invalid relayer URL: exceeds maximum length of 2048 characters"
        ));
    }

    let _recipient_addr = validate_address(&recipient).context("Invalid recipient address")?;

    let mut proof_content =
        std::fs::read_to_string(&proof_path).context("Failed to read proof file")?;

    let proof_data: ProofData = serde_json::from_str(&proof_content).with_context(|| {
        format!(
            "Failed to parse proof JSON from file {}",
            proof_path.display()
        )
    })?;

    proof_content.zeroize();

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

    println!("{} {}", "Relayer:".cyan(), sanitize_output(&relayer_url));
    println!("{} {}", "Proof:".cyan(), proof_path.display());
    println!("{} {}", "Recipient:".cyan(), sanitize_output(&recipient));
    println!("{} {}", "Proof Type:".cyan(), proof_data.proof.type_name());
    println!(
        "{} {} bytes",
        "Proof Size:".cyan(),
        proof_data.proof.estimated_size_bytes()
    );

    let proof_type = proof_data.proof.type_name().to_string();

    fn sanitize_output(input: &str) -> String {
        input
            .chars()
            .filter(|c| {
                c.is_ascii_alphanumeric() || *c == '-' || *c == '_' || *c == '.' || *c == ':'
            })
            .take(50)
            .collect::<String>()
    }

    let request = SubmitClaimRequest {
        proof: proof_data.proof,
        recipient: recipient.clone(),
        nullifier: proof_data.nullifier,
        merkle_root: proof_data.merkle_root,
    };

    println!(
        "{} {} proof with {}-element structure",
        "Proof Type:".cyan(),
        proof_type,
        match proof_type.as_str() {
            "Plonk" => "8-field",
            _ => "3-field",
        }
    );

    let client = Client::builder()
        .timeout(Duration::from_secs(HTTP_TIMEOUT_SECONDS))
        .build()
        .context("Failed to create HTTP client")?;
    let url = format!("{relayer_url}/api/v1/submit-claim");

    println!("\n{}", "Submitting claim...".yellow());

    let response = client
        .post(&url)
        .json(&request)
        .send()
        .await
        .context("Failed to send HTTP request to relayer")?;

    let status = response.status();
    let headers = response.headers().clone();
    let response_text = response
        .text()
        .await
        .context("Failed to read response body from relayer")?;

    let submit_response: SubmitClaimResponse =
        serde_json::from_str(&response_text).context("Failed to parse response JSON")?;

    if !status.is_success() {
        let error_msg = submit_response
            .error
            .unwrap_or_else(|| "Unknown error".to_string());
        println!("\n{} {}", "Error:".red(), error_msg);

        if let Some(code) = &submit_response.code {
            match code.as_str() {
                "RATE_LIMITED" => {
                    use reqwest::header::RETRY_AFTER;
                    if let Some(retry_after) = headers.get(RETRY_AFTER) {
                        if let Ok(seconds_str) = retry_after.to_str() {
                            if let Ok(secs) = seconds_str.parse::<u64>() {
                                if secs > MAX_RETRY_AFTER_SECONDS {
                                    tracing::warn!("Suspicious Retry-After value: {}", secs);
                                } else {
                                    println!("{} Try again in {} seconds.", "Note:".yellow(), secs);
                                }
                            } else {
                                tracing::warn!(
                                    "Failed to parse Retry-After header: {}",
                                    seconds_str
                                );
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
                            "{} Plonk proofs must use flat array structure with at least 8 field elements.",
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
                    println!("{} Plonk proof format is invalid.", "Error:".red());
                    println!(
                        "{} Expected at least 8 field elements in flat array format.",
                        "Info:".blue()
                    );
                }
                _ => {
                    tracing::warn!("Unhandled error code: {}", code);
                }
            }
        }
        return Err(anyhow::anyhow!(
            "Relayer returned error: {} (code: {:?})",
            error_msg,
            submit_response.code
        ));
    }

    println!(
        "\n{} {}",
        "✓ Claim submitted successfully!".green(),
        submit_response.tx_hash.as_deref().unwrap_or("N/A")
    );

    if proof_type == "Plonk" {
        println!(
            "{} Note: Plonk verification uses ~1.3M gas (higher than Groth16)",
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
                sleep(Duration::from_secs(TRANSACTION_CHECK_INTERVAL_SECONDS)).await;
                print!(".");
                std::io::stdout().flush()?;

                if check_transaction_status(&rpc_url, &tx_hash).await {
                    confirmed = true;
                    break;
                }
            }
            println!();

            if confirmed {
                println!("{} Transaction confirmed successfully!", "✓".green());
            } else {
                println!(
                    "\n{} Transaction not confirmed within timeout. Check manually:",
                    "Timeout:".yellow()
                );
                let explorer_url = match config.network.as_str() {
                    "optimism" => "https://optimism.etherscan.io",
                    "optimism-sepolia" => "https://sepolia-optimism.etherscan.io",
                    _ => "https://optimism.etherscan.io",
                };

                tracing::info!(
                    "Transaction submitted on network: {}",
                    config.network.as_str()
                );
                println!("  {}", format!("{explorer_url}/tx/{tx_hash}").cyan());
            }
        }
    }

    Ok(())
}

async fn check_transaction_status(rpc_url: &str, tx_hash: &str) -> bool {
    use ethers::providers::{Http, Provider};
    use ethers::types::H256;

    let tx_hash_parsed: H256 = tx_hash.parse().unwrap_or_else(|e| {
        tracing::warn!("Invalid transaction hash format: {}, using default", e);
        H256::default()
    });

    match Provider::<Http>::try_from(rpc_url) {
        Ok(provider) => match provider.get_transaction_receipt(tx_hash_parsed).await {
            Ok(Some(_receipt)) => true,
            Ok(None) => false,
            Err(e) => {
                tracing::warn!("Failed to check transaction status for {}: {}", tx_hash, e);
                false
            }
        },
        Err(e) => {
            tracing::warn!("Failed to create RPC provider from {}: {}", rpc_url, e);
            false
        }
    }
}

fn validate_proof_structure(proof_data: &ProofData) -> Result<()> {
    match &proof_data.proof {
        Proof::Plonk(plonk_proof) => {
            if plonk_proof.proof.is_empty() {
                return Err(anyhow::anyhow!("Invalid Plonk proof: proof array is empty"));
            }
            if plonk_proof.proof.len() < 8 {
                return Err(anyhow::anyhow!(
                    "Invalid Plonk proof: must have at least 8 elements, found {}",
                    plonk_proof.proof.len()
                ));
            }

            for (idx, element) in plonk_proof.proof.iter().enumerate() {
                if element.is_empty() {
                    return Err(anyhow::anyhow!(
                        "Invalid Plonk proof: element at index {idx} is empty"
                    ));
                }
                if !element.starts_with("0x") {
                    return Err(anyhow::anyhow!(
                        "Invalid Plonk proof: element at index {idx} must be hex string starting with 0x"
                    ));
                }
            }
        }
    }

    if proof_data.public_signals.is_empty() {
        return Err(anyhow::anyhow!(
            "Invalid proof: public_signals array is empty"
        ));
    }

    if proof_data.public_signals.len() != 3 {
        return Err(anyhow::anyhow!(
            "Invalid proof: public_signals must have 3 elements, found {}",
            proof_data.public_signals.len()
        ));
    }

    for (idx, signal) in proof_data.public_signals.iter().enumerate() {
        if signal.is_empty() {
            return Err(anyhow::anyhow!(
                "Invalid proof: public_signal at index {idx} is empty"
            ));
        }
        if !signal.starts_with("0x") {
            return Err(anyhow::anyhow!(
                "Invalid proof: public_signal at index {idx} must be hex string starting with 0x"
            ));
        }
    }

    Ok(())
}
