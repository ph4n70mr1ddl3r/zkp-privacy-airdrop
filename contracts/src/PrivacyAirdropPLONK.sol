// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {BasePrivacyAirdrop} from "./BasePrivacyAirdrop.sol";

/**
 * @title PrivacyAirdropPLONK
 * @notice Privacy-preserving ERC20 token airdrop using PLONK ZK proofs
 * @dev Allows users to claim tokens without revealing their address from Merkle tree
 * Uses universal trusted setup (Perpetual Powers of Tau) instead of per-circuit trusted setup
 * Inherits from BasePrivacyAirdrop to share common functionality with PrivacyAirdrop
 *
 * IMPORTANT: The PLONK verifier contract must contain the proper verification key
 * before deploying to production. See PLONK-README.md for verification key generation steps.
 */
contract PrivacyAirdropPLONK is BasePrivacyAirdrop {
    error InvalidVerifierAddress();
    error InvalidPLONKProofLength();
    error InvalidPLONKProofZero();
    error InvalidPLONKProofOverflow();
    error InvalidPLONKProofAllZeros();
    error InvalidPLONKProofUniform();
    error PLONKProofVerificationFailed();

    IPLONKVerifier public immutable VERIFIER;
    uint256 private constant BN254_FIELD_PRIME =
        21888242871839275222246405745257275088548364400416034343698204186575808495617;

    /**
     * @notice PLONK proof structure
     * @dev Contains 8 field elements: A, B, C, Z, T1, T2, T3, WXi
     */
    struct PLONKProof {
        uint256[8] proof;
    }

    /**
     * @notice Initialize the PLONK airdrop contract
     * @param _token Address of the ERC20 token to distribute
     * @param _merkleRoot Root of the Merkle tree containing eligible addresses
     * @param _claimAmount Number of tokens each eligible address can claim
     * @param _claimDeadline Unix timestamp after which claims are no longer accepted
     * @param _verifier Address of the PLONK verifier contract
     * @param _maxWithdrawalPercent Maximum percentage of unclaimed tokens that can be withdrawn per period (default 10)
     * @param _withdrawalCooldown Time in seconds between withdrawal periods (default 24 hours)
     */
    constructor(
        address _token,
        bytes32 _merkleRoot,
        uint256 _claimAmount,
        uint256 _claimDeadline,
        address _verifier,
        uint256 _maxWithdrawalPercent,
        uint256 _withdrawalCooldown
    ) BasePrivacyAirdrop(
        _token,
        _merkleRoot,
        _claimAmount,
        _claimDeadline,
        _maxWithdrawalPercent,
        _withdrawalCooldown
    ) {
        if (_verifier == address(0)) {
            revert InvalidVerifierAddress();
        }
        VERIFIER = IPLONKVerifier(_verifier);
    }



    /**
     * @notice Claim tokens by presenting a PLONK zero-knowledge proof
     * @param proof PLONK proof of Merkle tree membership (8 field elements)
     * @param nullifier Unique identifier derived from private key (prevents double-claims)
     * @param recipient Address to receive the claimed tokens
     */
    function claim(
        PLONKProof calldata proof,
        bytes32 nullifier,
        address recipient
    ) external nonReentrant validClaim(recipient, nullifier) {
        _validatePLONKProof(proof);

        uint256[3] memory instances;
        instances[0] = uint256(MERKLE_ROOT);
        instances[1] = uint256(uint160(recipient));
        instances[2] = uint256(nullifier);

        if (!VERIFIER.verifyProof(proof.proof, instances)) {
            revert PLONKProofVerificationFailed();
        }

        _transferTokens(recipient, CLAIM_AMOUNT);

        nullifiers[nullifier] = true;

        // solhint-disable not-rely-on-time
        emit Claimed(nullifier, recipient, block.timestamp);
        // solhint-enable not-rely-on-time
    }

    function _validatePLONKProof(PLONKProof calldata proof) private pure {
        if (proof.proof.length != 8) {
            revert InvalidPLONKProofLength();
        }

        uint256 allZeros;
        uint256 firstValue = proof.proof[0];
        bool allSame = true;

        for (uint256 i = 0; i < 8; i++) {
            if (proof.proof[i] == 0) {
                revert InvalidPLONKProofZero();
            }
            if (proof.proof[i] >= BN254_FIELD_PRIME) {
                revert InvalidPLONKProofOverflow();
            }

            allZeros |= proof.proof[i];

            if (i > 0 && proof.proof[i] != firstValue) {
                allSame = false;
            }
        }

        if (allZeros == 0) {
            revert InvalidPLONKProofAllZeros();
        }

        if (allSame) {
            revert InvalidPLONKProofUniform();
        }
    }

/**
 * @notice Estimate gas required for a PLONK claim transaction
 * @dev PLONK verification requires more gas than Groth16
 * @return Estimated gas in wei (conservative 1.3M with buffer)
 */
function estimateClaimGas() external pure returns (uint256) {
    return 1_300_000;
}
}

/**
 * @title PLONK Verifier Interface
 * @notice Interface for PLONK proof verification
 */
interface IPLONKVerifier {
    /**
     * @notice Verify PLONK proof
     * @param _proof PLONK proof (8 field elements)
     * @param _instances Public inputs (3 field elements: merkle_root, recipient, nullifier)
     * @return True if proof is valid
     */
    function verifyProof(
        uint256[8] calldata _proof,
        uint256[3] calldata _instances
    ) external view returns (bool);

    /**
     * @notice Get number of public inputs
     * @return Number of public inputs (3 for merkle_root, recipient, nullifier)
     */
    function getInstanceCount() external view returns (uint256);

    /**
     * @notice Get number of proof elements
     * @return Number of proof elements (8 for PLONK)
     */
    function getProofElementCount() external view returns (uint256);
}
