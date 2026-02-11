// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {ReentrancyGuard} from "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";

/**
 * @title BasePrivacyAirdrop
 * @notice Base contract containing common functionality for privacy airdrop implementations
 * @dev This contract provides shared logic for different proof systems (Groth16, PLONK)
 */
abstract contract BasePrivacyAirdrop is ReentrancyGuard, Ownable {
    using SafeERC20 for IERC20;

    bytes32 public immutable MERKLE_ROOT;
    uint256 private constant MAX_CLAIM_DEADLINE = 365 days;
    mapping(bytes32 => bool) public nullifiers;
    IERC20 public immutable TOKEN;
    uint256 public immutable CLAIM_AMOUNT;
    uint256 public immutable CLAIM_DEADLINE;
    bool public paused;
    uint256 public totalClaimed;
    uint256 public totalWithdrawn;
    uint256 public lastWithdrawalTime;
    uint256 public immutable MAX_WITHDRAWAL_PERCENT;
    uint256 public immutable WITHDRAWAL_COOLDOWN;

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
     * @param _maxWithdrawalPercent Maximum percentage of unclaimed tokens that can be withdrawn per period (default 10)
     * @param _withdrawalCooldown Time in seconds between withdrawal periods (default 24 hours)
     */
    constructor(
        address _token,
        bytes32 _merkleRoot,
        uint256 _claimAmount,
        uint256 _claimDeadline,
        uint256 _maxWithdrawalPercent,
        uint256 _withdrawalCooldown
    ) Ownable(msg.sender) {
        require(_token != address(0), "Invalid token address: cannot be zero address");
        
        bytes32 zeroRoot = bytes32(0);
        bytes32 onesRoot = bytes32(type(uint256).max);
        require(_merkleRoot != zeroRoot && _merkleRoot != onesRoot,
            "Invalid merkle root: cannot be all zeros or all ones");
        
        bytes4 prefix = bytes4(_merkleRoot);
        require(prefix != bytes4(0) && prefix != bytes4(type(uint32).max),
            "Invalid merkle root: suspicious prefix pattern");
        
        require(_claimAmount > 0, "Invalid claim amount: must be greater than zero");
        require(_claimDeadline > block.timestamp, "Invalid deadline: must be in the future");
        require(_claimDeadline < block.timestamp + MAX_CLAIM_DEADLINE, "Deadline too far in future");
        require(_maxWithdrawalPercent > 0 && _maxWithdrawalPercent <= 100, "Invalid withdrawal percentage");
        require(_withdrawalCooldown > 0, "Invalid withdrawal cooldown");

        TOKEN = IERC20(_token);
        MERKLE_ROOT = _merkleRoot;
        CLAIM_AMOUNT = _claimAmount;
        CLAIM_DEADLINE = _claimDeadline;
        MAX_WITHDRAWAL_PERCENT = _maxWithdrawalPercent;
        WITHDRAWAL_COOLDOWN = _withdrawalCooldown;
    }

    /**
     * @notice Modifier to validate claim parameters
     * @param recipient Address to receive tokens
     * @param nullifier Unique identifier for the claim
     */
    modifier validClaim(address recipient, bytes32 nullifier) {
        require(!paused, "Contract is paused: claims are temporarily suspended");
        require(block.timestamp <= CLAIM_DEADLINE, "Claim period ended: deadline has passed");
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
    function _transferTokens(address recipient, uint256 amount) internal {
        TOKEN.safeTransfer(recipient, amount);
        totalClaimed += amount;
        emit TokensTransferred(recipient, amount, block.timestamp);
    }

    /**
     * @notice Emergency withdraw tokens after claim deadline
     * @param recipient Address to receive the withdrawn tokens
     * @param amount Amount of tokens to withdraw
     * @dev Only callable by owner and only after claim deadline has passed
     * @dev This is a safety mechanism to recover unclaimed tokens
     * @dev Can only withdraw tokens that have not been claimed (balance - totalClaimed)
     * @dev Implements withdrawal limits: max maxWithdrawalPercent of remaining tokens per withdrawalCooldown period
     * @dev Uses nonReentrant modifier to prevent reentrancy attacks
     */
    function emergencyWithdraw(address recipient, uint256 amount) external onlyOwner nonReentrant {
        require(block.timestamp > CLAIM_DEADLINE, "Claim period not ended");
        require(recipient != address(0), "Invalid recipient");
        require(amount > 0, "Amount must be greater than zero");

        uint256 contractBalance = TOKEN.balanceOf(address(this));
        require(contractBalance > 0, "Contract has no balance");

        uint256 claimed = totalClaimed;
        require(contractBalance >= claimed, "Contract balance inconsistent");
        uint256 unclaimedAmount = contractBalance - claimed;
        require(unclaimedAmount > 0, "No unclaimed tokens available");
        require(amount <= unclaimedAmount, "Cannot withdraw claimed tokens");

        uint256 timeSinceLastWithdrawal = block.timestamp - lastWithdrawalTime;

        if (timeSinceLastWithdrawal >= WITHDRAWAL_COOLDOWN) {
            totalWithdrawn = 0;
        }

        uint256 maxWithdrawalThisPeriod = (unclaimedAmount * MAX_WITHDRAWAL_PERCENT) / 100;
        require(amount + totalWithdrawn <= maxWithdrawalThisPeriod, "Withdrawal amount exceeds per-period limit");

        totalWithdrawn += amount;
        lastWithdrawalTime = block.timestamp;

        emit EmergencyWithdraw(recipient, amount, block.timestamp);
        TOKEN.safeTransfer(recipient, amount);
    }
}
