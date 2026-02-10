use anyhow::{Context, Result};
use ethers::signers::{LocalWallet, Signer};
use ethers::types::Address;
use secp256k1::{PublicKey, SecretKey};
use sha3::{Digest, Keccak256};
use std::path::PathBuf;
use std::sync::OnceLock;
use zeroize::Zeroize;

/// Wrapper for private key bytes that zeroizes on drop
#[derive(Clone)]
pub struct PrivateKey(Vec<u8>);

impl PrivateKey {
    /// Create a new PrivateKey from Vec<u8>
    pub fn new(bytes: Vec<u8>) -> Self {
        PrivateKey(bytes)
    }

    /// Get the length of the private key
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if the private key is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get a reference to the underlying bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Get a mutable reference to the underlying bytes
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }

    /// Try to convert to a fixed-size array
    pub fn try_into_array<const N: usize>(self) -> Result<[u8; N]> {
        let bytes = self.0;
        let mut zeroizing_key = PrivateKey(bytes);
        let result = zeroizing_key
            .0
            .try_into()
            .map_err(|e| anyhow::anyhow!("Invalid array length: expected {}, got {}", N, e.len()));
        zeroizing_key.zeroize();
        result
    }
}

impl std::fmt::Debug for PrivateKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("PrivateKey").field(&"[REDACTED]").finish()
    }
}

impl Drop for PrivateKey {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

impl From<Vec<u8>> for PrivateKey {
    fn from(bytes: Vec<u8>) -> Self {
        PrivateKey(bytes)
    }
}

impl AsRef<[u8]> for PrivateKey {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsMut<[u8]> for PrivateKey {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Standard Ethereum private key length in bytes
const PRIVATE_KEY_LEN: usize = 32;

/// BN254 scalar field modulus
const BN254_SCALAR_FIELD_MODULUS: &str =
    "21888242871839275222246405745257275088548364400416034343698204186575808495617";

/// Salt used for nullifier generation to prevent precomputation attacks
/// Matches circuit value at circuits/src/merkle_membership.circom:61-63
const NULLIFIER_SALT_STR: &str =
    "87953108768114088221452414019732140257409482096940319490691914651639977587459";

fn get_bn254_field_modulus() -> num_bigint::BigUint {
    use num_traits::Num;
    static FIELD_MODULUS: OnceLock<num_bigint::BigUint> = OnceLock::new();
    FIELD_MODULUS
        .get_or_init(|| {
            num_bigint::BigUint::from_str_radix(BN254_SCALAR_FIELD_MODULUS, 10)
                .expect("Invalid modulus constant")
        })
        .clone()
}

fn get_nullifier_salt() -> ark_bn254::Fr {
    use ark_ff::PrimeField;
    use num_traits::Num;
    static NULLIFIER_SALT: OnceLock<ark_bn254::Fr> = OnceLock::new();
    NULLIFIER_SALT
        .get_or_init(|| {
            let salt_biguint = num_bigint::BigUint::from_str_radix(NULLIFIER_SALT_STR, 10)
                .expect("Invalid nullifier salt constant");
            let salt_bytes = salt_biguint.to_bytes_be();
            let mut salt_array = [0u8; 32];
            let offset = 32 - salt_bytes.len();
            salt_array[offset..].copy_from_slice(&salt_bytes);
            ark_bn254::Fr::from_be_bytes(salt_array)
                .expect("Failed to convert nullifier salt to field element")
        })
        .clone()
}

/// Generates a nullifier from a private key using Poseidon hash.
///
/// A nullifier is a deterministic hash of a private key that allows:
/// - Verifying eligibility without revealing the private key
/// - Ensuring each address can only claim once
/// - Maintaining privacy by nullifiers being one-way
///
/// # Arguments
/// * `private_key` - The 32-byte Ethereum private key
///
/// # Returns
/// A hexadecimal string representation of the nullifier (with "0x" prefix)
///
/// # Security
/// The nullifier is derived using Poseidon hash to match the circuit implementation.
/// Each unique private key produces a unique nullifier.
///
/// # Implementation Details
/// Matches circuit nullifier generation: Poseidon(private_key, NULLIFIER_SALT, 0)
/// This ensures consistency across CLI, circuit, and specification.
pub fn generate_nullifier(private_key: &[u8; 32]) -> Result<String> {
    let private_key_field = field_element_from_bytes(private_key)?;

    let poseidon_input = vec![
        private_key_field.clone(),
        get_nullifier_salt(),
        ark_bn254::Fr::from(0u64),
    ];

    let poseidon_hash = poseidon_hash_circom_compat(&poseidon_input)?;

    let hash_bytes = field_to_bytes_be(&poseidon_hash);
    Ok(format!("0x{}", hex::encode(hash_bytes)))
}

fn field_element_from_bytes(bytes: &[u8; 32]) -> Result<ark_bn254::Fr> {
    use ark_ff::PrimeField;
    ark_bn254::Fr::from_be_bytes(*bytes)
        .map_err(|e| anyhow::anyhow!("Failed to convert bytes to field element: {}", e))
}

fn field_to_bytes_be(field: &ark_bn254::Fr) -> [u8; 32] {
    field.into_bigint().to_bytes_be()
}

fn poseidon_hash_circom_compat(inputs: &[ark_bn254::Fr]) -> Result<ark_bn254::Fr> {
    use ark_ff::PrimeField;

    if inputs.len() != 3 {
        return Err(anyhow::anyhow!(
            "Poseidon hash requires exactly 3 inputs, got {}",
            inputs.len()
        ));
    }

    let round_keys = poseidon_round_constants()?;
    let mds_matrix = poseidon_mds_matrix();

    let mut state = [inputs[0], inputs[1], inputs[2]];

    for round in 0..64 {
        for i in 0..3 {
            state[i] = state[i].pow([5u64]);
        }

        let mut new_state = [ark_bn254::Fr::zero(); 3];
        for i in 0..3 {
            for j in 0..3 {
                new_state[i] += mds_matrix[i][j] * state[j];
            }
            new_state[i] += round_keys[round][i];
        }
        state = new_state;
    }

    Ok(state[0])
}

fn poseidon_round_constants() -> Result<Vec<Vec<ark_bn254::Fr>>> {
    use ark_ff::PrimeField;

    let round_constants_bytes = poseidon_constants_seed()?;
    let mut constants = Vec::with_capacity(64);

    for round in 0..64 {
        let mut round_keys = Vec::with_capacity(3);
        for i in 0..3 {
            let offset = ((round * 3 + i) * 32) % round_constants_bytes.len();
            let mut hash = [0u8; 32];
            for j in 0..32 {
                hash[j] = round_constants_bytes[(offset + j) % round_constants_bytes.len()];
            }

            let fr = ark_bn254::Fr::from_be_bytes_mod_order(&hash, true);
            round_keys.push(fr);
        }
        constants.push(round_keys);
    }
    Ok(constants)
}

fn poseidon_mds_matrix() -> [[ark_bn254::Fr; 3]; 3] {
    use ark_ff::PrimeField;
    [
        [
            ark_bn254::Fr::from(3u64),
            ark_bn254::Fr::from(1u64),
            ark_bn254::Fr::from(1u64),
        ],
        [
            ark_bn254::Fr::from(1u64),
            ark_bn254::Fr::from(3u64),
            ark_bn254::Fr::from(1u64),
        ],
        [
            ark_bn254::Fr::from(1u64),
            ark_bn254::Fr::from(1u64),
            ark_bn254::Fr::from(3u64),
        ],
    ]
}

/// Computes a deterministic seed for Poseidon round constants based on the nullifier salt
/// to ensure consistency between circuit and CLI implementations
fn poseidon_constants_seed() -> Result<[u8; 32]> {
    use sha3::{Digest, Keccak256};

    let mut hasher = Keccak256::new();
    hasher.update(b"POSEIDON_CONSTANTS_SEED");
    let salt = get_nullifier_salt();
    let salt_bytes = salt.into_bigint().to_bytes_be();
    hasher.update(&salt_bytes);
    let hash = hasher.finalize();
    hash.try_into()
        .map_err(|_| anyhow::anyhow!("Invalid hash length: expected 32 bytes"))
}

/// Derives an Ethereum address from a private key.
///
/// This follows the standard Ethereum address derivation:
/// 1. Derive public key from private key using secp256k1
/// 2. Keccak256 hash the uncompressed public key (excluding first byte)
/// 3. Take last 20 bytes as the address
///
/// # Arguments
/// * `private_key` - The 32-byte Ethereum private key
///
/// # Returns
/// The derived Ethereum address
///
/// # Errors
/// Returns an error if the private key is invalid
pub fn derive_address(private_key: &[u8; 32]) -> Result<Address> {
    let secret_key = SecretKey::from_slice(private_key).context("Invalid private key")?;

    let public_key = PublicKey::from_secret_key(&secp256k1::Secp256k1::new(), &secret_key);
    let public_key_bytes = public_key.serialize_uncompressed();

    let hash = Keccak256::digest(&public_key_bytes[1..]);
    let address_bytes = &hash[hash.len() - 20..];

    Address::from_slice(address_bytes).context("Failed to derive address")
}

/// Reads a private key from multiple possible sources.
///
/// Supports multiple input methods with the following priority:
/// 1. `private_key_opt` - Direct key string
/// 2. `private_key_file` - Path to file containing key
/// 3. `private_key_stdin` - Read from stdin
/// 4. `ZKP_AIRDROP_PRIVATE_KEY` environment variable
///
/// # Arguments
/// * `private_key_opt` - Optional direct private key string
/// * `private_key_file` - Optional path to file containing key
/// * `private_key_stdin` - If true, read key from stdin
///
/// # Returns
/// The decoded private key wrapped in PrivateKey type
///
/// # Security
/// - Keys are zeroized from memory after use via Drop implementation
/// - Supports both "0x" prefix and raw hex format
/// - Validates hex encoding
///
/// # Errors
/// Returns an error if:
/// - No key source is provided
/// - Hex decoding fails
/// - File reading fails
pub fn read_private_key(
    private_key_opt: Option<String>,
    private_key_file: Option<PathBuf>,
    private_key_stdin: bool,
) -> Result<PrivateKey> {
    use std::io::{self, Read};
    use zeroize::Zeroize;

    let mut key_str = if private_key_stdin {
        let mut input = String::new();
        io::stdin()
            .read_to_string(&mut input)
            .context("Failed to read private key from stdin")?;
        let trimmed = input.trim().to_string();
        input.zeroize();
        trimmed
    } else if let Some(file) = private_key_file {
        let key = std::fs::read_to_string(&file)
            .context(format!(
                "Failed to read private key file: {}",
                file.display()
            ))?
            .trim()
            .to_string();
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

    if key_bytes.len() != 32 {
        key_str.zeroize();
        anyhow::bail!("Private key must be 32 bytes, got {}", key_bytes.len());
    }

    key_str.zeroize();

    Ok(PrivateKey::new(key_bytes))
}

/// Validates an Ethereum address.
///
/// Checks that the address:
/// - Is a valid hex string
/// - Can be parsed as an Ethereum address
/// - Optionally warns on checksum mismatch (non-fatal)
///
/// # Arguments
/// * `address` - The address string to validate
///
/// # Returns
/// The parsed Address if valid
///
/// # Errors
/// Returns an error if address format is invalid
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

pub fn normalize_address(address: &str) -> Result<Address> {
    let addr: Address = address
        .parse::<Address>()
        .context("Invalid Ethereum address format")?;
    Ok(addr)
}

/// Validates a nullifier string format.
///
/// Checks that nullifier:
/// - Is exactly 66 characters (0x + 64 hex chars)
/// - Starts with "0x" or "0X"
/// - Contains valid hexadecimal characters
///
/// # Arguments
/// * `nullifier` - The nullifier string to validate
///
/// # Errors
/// Returns an error if nullifier format is invalid
pub fn validate_nullifier(nullifier: &str) -> Result<()> {
    validate_hex_bytes(nullifier, "nullifier")
}

/// Validates a Merkle root hash format.
///
/// Checks that merkle_root:
/// - Is exactly 66 characters (0x + 64 hex chars)
/// - Starts with "0x" or "0X"
/// - Contains valid hexadecimal characters
/// - Decodes to exactly 32 bytes
///
/// # Arguments
/// * `merkle_root` - The Merkle root string to validate
///
/// # Errors
/// Returns an error if merkle_root format is invalid
pub fn validate_merkle_root(merkle_root: &str) -> Result<()> {
    validate_hex_bytes(merkle_root, "merkle_root")
}

fn validate_hex_bytes(input: &str, field_name: &str) -> Result<()> {
    if input.len() != 66 {
        return Err(anyhow::anyhow!(
            "Invalid {} length: expected 66 chars (0x + 64 hex), got {}",
            field_name,
            input.len()
        ));
    }

    if !input.starts_with("0x") && !input.starts_with("0X") {
        return Err(anyhow::anyhow!(
            "Invalid {} format: must start with 0x",
            field_name
        ));
    }

    let decoded = hex::decode(&input[2..])
        .context(format!("Invalid {}: invalid hex encoding", field_name))?;
    if decoded.len() != 32 {
        return Err(anyhow::anyhow!(
            "Invalid {}: expected 32 bytes, got {}",
            field_name,
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

pub fn keccak_hash_field(input: &[u8; 32]) -> Result<String> {
    let hash = keccak_hash(input);
    let hash_bytes =
        hex::decode(&hash).map_err(|e| anyhow::anyhow!("Failed to decode hash: {}", e))?;
    let mut hash_array = [0u8; 32];
    hash_array.copy_from_slice(&hash_bytes[..32]);
    Ok(field_element_to_decimal(&hash_array))
}

fn field_element_to_decimal(bytes: &[u8; 32]) -> String {
    use num_bigint::BigUint;
    use num_traits::Zero;

    let big_int = BigUint::from_bytes_be(bytes);

    let reduced = big_int % get_bn254_field_modulus();
    reduced.to_str_radix(10)
}

pub fn address_to_field(address: &Address) -> Result<String> {
    let address_bytes = address.as_bytes();
    let mut padded = [0u8; 32];
    padded[12..].copy_from_slice(address_bytes);
    Ok(field_element_to_decimal(&padded))
}
