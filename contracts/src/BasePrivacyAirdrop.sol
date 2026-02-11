// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {ReentrancyGuard} from "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";

abstract contract BasePrivacyAirdrop is ReentrancyGuard, Ownable {
    using SafeERC20 for IERC20;

    error InvalidTokenAddress();
    error InvalidMerkleRoot();
    error InvalidClaimAmount();
    error InvalidClaimDeadline();
    error InvalidWithdrawalPercentage();
    error InvalidWithdrawalCooldown();
    error ContractPaused();
    error ClaimPeriodEnded();
    error InvalidRecipient();
    error InvalidNullifier();
    error AlreadyClaimed();
    error InvalidMerkleRootPrefix();
    error InsufficientTokensReceived();
    error EmergencyWithdrawalNotAvailable();
    error InvalidWithdrawalAmount();
    error NoContractBalance();
    error BalanceInconsistent();
    error NoUnclaimedTokens();
    error WithdrawalExceedsUnclaimed();
    error WithdrawalExceedsLimit();
    error CumulativeWithdrawalExceedsAvailable();

    bytes32 public immutable MERKLE_ROOT;
    uint256 private constant MAX_CLAIM_DEADLINE = 365 days;
    mapping(bytes32 => bool) public nullifiers;
    IERC20 public immutable TOKEN;
    uint256 public immutable CLAIM_AMOUNT;
    uint256 public immutable CLAIM_DEADLINE;
    bool public paused;
    uint256 public totalClaimed;
    uint256 public totalWithdrawn;
    uint256 public cumulativeWithdrawn;
    uint256 public lastWithdrawalTime;
    uint256 public immutable MAX_WITHDRAWAL_PERCENT;
    uint256 public immutable WITHDRAWAL_COOLDOWN;
    uint256 public constant EMERGENCY_WITHDRAWAL_DELAY = 30 days;

    event Claimed(bytes32 indexed nullifier, address indexed recipient, uint256 timestamp);
    event TokensTransferred(address indexed recipient, uint256 amount, uint256 timestamp);
    event Paused(address indexed account);
    event Unpaused(address indexed account);
    event EmergencyWithdraw(address indexed recipient, uint256 amount, uint256 cumulativeWithdrawn, uint256 timestamp);

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
        if (_token == address(0)) {
            revert InvalidTokenAddress();
        }

        bytes32 zeroRoot = bytes32(0);
        bytes32 onesRoot = bytes32(type(uint256).max);
        if (_merkleRoot == zeroRoot || _merkleRoot == onesRoot) {
            revert InvalidMerkleRoot();
        }

        bytes4 prefix = bytes4(_merkleRoot);
        if (prefix == bytes4(0) || prefix == bytes4(type(uint32).max)) {
            revert InvalidMerkleRootPrefix();
        }

        if (_claimAmount == 0) {
            revert InvalidClaimAmount();
        }
        if (_claimDeadline <= block.timestamp) {
            revert InvalidClaimDeadline();
        }
        if (_claimDeadline >= block.timestamp + MAX_CLAIM_DEADLINE) {
            revert InvalidClaimDeadline();
        }
        if (_maxWithdrawalPercent == 0 || _maxWithdrawalPercent > 100) {
            revert InvalidWithdrawalPercentage();
        }
        if (_withdrawalCooldown == 0) {
            revert InvalidWithdrawalCooldown();
        }

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
        if (paused) {
            revert ContractPaused();
        }
        if (block.timestamp > CLAIM_DEADLINE) {
            revert ClaimPeriodEnded();
        }
        if (recipient == address(0)) {
            revert InvalidRecipient();
        }
        if (nullifier == bytes32(0)) {
            revert InvalidNullifier();
        }
        if (nullifiers[nullifier]) {
            revert AlreadyClaimed();
        }
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
        uint256 balanceBefore = TOKEN.balanceOf(recipient);
        TOKEN.safeTransfer(recipient, amount);
        uint256 balanceAfter = TOKEN.balanceOf(recipient);

        uint256 actualReceived = balanceAfter - balanceBefore;
        if (actualReceived < amount) {
            revert InsufficientTokensReceived();
        }
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
        if (block.timestamp <= CLAIM_DEADLINE + EMERGENCY_WITHDRAWAL_DELAY) {
            revert EmergencyWithdrawalNotAvailable();
        }
        if (recipient == address(0)) {
            revert InvalidRecipient();
        }
        if (amount == 0) {
            revert InvalidWithdrawalAmount();
        }

        uint256 contractBalance = TOKEN.balanceOf(address(this));
        if (contractBalance == 0) {
            revert NoContractBalance();
        }

        uint256 claimed = totalClaimed;
        if (contractBalance < claimed) {
            revert BalanceInconsistent();
        }
        uint256 unclaimedAmount = contractBalance - claimed;
        if (unclaimedAmount == 0) {
            revert NoUnclaimedTokens();
        }
        if (amount > unclaimedAmount) {
            revert WithdrawalExceedsUnclaimed();
        }

        uint256 timeSinceLastWithdrawal = block.timestamp - lastWithdrawalTime;

        uint256 maxWithdrawalThisPeriod;
        if (timeSinceLastWithdrawal >= WITHDRAWAL_COOLDOWN) {
            maxWithdrawalThisPeriod = (unclaimedAmount * MAX_WITHDRAWAL_PERCENT) / 100;
            totalWithdrawn = amount;
            lastWithdrawalTime = block.timestamp;
        } else {
            maxWithdrawalThisPeriod = (unclaimedAmount * MAX_WITHDRAWAL_PERCENT) / 100;
            if (totalWithdrawn + amount > maxWithdrawalThisPeriod) {
                revert WithdrawalExceedsLimit();
            }
            totalWithdrawn += amount;
        }

        if (cumulativeWithdrawn + amount > unclaimedAmount) {
            revert CumulativeWithdrawalExceedsAvailable();
        }

        cumulativeWithdrawn += amount;

        emit EmergencyWithdraw(recipient, amount, cumulativeWithdrawn, block.timestamp);
        TOKEN.safeTransfer(recipient, amount);
    }
}
