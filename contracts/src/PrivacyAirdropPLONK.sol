// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title Privacy Airdrop with PLONK Verification
 * @notice ZKP Privacy Airdrop contract using PLONK proofs
 * @dev Uses Perpetual Powers of Tau - no trusted setup ceremony required
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
     * @dev Different from Groth16 - has 8 elements instead of 3
     */
    struct PLONKProof {
        uint256[8] proof;  // A, B, C, Z, T1, T2, T3, WXi...
    }
    
    /**
     * @notice Initialize contract
     * @param _token Address of ZKP token contract
     * @param _merkleRoot Merkle tree root
     * @param _claimAmount Tokens per claim (1000 ZKP)
     * @param _claimDeadline Unix timestamp for end of claim period
     * @param _verifier Address of PLONK verifier contract
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
     * @notice Claim tokens using PLONK proof
     * @param proof PLONK zero-knowledge proof
     * @param nullifier Unique identifier (prevents double-claims)
     * @param recipient Address to receive tokens
     */
    function claim(
        PLONKProof calldata proof,
        bytes32 nullifier,
        address recipient
    ) external {
        require(block.timestamp < claimDeadline, "Claim period ended");
        require(recipient != address(0), "Invalid recipient");
        require(!nullifiers[nullifier], "Already claimed");

        // Prepare public inputs: [merkle_root, recipient, nullifier]
        uint256[3] memory instances = [
            uint256(merkleRoot),
            uint256(uint160(recipient)),
            uint256(nullifier)
        ];

        // Verify PLONK proof
        require(
            verifier.verifyProof(proof.proof, instances),
            "Invalid proof"
        );

        // Mark as claimed
        nullifiers[nullifier] = true;

        // Transfer tokens
        (bool success, ) = address(token).call(
            abi.encodeWithSelector(IERC20.transfer.selector, recipient, claimAmount)
        );
        require(success, "Token transfer failed");

        emit Claimed(nullifier, recipient, block.timestamp);
    }
    
    /**
     * @notice Check if nullifier has been claimed
     * @param nullifier The nullifier to check
     * @return True if already claimed
     */
    function isClaimed(bytes32 nullifier) external view returns (bool) {
        return nullifiers[nullifier];
    }
    
    /**
     * @notice Estimate gas for claim transaction
     * @dev Updated for PLONK - higher gas cost than Groth16
     * @return Estimated gas (1.3M for PLONK)
     */
    function estimateClaimGas(
        PLONKProof calldata proof,
        bytes32 nullifier,
        address recipient
    ) external view returns (uint256) {
        // PLONK verification costs ~900K gas (vs 300K for Groth16)
        // Plus storage + transfer (~200K)
        // Plus buffer (~200K)
        return 1_300_000; // Conservative estimate with buffer
    }
}

/**
 * @title ERC20 Interface
 * @notice Standard ERC20 interface
 */
interface IERC20 {
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
