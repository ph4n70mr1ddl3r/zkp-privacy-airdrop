use anyhow::{Context, Result};
use colored::Colorize;
use reqwest::Client;
use std::path::PathBuf;
use tracing::info;
use tokio::time::{sleep, Duration};

use crate::config::Config;
use crate::crypto::{validate_address, validate_nullifier};
use crate::types_plonk::{ProofData, SubmitClaimRequest, SubmitClaimResponse, Proof};

pub async fn execute(
    proof_path: PathBuf,
    relayer_url_opt: Option<String>,
    recipient: String,
    wait: bool,
    timeout: u64,
    config: &Config,
) -> Result<()> {
    info!("Submitting proof to relayer...");
    
    let relayer_url = relayer_url_opt
        .unwrap_or_else(|| config.get_relayer_url().unwrap_or_else(|_| "https://relayer.zkp-airdrop.io".to_string()));
    
    let recipient_addr = validate_address(&recipient)
        .context("Invalid recipient address")?;
    
    let proof_content = std::fs::read_to_string(&proof_path)
        .context("Failed to read proof file")?;
    
    let proof_data: ProofData = serde_json::from_str(&proof_content)
        .context("Failed to parse proof JSON")?;
    
    if proof_data.recipient.to_lowercase() != recipient.to_lowercase() {
        return Err(anyhow::anyhow!(
            "Proof recipient mismatch: proof is for {}, but submitting for {}",
            proof_data.recipient,
            recipient
        ));
    }

    validate_nullifier(&proof_data.nullifier)
        .context("Invalid nullifier in proof file")?;
    
    println!("{} {}", "Relayer:".cyan(), relayer_url);
    println!("{} {}", "Proof:".cyan(), proof_path.display());
    println!("{} {}", "Recipient:".cyan(), recipient);
    println!("{} {}", "Proof Type:".cyan(), proof_data.proof.type_name());
    println!("{} {} bytes", "Proof Size:".cyan(), proof_data.proof.estimated_size_bytes());
    
    let proof_type = proof_data.proof.type_name();
    
    let request = SubmitClaimRequest {
        proof: proof_data.proof,
        recipient: recipient.to_string(),
        nullifier: proof_data.nullifier,
        merkle_root: proof_data.merkle_root,
    };
    
    println!("\n{} {} proof with {}-element structure", 
             "Proof Type:".cyan(), proof_type, 
             match proof_type {
                 "Plonk" => "8-field",
                 _ => "3-field"
             });
    
    let client = Client::new();
    let url = format!("{}/api/v1/submit-claim", relayer_url);
    
    println!("\n{}", "Submitting claim...".yellow());
    
    let response = client
        .post(&url)
        .json(&request)
        .send()
        .await
        .context("Failed to submit claim to relayer")?;
    
    let status = response.status();
    let response_text = response.text().await?;
    
    let submit_response: SubmitClaimResponse = serde_json::from_str(&response_text)
        .context("Failed to parse response")?;
    
    if !status.is_success() {
        println!("\n{} {}", "Error:".red(), 
                 submit_response.error.unwrap_or_else(|| "Unknown error".to_string()));
        
        if let Some(code) = &submit_response.code {
            match code.as_str() {
                "RATE_LIMITED" => {
                    if let Some(retry_after) = response.headers().get("Retry-After") {
                        if let Ok(seconds_str) = retry_after.to_str() {
                            if let Ok(seconds) = seconds_str.parse::<u64>() {
                                println!("{} Try again in {} seconds.", "Note:".yellow(), seconds);
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
                        println!("{} PLONK proofs must use 8-field element structure.", "Note:".yellow());
                    }
                }
                "INSUFFICIENT_FUNDS" => {
                    println!("{} The relayer is temporarily out of funds.", "Info:".blue());
                    println!("{} Consider submitting directly to the contract or try another relayer.", "Info:".blue());
                }
                "PLONK_FORMAT_ERROR" => {
                    println!("{} PLONK proof format is invalid.", "Error:".red());
                    println!("{} Expected 8 field elements: A, B, C, Z, T1, T2, T3, WXi", "Info:".blue());
                }
                _ => {}
            }
        }
        return Ok(());
    }
    
    println!("\n{} {}", "✓ Claim submitted successfully!".green(), 
             submit_response.tx_hash.unwrap_or_else(|| "N/A".to_string()));
    
    if proof_type == "Plonk" {
        println!("{} Note: PLONK verification uses ~1.3M gas (higher than Groth16)", "Info:".blue());
    }
    
    if wait {
        if let Some(tx_hash) = submit_response.tx_hash {
            println!("\n{}", "Waiting for confirmation...".yellow());
            
            let start = std::time::Instant::now();
            loop {
                if start.elapsed().as_secs() > timeout {
                    println!("\n{} {}", "Timeout:".yellow(), 
                             "Transaction not confirmed within timeout");
                    break;
                }
                
                sleep(Duration::from_secs(5)).await;
                print!(".");
                std::io::stdout().flush()?;
                
                // In a real implementation, check transaction status here
                break;
            }
            println!();
        }
    }
    
    Ok(())
}
