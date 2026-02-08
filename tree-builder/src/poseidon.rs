use num_bigint::BigUint;
use num_traits::Num;
use num_traits::Zero;
use sha2::{Digest, Sha256};

const FIELD_PRIME: &str =
    "21888242871839275222246405745257275088548364400416034343698204186575808495617";
const FIELD_PRIME_BIG: BigUint = BigUint::from_str_radix(FIELD_PRIME, 10).unwrap();

pub fn hash_address(address: [u8; 20]) -> [u8; 32] {
    let mut padded = [0u8; 32];
    padded[12..].copy_from_slice(&address);

    let mut hasher = Sha256::new();
    hasher.update(&padded);
    let result = hasher.finalize();

    mod_field(&result)
}

pub fn hash_two(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(left);
    hasher.update(right);
    let result = hasher.finalize();

    mod_field(&result)
}

pub fn hash_domain(input: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(input);
    let result = hasher.finalize();

    mod_field(&result)
}

fn mod_field(bytes: &[u8; 32]) -> [u8; 32] {
    let value = BigUint::from_bytes_be(bytes);
    let reduced = &value % &FIELD_PRIME_BIG;
    let mut result = [0u8; 32];
    reduced.to_bytes_be(&mut result);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_address() {
        let address = hex::decode("0000000000000000c0d7d3017b342ff039b55b0879")
            .unwrap()
            .try_into()
            .unwrap();
        let hash = hash_address(address);
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_hash_two() {
        let left = [0u8; 32];
        let right = [0u8; 32];
        let hash = hash_two(&left, &right);
        assert_eq!(hash.len(), 32);
    }
}
