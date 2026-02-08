// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title Privacy Airdrop with PLONK Verification
 * @notice ZKP Privacy Airdrop contract using PLONK proofs
 * @dev Uses Perpetual Powers of Tau - no trusted setup ceremony required
 */
abstract contract ReentrancyGuard {
    uint256 private constant _NOT_ENTERED = 1;
    uint256 private constant _ENTERED = 2;
    uint256 private _status;

    constructor() {
        _status = _NOT_ENTERED;
    }

    modifier nonReentrant() {
        require(_status != _ENTERED, "ReentrancyGuard: reentrant call");
        _status = _ENTERED;
        _;
        _status = _NOT_ENTERED;
    }
}

/**
 * @title PrivacyAirdropPLONK
 * @notice Privacy-preserving ERC20 token airdrop using PLONK ZK proofs
 * @dev Allows users to claim tokens without revealing their address from the Merkle tree
 * Uses universal trusted setup (Perpetual Powers of Tau) instead of per-circuit trusted setup
 */
contract PrivacyAirdropPLONK {
    bytes32 public immutable merkleRoot;
    mapping(bytes32 => bool) public nullifiers;
    address public immutable token;
    uint256 public immutable claimAmount;
    uint256 public immutable claimDeadline;
    IPLONKVerifier public immutable verifier;

    event Claimed(bytes32 indexed nullifier, address indexed recipient, uint256 timestamp);

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
        verifier = IPLONKVerifier(_verifier);
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
    ) external nonReentrant {
        require(block.timestamp < claimDeadline, "Claim period ended");
        require(recipient != address(0), "Invalid recipient");
        require(!nullifiers[nullifier], "Already claimed");

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

        (bool success, bytes memory data) = address(token).call(
            abi.encodeWithSelector(IERC20.transfer.selector, recipient, claimAmount)
        );
        require(success && (data.length == 0 || abi.decode(data, (bool))), "Token transfer failed");

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
     * @notice Estimate gas required for a PLONK claim transaction
     * @dev PLONK verification requires more gas than Groth16 (~900K vs ~300K)
     * @param proof PLONK proof (unused in estimate, kept for interface)
     * @param nullifier Nullifier hash (unused in estimate, kept for interface)
     * @param recipient Recipient address (unused in estimate, kept for interface)
     * @return Estimated gas in wei (conservative 1.3M with buffer)
     */
    function estimateClaimGas(
        PLONKProof calldata proof,
        bytes32 nullifier,
        address recipient
    ) external view returns (uint256) {
        proof;
        nullifier;
        recipient;
        return 1_300_000;
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
