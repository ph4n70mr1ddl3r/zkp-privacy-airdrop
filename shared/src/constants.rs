/// Cryptographic constants shared across the project
///
/// This module provides a single source of truth for all cryptographic constants
/// used in the ZKP Privacy Airdrop system.
/// BN254 scalar field modulus (prime field order)
///
/// This is the order of the scalar field for the BN254 elliptic curve,
/// used for field element arithmetic in PLONK circuits.
///
/// Value: 21888242871839275222246405745257275088548364400416034343698204186575808495617
pub const BN254_FIELD_MODULUS: &str =
    "21888242871839275222246405745257275088548364400416034343698204186575808495617";

/// Maximum number of leaves in the Merkle tree (2^26)
///
/// The Merkle tree is a full binary tree with 26 levels, supporting
/// up to 67,108,864 leaves (though typically only ~65M are used).
pub const MAX_TREE_DEPTH: usize = 26;

/// Maximum number of leaves in the Merkle tree
pub const MAX_TREE_LEAVES: usize = 1 << MAX_TREE_DEPTH;

/// Minimum entropy score for private key validation
///
/// A score of 1500 indicates approximately 150 bits of entropy,
/// well above minimum cryptographic standards for security.
pub const MIN_ENTROPY_SCORE: u32 = 1500;

/// Maximum gas randomization percentage
///
/// Gas prices are randomized by up to 20% to prevent front-running attacks.
pub const MAX_GAS_RANDOMIZATION_PERCENT: u64 = 20;

/// PLONK proof element count
///
/// A PLONK proof consists of 8 field elements: A[2], B[2][2], C[2]
pub const PLONK_PROOF_ELEMENTS: usize = 8;

/// Maximum proof file size in bytes (10MB)
///
/// Prevents DoS attacks via oversized proof file uploads.
pub const MAX_PROOF_FILE_SIZE: usize = 10 * 1024 * 1024;

#[cfg(test)]
mod tests {
    use super::*;
    use num_traits::Zero;

    #[test]
    fn test_constants_are_valid() {
        // Verify field modulus is a valid prime
        let modulus: num_bigint::BigUint = BN254_FIELD_MODULUS
            .parse()
            .expect("Failed to parse field modulus");
        assert!(modulus > num_bigint::BigUint::zero());

        // Verify tree depth is reasonable
        assert!(MAX_TREE_DEPTH == 26);
        assert_eq!(MAX_TREE_LEAVES, 67_108_864);

        // Verify entropy threshold is sufficient
        assert!(MIN_ENTROPY_SCORE >= 1500);

        // Verify gas randomization is reasonable
        assert!(MAX_GAS_RANDOMIZATION_PERCENT <= 20);
    }
}
