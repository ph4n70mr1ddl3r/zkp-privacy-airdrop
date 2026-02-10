// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "./BasePrivacyAirdrop.sol";

/**
 * @title IVerifier
 * @notice Interface for Groth16 proof verification
 */
interface IVerifier {
    /**
     * @notice Verify a Groth16 zero-knowledge proof
     * @param _pA First proof value (2 elements)
     * @param _pB Second proof value (2x2 matrix)
     * @param _pC Third proof value (2 elements)
     * @param _pubSignals Public signals (3 elements: merkle_root, recipient, nullifier)
     * @return True if proof is valid
     */
    function verifyProof(
        uint[2] calldata _pA,
        uint[2][2] calldata _pB,
        uint[2] calldata _pC,
        uint[3] calldata _pubSignals
    ) external view returns (bool);
}

/**
 * @title PrivacyAirdrop
 * @notice Privacy-preserving ERC20 token airdrop using Groth16 ZK proofs
 * @dev Allows users to claim tokens without revealing their address from the Merkle tree
 * Inherits from BasePrivacyAirdrop to share common functionality with PrivacyAirdropPLONK
 */
contract PrivacyAirdrop is BasePrivacyAirdrop {
    uint256 private constant GROTH16_GAS_ESTIMATE = 700_000;
    IVerifier public immutable verifier;

    struct Proof {
        uint[2] a;
        uint[2][2] b;
        uint[2] c;
    }

    /**
     * @notice Initialize the airdrop contract
     * @param _token Address of the ERC20 token to distribute
     * @param _merkleRoot Root of the Merkle tree containing eligible addresses
     * @param _claimAmount Number of tokens each eligible address can claim
     * @param _claimDeadline Unix timestamp after which claims are no longer accepted
     * @param _verifier Address of the Groth16 verifier contract
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
    ) BasePrivacyAirdrop(_token, _merkleRoot, _claimAmount, _claimDeadline, _maxWithdrawalPercent, _withdrawalCooldown) {
        require(_verifier != address(0), "Invalid verifier address");
        verifier = IVerifier(_verifier);

        bytes32 zeroRoot = 0x0000000000000000000000000000000000000000000000000000000000000000;
        bytes32 onesRoot = 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff;
        require(_merkleRoot != zeroRoot && _merkleRoot != onesRoot, "Invalid merkle root: cannot be all zeros or all ones");
    }

    /**
     * @notice Claim tokens by presenting a zero-knowledge proof
     * @param proof Groth16 proof of Merkle tree membership
     * @param nullifier Unique identifier derived from private key (prevents double-claims)
     * @param recipient Address to receive the claimed tokens
     * @dev Emits Claimed event on successful claim
     */
    function claim(
        Proof calldata proof,
        bytes32 nullifier,
        address recipient
    ) external nonReentrant {
        require(!paused, "Contract is paused");
        require(block.timestamp <= claimDeadline, "Claim period ended");
        require(recipient != address(0), "Invalid recipient");
        require(nullifier != bytes32(0), "Invalid nullifier");
        require(!nullifiers[nullifier], "Already claimed");

        uint[3] memory publicSignals;
        publicSignals[0] = uint256(merkleRoot);
        publicSignals[1] = uint160(recipient);
        publicSignals[2] = uint256(nullifier);

        require(verifier.verifyProof(proof.a, proof.b, proof.c, publicSignals), "Invalid proof");

        nullifiers[nullifier] = true;

        emit Claimed(nullifier, recipient, block.timestamp);

        _transferTokens(recipient, claimAmount);
    }

    /**
     * @notice Estimate gas required for a claim transaction
     * @dev This returns a conservative estimate. In production, consider using gasleft()
     *      to measure actual gas consumption or estimate dynamically based on current gas prices.
     *      The actual gas cost depends on gas price and network congestion.
     * @return Estimated gas in wei (conservative 700K with buffer for Groth16 verification)
     */
    function estimateClaimGas() external pure returns (uint256) {
        return GROTH16_GAS_ESTIMATE;
    }
}
