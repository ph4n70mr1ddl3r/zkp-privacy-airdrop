use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;

use crate::config::Config;
use crate::ConfigAction;

pub fn execute(action: ConfigAction, config_path: &PathBuf, config: &Config) -> Result<()> {
    match action {
        ConfigAction::Show => show_config(config),
        ConfigAction::Set { key, value } => set_config(config_path, config, &key, &value),
        ConfigAction::Reset => reset_config(config_path),
    }
}

fn show_config(config: &Config) -> Result<()> {
    println!("{}", "Current Configuration:".green().bold());
    println!("  {} {}", "network:".cyan(), config.network);
    println!("  {} {}", "chain_id:".cyan(), config.chain_id);
    println!(
        "  {} {}",
        "relayer_url:".cyan(),
        config
            .relayer_url
            .as_ref()
            .unwrap_or(&"Not set".to_string())
    );
    println!(
        "  {} {}",
        "merkle_tree_source:".cyan(),
        config
            .merkle_tree_source
            .as_ref()
            .unwrap_or(&"Not set".to_string())
    );
    println!(
        "  {} {}",
        "rpc_url:".cyan(),
        config.rpc_url.as_ref().unwrap_or(&"Not set".to_string())
    );
    Ok(())
}

fn set_config(config_path: &PathBuf, config: &Config, key: &str, value: &str) -> Result<()> {
    println!("{} {} = {}", "Setting config:".cyan(), key, value);

    let mut new_config = config.clone();

    match key {
        "network" => {
            new_config.network = value.to_string();
            new_config.chain_id = match value {
                "optimism" => 10,
                "optimism-sepolia" => 11155420,
                _ => return Err(anyhow::anyhow!("Unknown network: {value}")),
            };
        }
        "relayer_url" => {
            new_config.relayer_url = Some(value.to_string());
        }
        "merkle_tree_source" => {
            new_config.merkle_tree_source = Some(value.to_string());
        }
        "rpc_url" => {
            new_config.rpc_url = Some(value.to_string());
        }
        _ => {
            return Err(anyhow::anyhow!(
                "Unknown config key: {key}. Valid keys: network, relayer_url, merkle_tree_source, rpc_url"
            ));
        }
    }

    new_config.save(config_path)?;
    println!("{} Configuration saved", "✓".green());
    Ok(())
}

fn reset_config(config_path: &PathBuf) -> Result<()> {
    println!("{}", "Resetting configuration to defaults...".yellow());

    let default_config = Config::default();
    default_config.save(config_path)?;

    println!("{} Configuration reset", "✓".green());
    Ok(())
}
