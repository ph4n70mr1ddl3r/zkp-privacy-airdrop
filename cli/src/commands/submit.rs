use anyhow::{Context, Result};
use colored::Colorize;
use reqwest::Client;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::info;

use crate::config::Config;
use crate::crypto::{validate_address, validate_merkle_root, validate_nullifier};
use crate::types_plonk::{Proof, ProofData, SubmitClaimRequest, SubmitClaimResponse};

const HTTP_TIMEOUT_SECONDS: u64 = 30;
const MAX_RETRY_AFTER_SECONDS: u64 = 86400;
const TRANSACTION_CHECK_INTERVAL_SECONDS: u64 = 5;
const SUBMIT_RATE_LIMIT_WINDOW: Duration = Duration::from_secs(60);
const MAX_PROOF_SIZE_BYTES: usize = 10 * 1024 * 1024;

struct RateLimitState {
    window_start: AtomicU64,
    count: AtomicU32,
}

static RATE_LIMIT_STATE: RateLimitState = RateLimitState {
    window_start: AtomicU64::new(0),
    count: AtomicU32::new(0),
};

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

    if proof_content.len() > MAX_PROOF_SIZE_BYTES {
        proof_content.zeroize();
        return Err(anyhow::anyhow!(
            "Proof file too large: {} bytes (max: {})",
            proof_content.len(),
            MAX_PROOF_SIZE_BYTES
        ));
    }

    use zeroize::Zeroize;

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

    let proof_type = proof_data.proof.type_name();

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

    let now = Instant::now();
    let current_secs = now.elapsed().as_secs();
    let window_secs = SUBMIT_RATE_LIMIT_WINDOW.as_secs();

    loop {
        let window_start = RATE_LIMIT_STATE.window_start.load(Ordering::Acquire);

        if window_start == 0 || current_secs.saturating_sub(window_start) >= window_secs {
            let mut new_count = 1u32;
            match RATE_LIMIT_STATE.window_start.compare_exchange(
                window_start,
                current_secs,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    RATE_LIMIT_STATE.count.store(new_count, Ordering::Release);
                    break;
                }
                Err(_) => continue,
            }
        } else {
            let count = RATE_LIMIT_STATE.count.fetch_add(1, Ordering::AcqRel);
            if count >= config.max_submits_per_window {
                let elapsed = current_secs.saturating_sub(window_start);
                let wait_time =
                    SUBMIT_RATE_LIMIT_WINDOW.saturating_sub(Duration::from_secs(elapsed));
                println!(
                    "{} Rate limit exceeded. Please wait {} seconds before submitting again.",
                    "Warning:".yellow(),
                    wait_time.as_secs()
                );
                return Err(anyhow::anyhow!("Rate limit exceeded"));
            }
            break;
        }
    }

    let client = Client::builder()
        .timeout(Duration::from_secs(HTTP_TIMEOUT_SECONDS))
        .build()
        .context("Failed to create HTTP client")?;
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
        let error_msg = submit_response
            .error
            .unwrap_or_else(|| "Unknown error".to_string());
        println!("\n{} {}", "Error:".red(), error_msg);

        if let Some(code) = &submit_response.code {
            match code.as_str() {
                "RATE_LIMITED" => {
                    use reqwest::header::RETRY_AFTER;
                    if let Some(retry_after) = response.headers().get(RETRY_AFTER) {
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
        submit_response.tx_hash.unwrap_or_else(|| "N/A".to_string())
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
                println!("{} {}", "✓".green(), "Transaction confirmed successfully!");
            } else {
                println!(
                    "\n{} {}",
                    "Timeout:".yellow(),
                    "Transaction not confirmed within timeout. Check manually:"
                );
                let explorer_url = match config.network.as_str() {
                    "optimism" => "https://optimism.etherscan.io",
                    "optimism-sepolia" => "https://sepolia-optimism.etherscan.io",
                    _ => "https://optimism.etherscan.io",
                };
                println!("  {}", format!("{}/tx/{}", explorer_url, tx_hash).cyan());
            }
        }
    }

    Ok(())
}

async fn check_transaction_status(rpc_url: &str, tx_hash: &str) -> bool {
    use ethers::providers::{Http, Provider};

    if let Ok(provider) = Provider::<Http>::try_from(rpc_url) {
        match provider.get_transaction_receipt(tx_hash).await {
            Ok(Some(_receipt)) => true,
            Ok(None) => false,
            Err(e) => {
                tracing::warn!("Failed to check transaction status for {}: {}", tx_hash, e);
                false
            }
        }
    } else {
        false
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
                        "Invalid Plonk proof: element at index {} is empty",
                        idx
                    ));
                }
                if !element.starts_with("0x") {
                    return Err(anyhow::anyhow!(
                        "Invalid Plonk proof: element at index {} must be hex string starting with 0x",
                        idx
                    ));
                }
            }
        }
        Proof::Groth16(_) => {
            return Err(anyhow::anyhow!(
                "Groth16 proofs are no longer supported. Please use PLONK proofs."
            ));
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
                "Invalid proof: public_signal at index {} is empty",
                idx
            ));
        }
        if !signal.starts_with("0x") {
            return Err(anyhow::anyhow!(
                "Invalid proof: public_signal at index {} must be hex string starting with 0x",
                idx
            ));
        }
    }

    Ok(())
}
