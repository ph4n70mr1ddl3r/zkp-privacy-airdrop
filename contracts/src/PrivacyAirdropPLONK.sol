// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "./BasePrivacyAirdrop.sol";

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
    uint256 private constant PLONK_GAS_ESTIMATE = 1_300_000;
    IPLONKVerifier public immutable verifier;

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
     */
    constructor(
        address _token,
        bytes32 _merkleRoot,
        uint256 _claimAmount,
        uint256 _claimDeadline,
        address _verifier
    ) BasePrivacyAirdrop(_token, _merkleRoot, _claimAmount, _claimDeadline) {
        require(_verifier != address(0), "Invalid verifier address");
        verifier = IPLONKVerifier(_verifier);

        bytes32 zeroRoot = 0x0000000000000000000000000000000000000000000000000000000000000000;
        bytes32 onesRoot = 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff;
        require(_merkleRoot != zeroRoot && _merkleRoot != onesRoot, "Invalid merkle root: cannot be all zeros or all ones");
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

        uint256[3] memory instances = [
            uint256(merkleRoot),
            uint256(uint160(recipient)),
            uint256(nullifier)
        ];

        require(
            verifier.verifyProof(proof.proof, instances),
            "Invalid proof"
        );

        nullifiers[nullifier] = true;

        _transferTokens(recipient, claimAmount);

        emit Claimed(nullifier, recipient, block.timestamp);
    }

    /**
     * @notice Estimate gas required for a PLONK claim transaction
     * @dev PLONK verification requires more gas than Groth16 (~900K vs ~300K)
     * @return Estimated gas in wei (conservative 1.3M with buffer)
     */
    function estimateClaimGas() external pure returns (uint256) {
        return PLONK_GAS_ESTIMATE;
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
