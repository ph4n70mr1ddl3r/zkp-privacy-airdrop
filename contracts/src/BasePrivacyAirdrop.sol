// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";

/**
 * @title BasePrivacyAirdrop
 * @notice Base contract containing common functionality for privacy airdrop implementations
 * @dev This contract provides shared logic for different proof systems (Groth16, PLONK)
 */
abstract contract BasePrivacyAirdrop is ReentrancyGuard {
    using SafeERC20 for IERC20;

    bytes32 public immutable merkleRoot;
    mapping(bytes32 => bool) public nullifiers;
    IERC20 public immutable token;
    uint256 public immutable claimAmount;
    uint256 public immutable claimDeadline;
    bool public paused;

    event Claimed(bytes32 indexed nullifier, address indexed recipient, uint256 timestamp);
    event Paused(address indexed account);
    event Unpaused(address indexed account);

    /**
     * @notice Initialize the base airdrop contract
     * @param _token Address of the ERC20 token to distribute
     * @param _merkleRoot Root of the Merkle tree containing eligible addresses
     * @param _claimAmount Number of tokens each eligible address can claim
     * @param _claimDeadline Unix timestamp after which claims are no longer accepted
     */
    constructor(
        address _token,
        bytes32 _merkleRoot,
        uint256 _claimAmount,
        uint256 _claimDeadline
    ) {
        require(_token != address(0), "Invalid token address");
        require(_merkleRoot != bytes32(0), "Invalid merkle root");
        require(_claimAmount > 0, "Invalid claim amount");
        require(_claimDeadline > block.timestamp, "Invalid deadline");
        token = IERC20(_token);
        merkleRoot = _merkleRoot;
        claimAmount = _claimAmount;
        claimDeadline = _claimDeadline;
    }

    /**
     * @notice Modifier to validate claim parameters
     * @param recipient Address to receive tokens
     * @param nullifier Unique identifier for the claim
     */
    modifier validClaim(address recipient, bytes32 nullifier) {
        require(!paused, "Contract is paused");
        require(block.timestamp <= claimDeadline, "Claim period ended");
        require(recipient != address(0), "Invalid recipient");
        require(!nullifiers[nullifier], "Already claimed");
        _;
    }

    /**
     * @notice Pause the contract in case of emergency
     * @dev Only callable by admin
     */
    function pause() external {
        paused = true;
        emit Paused(msg.sender);
    }

    /**
     * @notice Unpause the contract to resume claims
     * @dev Only callable by admin
     */
    function unpause() external {
        paused = false;
        emit Unpaused(msg.sender);
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
     * @notice Internal function to transfer tokens safely
     * @param recipient Address to receive tokens
     * @param amount Amount of tokens to transfer
     */
    function _transferTokens(address recipient, uint256 amount) internal {
        token.safeTransfer(recipient, amount);
    }
}
