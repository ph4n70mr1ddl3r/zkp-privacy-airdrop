use anyhow::{Context, Result};
use colored::Colorize;
use ethers::signers::{LocalWallet, Signer};
use ethers::types::Address;
use secp256k1::{PublicKey, SecretKey};
use sha3::{Digest, Keccak256};
use std::path::PathBuf;

use crate::types::ProofData;

const NULLIFIER_DOMAIN_SEPARATOR: &[u8] = b"zkp_airdrop_nullifier_v1";
const DOMAIN_SEPARATOR_LEN: usize = NULLIFIER_DOMAIN_SEPARATOR.len();
const PRIVATE_KEY_LEN: usize = 32;
const NULLIFIER_PADDING_LEN: usize = 41;
const NULLIFIER_INPUT_LEN: usize = 96;

pub fn generate_nullifier(private_key: &[u8; 32]) -> Result<String> {
    let mut nullifier_input = Vec::with_capacity(NULLIFIER_INPUT_LEN);
    nullifier_input.extend_from_slice(NULLIFIER_DOMAIN_SEPARATOR);
    nullifier_input.extend_from_slice(private_key);
    nullifier_input.extend_from_slice(&[0u8; NULLIFIER_PADDING_LEN]);

    if nullifier_input.len() != NULLIFIER_INPUT_LEN {
        return Err(anyhow::anyhow!(
            "Nullifier input length mismatch: expected {}, got {}",
            NULLIFIER_INPUT_LEN,
            nullifier_input.len()
        ));
    }

    Ok(keccak_hash(&nullifier_input))
}

pub fn derive_address(private_key: &[u8; 32]) -> Result<Address> {
    let secret_key = SecretKey::from_slice(private_key).context("Invalid private key")?;

    let public_key = PublicKey::from_secret_key(&secp256k1::Secp256k1::new(), &secret_key);
    let public_key_bytes = public_key.serialize_uncompressed();

    let hash = Keccak256::digest(&public_key_bytes[1..]);
    let address_bytes = &hash[hash.len() - 20..];

    Address::from_slice(address_bytes).context("Failed to derive address")
}

pub fn read_private_key(
    private_key_opt: Option<String>,
    private_key_file: Option<PathBuf>,
    private_key_stdin: bool,
) -> Result<Vec<u8>> {
    use std::io::{self, Read};
    use zeroize::Zeroize;

    let mut key_str = if private_key_stdin {
        let mut input = String::new();
        io::stdin().read_to_string(&mut input)?;
        let trimmed = input.trim().to_string();
        input.zeroize();
        trimmed
    } else if let Some(file) = private_key_file {
        let key = std::fs::read_to_string(file)?.trim().to_string();
        let mut key_bytes = key.into_bytes();
        let key_str = String::from_utf8_lossy(&key_bytes).to_string();
        key_bytes.zeroize();
        key_str
    } else if let Some(key) = private_key_opt {
        key
    } else if let Ok(key) = std::env::var("ZKP_AIRDROP_PRIVATE_KEY") {
        key
    } else {
        anyhow::bail!(
            "No private key provided. Use one of:\n\
             --private-key <KEY>\n\
             --private-key-file <PATH>\n\
             --private-key-stdin\n\
             ZKP_AIRDROP_PRIVATE_KEY environment variable"
        );
    };

    let key_bytes = if key_str.starts_with("0x") || key_str.starts_with("0X") {
        hex::decode(&key_str[2..])
    } else {
        hex::decode(&key_str)
    }
    .context("Invalid hex private key")?;

    key_str.zeroize();

    if key_bytes.len() != 32 {
        anyhow::bail!("Private key must be 32 bytes, got {}", key_bytes.len());
    }

    Ok(key_bytes)
}

pub fn validate_address(address: &str) -> Result<Address> {
    let addr: Address = address
        .parse::<Address>()
        .context("Invalid Ethereum address format")?;

    let expected = format!("{:#x}", addr);
    if !address.eq_ignore_ascii_case(&expected) {
        tracing::warn!(
            "Address checksum mismatch: provided={}, expected={}",
            address,
            expected
        );
    }

    Ok(addr)
}

pub fn validate_nullifier(nullifier: &str) -> Result<()> {
    if nullifier.len() != 66 {
        return Err(anyhow::anyhow!(
            "Invalid nullifier length: expected 66 chars (0x + 64 hex), got {}",
            nullifier.len()
        ));
    }

    if !nullifier.starts_with("0x") && !nullifier.starts_with("0X") {
        return Err(anyhow::anyhow!(
            "Invalid nullifier format: must start with 0x"
        ));
    }

    hex::decode(&nullifier[2..]).context("Invalid nullifier: invalid hex encoding")?;

    Ok(())
}

pub fn validate_merkle_root(merkle_root: &str) -> Result<()> {
    if merkle_root.len() != 66 {
        return Err(anyhow::anyhow!(
            "Invalid merkle_root length: expected 66 chars (0x + 64 hex), got {}",
            merkle_root.len()
        ));
    }

    if !merkle_root.starts_with("0x") && !merkle_root.starts_with("0X") {
        return Err(anyhow::anyhow!(
            "Invalid merkle_root format: must start with 0x"
        ));
    }

    let decoded =
        hex::decode(&merkle_root[2..]).context("Invalid merkle_root: invalid hex encoding")?;
    if decoded.len() != 32 {
        return Err(anyhow::anyhow!(
            "Invalid merkle_root: expected 32 bytes, got {}",
            decoded.len()
        ));
    }

    Ok(())
}

fn keccak_hash(input: &[u8]) -> String {
    let mut hasher = Keccak256::new();
    hasher.update(input);
    let result = hasher.finalize();
    hex::encode(result)
}

pub fn poseidon_hash_field(input: &[u8; 32]) -> Result<String> {
    let hash = keccak_hash(input);
    let hash_bytes =
        hex::decode(&hash).map_err(|e| anyhow::anyhow!("Failed to decode hash: {}", e))?;
    let mut hash_array = [0u8; 32];
    hash_array.copy_from_slice(&hash_bytes[..32]);
    Ok(field_element_to_decimal(&hash_array))
}

const BN254_SCALAR_FIELD_MODULUS: &str =
    "21888242871839275222246405745257275088548364400416034343698204186575808495617";

fn field_element_to_decimal(bytes: &[u8; 32]) -> String {
    use num_bigint::BigUint;
    use num_traits::{Num, Zero};

    let big_int = BigUint::from_bytes_be(bytes);

    let modulus =
        BigUint::from_str_radix(BN254_SCALAR_FIELD_MODULUS, 10).expect("Invalid modulus constant");

    let reduced = big_int % modulus;
    reduced.to_str_radix(10)
}

pub fn address_to_field(address: &Address) -> Result<String> {
    let address_bytes = address.as_bytes();
    let mut padded = [0u8; 32];
    padded[12..].copy_from_slice(address_bytes);
    Ok(field_element_to_decimal(&padded))
}
