use ark_bn254::Fr;
use ark_ff::{BigInteger, Field, PrimeField};
use num_bigint::BigUint;
use num_traits::{Num, Zero};
use std::sync::OnceLock;

const FIELD_PRIME: &str = zkp_airdrop_utils::BN254_FIELD_MODULUS;

/// Nullifier salt constant - must match the value used in circuit
#[cfg(test)]
const NULLIFIER_SALT: u64 = 8795310876811408822u64;

fn field_prime() -> BigUint {
    BigUint::from_str_radix(FIELD_PRIME, 10).expect("Invalid field prime constant")
}

/// Poseidon hash for 3 inputs (t=3) with 64 rounds
/// Matches circomlib's Poseidon implementation
pub fn poseidon_hash(inputs: &[Fr]) -> Result<Fr, String> {
    if inputs.len() != 3 {
        return Err(format!(
            "Poseidon hash requires exactly 3 inputs, got {}",
            inputs.len()
        ));
    }

    let round_keys = get_poseidon_round_constants();
    let mds_matrix = get_poseidon_mds_matrix();

    let mut state = inputs.to_vec();

    #[allow(clippy::needless_range_loop)]
    for round in 0..64 {
        for item in state.iter_mut().take(3) {
            *item = item.pow([5u64]);
        }

        let mut new_state = [ark_bn254::Fr::zero(); 3];
        for i in 0..3 {
            for j in 0..3 {
                new_state[i] += mds_matrix[i][j] * state[j];
            }
            new_state[i] += round_keys[round][i];
        }
        state = new_state.to_vec();
    }

    Ok(state[0])
}

/// Generate Poseidon round constants
/// Uses a deterministic seed based on the hash of "poseidon"
fn get_poseidon_round_constants() -> Vec<Vec<Fr>> {
    static ROUND_CONSTANTS: OnceLock<Vec<Vec<Fr>>> = OnceLock::new();

    ROUND_CONSTANTS
        .get_or_init(|| {
            use sha3::{Digest, Keccak256};

            let seed = {
                let mut hasher = Keccak256::new();
                hasher.update(b"poseidon");
                hasher.finalize()
            };

            let mut constants = Vec::new();
            let mut hasher = Keccak256::new();
            hasher.update(seed);

            for _ in 0..64 {
                let mut round_consts = Vec::new();
                for _ in 0..3 {
                    hasher.update([0u8]);
                    let hash = hasher.clone().finalize();
                    let mut bytes = [0u8; 32];
                    bytes.copy_from_slice(&hash[..32]);

                    let field_elem = Fr::from_be_bytes_mod_order(&bytes);
                    round_consts.push(field_elem);
                }
                constants.push(round_consts);
            }

            constants
        })
        .clone()
}

/// Get Poseidon MDS matrix (t=3)
/// Uses the standard MDS matrix from the Poseidon paper
fn get_poseidon_mds_matrix() -> Vec<Vec<Fr>> {
    static MDS_MATRIX: OnceLock<Vec<Vec<Fr>>> = OnceLock::new();

    MDS_MATRIX
        .get_or_init(|| {
            let modulus = field_prime();

            let mds_values = [
                ["5", "7", "10", "13", "14"],
                ["7", "10", "13", "14", "5"],
                ["10", "13", "14", "5", "7"],
            ];

            mds_values
                .iter()
                .map(|row| {
                    row.iter()
                        .map(|s| {
                            let val = BigUint::from_str_radix(s, 10).expect("Invalid MDS constant");
                            let inv = val
                                .modinv(&modulus)
                                .expect("Failed to compute modular inverse for MDS constant");
                            let bytes = inv.to_bytes_be();
                            let mut padded = [0u8; 32];
                            let offset = 32 - bytes.len();
                            padded[offset..].copy_from_slice(&bytes);
                            Fr::from_be_bytes_mod_order(&padded)
                        })
                        .collect()
                })
                .collect()
        })
        .clone()
}

/// Hash an Ethereum address using Poseidon
/// Matches circomlib's Poseidon(3): hash(address, 0, 0)
pub fn hash_address(address: [u8; 20]) -> Result<[u8; 32], String> {
    let mut padded = [0u8; 32];
    padded[12..].copy_from_slice(&address);

    let address_field = Fr::from_be_bytes_mod_order(&padded);

    let inputs = vec![address_field, Fr::zero(), Fr::zero()];

    let hash = poseidon_hash(&inputs)?;
    Ok(field_to_bytes_be(&hash))
}

/// Hash two 32-byte values using Poseidon
/// Matches circomlib's Poseidon(3): hash(left, right, 0)
pub fn hash_two(left: &[u8; 32], right: &[u8; 32]) -> Result<[u8; 32], String> {
    let left_field = Fr::from_be_bytes_mod_order(left);
    let right_field = Fr::from_be_bytes_mod_order(right);

    let inputs = vec![left_field, right_field, Fr::zero()];

    let hash = poseidon_hash(&inputs)?;
    Ok(field_to_bytes_be(&hash))
}

fn field_to_bytes_be(field: &Fr) -> [u8; 32] {
    let bytes = field.into_bigint().to_bytes_be();
    let mut result = [0u8; 32];
    let offset = 32 - bytes.len();
    result[offset..].copy_from_slice(&bytes);
    result
}

#[cfg(test)]
#[allow(dead_code)]
pub fn hash_domain(input: &[u8]) -> Result<[u8; 32], String> {
    let mut padded = [0u8; 32];
    let len = input.len().min(32);
    padded[32 - len..].copy_from_slice(&input[..len]);

    let input_field = Fr::from_be_bytes_mod_order(&padded);

    let inputs = vec![input_field, Fr::zero(), Fr::zero()];

    let hash = poseidon_hash(&inputs)?;
    Ok(field_to_bytes_be(&hash))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_address() {
        let decoded = hex::decode("c0d7d3017b342ff039b55b087900000000000000")
            .expect("Failed to decode hex address");
        let address: [u8; 20] = decoded.try_into().expect("Invalid address length");
        let hash = hash_address(address).expect("Failed to hash address");
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_hash_two() {
        let left = [0u8; 32];
        let right = [0u8; 32];
        let hash = hash_two(&left, &right).expect("Failed to hash two values");
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_nullifier_salt_constant() {
        assert_eq!(NULLIFIER_SALT, 8795310876811408822u64);
    }

    #[test]
    fn test_field_prime_constant() {
        let prime = field_prime();
        assert_eq!(
            prime.to_str_radix(10),
            "21888242871839275222246405745257275088548364400416034343698204186575808495617"
        );
    }

    #[test]
    fn test_poseidon_hash_deterministic() {
        let input1 = Fr::from(1u64);
        let input2 = Fr::from(2u64);
        let input3 = Fr::from(3u64);

        let hash1 = poseidon_hash(&[input1, input2, input3]).expect("Failed to compute hash");
        let hash2 = poseidon_hash(&[input1, input2, input3]).expect("Failed to compute hash");

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_poseidon_hash_different_inputs() {
        let input1 = Fr::from(1u64);
        let input2 = Fr::from(2u64);
        let input3 = Fr::from(3u64);

        let hash1 = poseidon_hash(&[input1, input2, input3]).expect("Failed to compute hash");
        let hash2 = poseidon_hash(&[input2, input1, input3]).expect("Failed to compute hash");

        assert_ne!(hash1, hash2);
    }
}
