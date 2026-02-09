// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";

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
 *
 * TODO: Consider refactoring to use a base contract pattern with PrivacyAirdropPLONK.sol
 * to reduce code duplication. Both contracts share similar logic for constructor validation,
 * claim checks, and token transfer. The main difference is the proof verification step
 * (Groth16 vs PLONK).
 */
contract PrivacyAirdrop is ReentrancyGuard {
    using SafeERC20 for IERC20;
    
    bytes32 public immutable merkleRoot;
    mapping(bytes32 => bool) public nullifiers;
    address public immutable token;
    uint256 public immutable claimAmount;
    uint256 public immutable claimDeadline;
    IVerifier public immutable verifier;

    event Claimed(bytes32 indexed nullifier, address indexed recipient, uint256 timestamp);

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
     */
    constructor(
        address _token,
        bytes32 _merkleRoot,
        uint256 _claimAmount,
        uint256 _claimDeadline,
        address _verifier
    ) {
        require(_token != address(0), "Invalid token address");
        require(_merkleRoot != bytes32(0), "Invalid merkle root");
        require(_claimAmount > 0, "Invalid claim amount");
        require(_claimDeadline > block.timestamp, "Invalid deadline");
        require(_verifier != address(0), "Invalid verifier address");
        token = _token;
        merkleRoot = _merkleRoot;
        claimAmount = _claimAmount;
        claimDeadline = _claimDeadline;
        verifier = IVerifier(_verifier);
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
        require(block.timestamp <= claimDeadline, "Claim period ended");
        require(recipient != address(0), "Invalid recipient");
        require(!nullifiers[nullifier], "Already claimed");

        uint[3] memory publicSignals = [
            uint256(merkleRoot),
            uint256(uint160(recipient)),
            uint256(nullifier)
        ];

        require(verifier.verifyProof(proof.a, proof.b, proof.c, publicSignals), "Invalid proof");

        nullifiers[nullifier] = true;

        IERC20(token).safeTransfer(recipient, claimAmount);

        emit Claimed(nullifier, recipient, block.timestamp);
    }

    /**
     * @notice Check if a nullifier has already been claimed
     * @param nullifier The nullifier to check
     * @return True if the nullifier has already been used
     */
    function isClaimed(bytes32 nullifier) external view returns (bool) {
        return nullifiers[nullifier];
    }

    /**
     * @notice Estimate gas required for a claim transaction
     * @dev This returns a conservative estimate. In production, consider using gasleft()
     *      to measure actual gas consumption or estimate dynamically based on current gas prices.
     *      The actual gas cost depends on gas price and network congestion.
     * @return Estimated gas in wei (conservative 700K with buffer for Groth16 verification)
     */
    function estimateClaimGas() external pure returns (uint256) {
        return 700_000;
    }
}

/**
 * @title IERC20
 * @notice Interface for ERC20 token transfers
 */
interface IERC20 {
    /**
     * @notice Transfer tokens from contract to recipient
     * @param to Recipient address
     * @param amount Number of tokens to transfer
     * @return True if transfer successful
     */
    function transfer(address to, uint256 amount) external returns (bool);
}
