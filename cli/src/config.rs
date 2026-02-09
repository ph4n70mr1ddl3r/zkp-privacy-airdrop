use anyhow::{Context, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use toml;
use zeroize::Zeroize;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub network: String,
    pub relayer_url: Option<String>,
    pub merkle_tree_source: Option<String>,
    pub rpc_url: Option<String>,
    pub chain_id: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            network: std::env::var("ZKP_NETWORK")
                .unwrap_or_else(|_| "optimism".to_string()),
            relayer_url: std::env::var("ZKP_RELAYER_URL").ok()
                .or_else(|| Some("https://relayer.zkp-airdrop.io".to_string())),
            merkle_tree_source: std::env::var("ZKP_MERKLE_TREE_SOURCE").ok(),
            rpc_url: std::env::var("ZKP_RPC_URL").ok(),
            chain_id: std::env::var("ZKP_CHAIN_ID")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
        }
    }
}

impl Config {
    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();

        if path.exists() {
            let content = fs::read_to_string(path).context("Failed to read config file")?;
            let config: Config = toml::from_str(&content).context("Failed to parse config file")?;
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let toml_str = toml::to_string_pretty(self).context("Failed to serialize config")?;

        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent).context("Failed to create config directory")?;
        }

        fs::write(path, toml_str).context("Failed to write config file")?;
        Ok(())
    }

    pub fn get_relayer_url(&self) -> Result<String> {
        self.relayer_url.clone().context(
            "No relayer URL configured. Set it with: zkp-airdrop config set relayer_url <URL>",
        )
    }

    pub fn get_rpc_url(&self) -> Result<String> {
        if let Some(url) = &self.rpc_url {
            Ok(url.clone())
        } else {
            Ok(self.get_default_rpc_url()?)
        }
    }

    pub fn get_default_rpc_url(&self) -> Result<String> {
        if let Ok(url) = std::env::var("ZKP_RPC_URL") {
            return Ok(url);
        }

        match self.network.as_str() {
            "optimism" => {
                Ok(std::env::var("ZKP_OPTIMISM_RPC_URL")
                    .unwrap_or_else(|_| "https://optimism.drpc.org".to_string()))
            }
            "optimism-sepolia" => {
                Ok(std::env::var("ZKP_OPTIMISM_SEPOLIA_RPC_URL")
                    .unwrap_or_else(|_| "https://sepolia.drpc.org/ogrpc".to_string()))
            }
            _ => Err(anyhow::anyhow!("Unsupported network: {}", self.network)),
        }
    }
}
