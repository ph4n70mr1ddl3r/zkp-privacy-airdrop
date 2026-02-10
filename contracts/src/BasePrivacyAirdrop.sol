// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";

/**
 * @title BasePrivacyAirdrop
 * @notice Base contract containing common functionality for privacy airdrop implementations
 * @dev This contract provides shared logic for different proof systems (Groth16, PLONK)
 */
abstract contract BasePrivacyAirdrop is ReentrancyGuard, Ownable {
    using SafeERC20 for IERC20;

    bytes32 public immutable merkleRoot;
    mapping(bytes32 => bool) public nullifiers;
    IERC20 public immutable token;
    uint256 public immutable claimAmount;
    uint256 public immutable claimDeadline;
    bool public paused;

    event Claimed(bytes32 indexed nullifier, address indexed recipient, uint256 timestamp);
    event TokensTransferred(address indexed recipient, uint256 amount, uint256 timestamp);
    event Paused(address indexed account);
    event Unpaused(address indexed account);
    event EmergencyWithdraw(address indexed recipient, uint256 amount, uint256 timestamp);

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
    ) Ownable(msg.sender) {
        require(_token != address(0), "Invalid token address: cannot be zero address");
        require(_merkleRoot != bytes32(0), "Invalid merkle root: cannot be zero");
        require(_claimAmount > 0, "Invalid claim amount: must be greater than zero");
        require(_claimDeadline > block.timestamp, "Invalid deadline: must be in the future");
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
        require(!paused, "Contract is paused: claims are temporarily suspended");
        require(block.timestamp <= claimDeadline, "Claim period ended: deadline has passed");
        require(recipient != address(0), "Invalid recipient: cannot be zero address");
        require(nullifier != bytes32(0), "Invalid nullifier: cannot be zero");
        require(!nullifiers[nullifier], "Already claimed: this nullifier has been used");
        _;
    }

    /**
     * @notice Pause the contract in case of emergency
     * @dev Only callable by contract owner
     */
    function pause() external onlyOwner {
        paused = true;
        emit Paused(msg.sender);
    }

    /**
     * @notice Unpause the contract to resume claims
     * @dev Only callable by contract owner
     */
    function unpause() external onlyOwner {
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
      * @dev Uses SafeERC20 to prevent token transfer failures
      */
    function _transferTokens(address recipient, uint256 amount) internal nonReentrant {
        token.safeTransfer(recipient, amount);
        emit TokensTransferred(recipient, amount, block.timestamp);
    }

    /**
     * @notice Emergency withdraw tokens after claim deadline
     * @param recipient Address to receive the withdrawn tokens
     * @param amount Amount of tokens to withdraw
     * @dev Only callable by owner and only after claim deadline has passed
     * @dev This is a safety mechanism to recover unclaimed tokens
     */
    function emergencyWithdraw(address recipient, uint256 amount) external onlyOwner nonReentrant {
        require(block.timestamp > claimDeadline, "Claim period not ended");
        require(recipient != address(0), "Invalid recipient");
        require(amount > 0, "Amount must be greater than zero");
        require(amount <= token.balanceOf(address(this)), "Insufficient contract balance");
        _transferTokens(recipient, amount);
        emit EmergencyWithdraw(recipient, amount, block.timestamp);
    }
}
