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
const MAX_SUBMITS_PER_WINDOW: u32 = 10;

static LAST_SUBMIT_TIME: AtomicU64 = AtomicU64::new(0);
static SUBMIT_COUNT: AtomicU32 = AtomicU32::new(0);

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
            .filter(|c| c.is_alphanumeric() || c.is_ascii_punctuation() || c.is_whitespace())
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
    let current_ms = now.elapsed().as_millis() as u64;
    let window_ms = SUBMIT_RATE_LIMIT_WINDOW.as_millis() as u64;

    loop {
        let last_time_ms = LAST_SUBMIT_TIME.load(Ordering::Acquire);
        let elapsed_ms = if last_time_ms == 0 {
            window_ms
        } else {
            current_ms.saturating_sub(last_time_ms)
        };

        if elapsed_ms >= window_ms {
            let mut new_count = 1u32;
            match LAST_SUBMIT_TIME.compare_exchange(
                last_time_ms,
                current_ms,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    SUBMIT_COUNT.store(new_count, Ordering::Release);
                    break;
                }
                Err(_) => continue,
            }
        } else {
            let mut count = SUBMIT_COUNT.load(Ordering::Acquire);
            loop {
                if count >= MAX_SUBMITS_PER_WINDOW {
                    let wait_time =
                        SUBMIT_RATE_LIMIT_WINDOW.saturating_sub(Duration::from_millis(elapsed_ms));
                    println!(
                        "{} Rate limit exceeded. Please wait {} seconds before submitting again.",
                        "Warning:".yellow(),
                        wait_time.as_secs()
                    );
                    return Err(anyhow::anyhow!("Rate limit exceeded"));
                }
                match SUBMIT_COUNT.compare_exchange_weak(
                    count,
                    count + 1,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                ) {
                    Ok(_) => break,
                    Err(new_count) => count = new_count,
                }
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
        Proof::Groth16(groth16_proof) => {
            if groth16_proof.a.is_empty() || groth16_proof.a.len() != 2 {
                return Err(anyhow::anyhow!(
                    "Invalid Groth16 proof: a field must have 2 elements, found {}",
                    groth16_proof.a.len()
                ));
            }
            if groth16_proof.b.is_empty()
                || groth16_proof.b.len() != 2
                || groth16_proof.b[0].len() != 2
                || groth16_proof.b[1].len() != 2
            {
                return Err(anyhow::anyhow!(
                    "Invalid Groth16 proof: b field must be 2x2 array"
                ));
            }
            if groth16_proof.c.is_empty() || groth16_proof.c.len() != 2 {
                return Err(anyhow::anyhow!(
                    "Invalid Groth16 proof: c field must have 2 elements, found {}",
                    groth16_proof.c.len()
                ));
            }

            for element in groth16_proof.a.iter().chain(groth16_proof.c.iter()) {
                if element.is_empty() {
                    return Err(anyhow::anyhow!(
                        "Invalid Groth16 proof: found empty element in proof fields"
                    ));
                }
                if !element.starts_with("0x") {
                    return Err(anyhow::anyhow!(
                        "Invalid Groth16 proof: proof elements must be hex strings starting with 0x"
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
