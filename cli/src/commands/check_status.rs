use anyhow::{Context, Result};
use colored::Colorize;
use reqwest::Client;

use crate::config::Config;
use crate::types::CheckStatusResponse;

pub async fn execute(
    nullifier: String,
    relayer_url_opt: Option<String>,
    rpc_url_opt: Option<String>,
    config: &Config,
) -> Result<()> {
    if let Some(rpc_url) = rpc_url_opt {
        check_status_on_chain(&nullifier, &rpc_url).await
    } else {
        check_status_via_relayer(&nullifier, relayer_url_opt, config).await
    }
}

async fn check_status_via_relayer(
    nullifier: &str,
    relayer_url_opt: Option<String>,
    config: &Config,
) -> Result<()> {
    let relayer_url = relayer_url_opt
        .unwrap_or_else(|| config.get_relayer_url().unwrap_or_else(|_| "https://relayer.zkp-airdrop.io".to_string()));
    
    println!("{} {}", "Checking status via relayer:".cyan(), relayer_url);
    println!("{} {}", "Nullifier:".cyan(), nullifier);
    
    let client = Client::new();
    let url = format!("{}/api/v1/check-status/{}", relayer_url, nullifier);
    
    let response = client
        .get(&url)
        .send()
        .await
        .context("Failed to check status")?;
    
    let status: CheckStatusResponse = response.json().await
        .context("Failed to parse response")?;
    
    println!("\n{}", "Claim Status:".green().bold());
    
    if status.claimed {
        println!("  {} {}", "Status:".green(), "Claimed");
        println!("  {} {}", "Recipient:".green(), status.recipient.unwrap_or_else(|| "N/A".to_string()));
        println!("  {} {}", "Transaction:".green(), status.tx_hash.unwrap_or_else(|| "N/A".to_string()));
        if let Some(block) = status.block_number {
            println!("  {} {}", "Block:".green(), block);
        }
        if let Some(timestamp) = status.timestamp {
            println!("  {} {}", "Timestamp:".green(), timestamp);
        }
    } else {
        println!("  {} {}", "Status:".yellow(), "Not claimed");
        println!("\n{}", "You can submit a claim using:".yellow());
        println!("  zkp-airdrop submit --proof <PROOF> --relayer-url {}", relayer_url);
    }
    
    Ok(())
}

async fn check_status_on_chain(
    nullifier: &str,
    rpc_url: &str,
) -> Result<()> {
    println!("{} {}", "Checking status on-chain:".cyan(), rpc_url);
    println!("{} {}", "Nullifier:".cyan(), nullifier);
    println!("\n{}", "Note:".yellow());
    println!("  On-chain checking requires contract ABI and provider.");
    println!("  For now, please use the relayer API to check status.");
    
    Ok(())
}
