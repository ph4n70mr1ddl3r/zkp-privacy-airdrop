// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {ReentrancyGuard} from "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";

/**
 * @title BasePrivacyAirdrop
 * @author ZKP Airdrop Team
 * @notice Base contract for privacy-preserving ERC20 token airdrops
 * @dev Implements common functionality for airdrop contracts using ZK proofs
 *      Provides Merkle tree verification, nullifier tracking, and token distribution
 *      Designed to be inherited by specific proof system implementations (e.g., PLONK)
 */
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
    error EmergencyWithdrawalExceedsLimit();
    error CumulativeWithdrawalExceedsAvailable();

    /// @notice Merkle tree root of eligible addresses
    bytes32 public immutable MERKLE_ROOT;

    uint256 private constant MAX_CLAIM_DEADLINE = 365 days;

    /// @notice Mapping of used nullifiers to prevent double-claims
    mapping(bytes32 => bool) public nullifiers;

    /// @notice ERC20 token being distributed
    IERC20 public immutable TOKEN;

    /// @notice Amount of tokens each eligible address can claim
    uint256 public immutable CLAIM_AMOUNT;

    /// @notice Unix timestamp after which claims are no longer accepted
    uint256 public immutable CLAIM_DEADLINE;

    /// @notice Whether the contract is currently paused
    bool public paused;

    /// @notice Total amount of tokens claimed so far
    uint256 public totalClaimed;

    /// @notice Total tokens withdrawn in current period
    uint256 public totalWithdrawn;

    /// @notice Cumulative tokens withdrawn since deployment
    uint256 public cumulativeWithdrawn;

    /// @notice Timestamp of last emergency withdrawal
    uint256 public lastWithdrawalTime;

    /// @notice Maximum percentage of unclaimed tokens withdrawable per period
    uint256 public immutable MAX_WITHDRAWAL_PERCENT;

    /// @notice Time in seconds between withdrawal periods
    uint256 public immutable WITHDRAWAL_COOLDOWN;

    /// @notice Delay period after claim deadline before emergency withdrawals are available
    uint256 public constant EMERGENCY_WITHDRAWAL_DELAY = 30 days;

    /// @notice Timestamp when contract was deployed
    uint256 public immutable DEPLOY_TIMESTAMP;

    /// @notice Emitted when a claim is successfully processed
    /// @param nullifier Unique identifier of the claim
    /// @param recipient Address receiving the tokens
    /// @param timestamp Time when the claim was processed
    event Claimed(bytes32 indexed nullifier, address indexed recipient, uint256 timestamp);

    /// @notice Emitted when tokens are transferred to a recipient
    /// @param recipient Address receiving the tokens
    /// @param amount Amount of tokens transferred
    /// @param timestamp Time when the transfer occurred
    event TokensTransferred(address indexed recipient, uint256 amount, uint256 timestamp);

    /// @notice Emitted when the contract is paused
    /// @param account Address that paused the contract
    event Paused(address indexed account);

    /// @notice Emitted when the contract is unpaused
    /// @param account Address that unpaused the contract
    event Unpaused(address indexed account);

    /// @notice Emitted when emergency withdrawal is executed
    /// @param recipient Address receiving the withdrawn tokens
    /// @param amount Amount of tokens withdrawn
    /// @param cumulativeWithdrawn Total tokens withdrawn since deployment
    /// @param timestamp Time when the withdrawal occurred
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
        _validateTokenAddress(_token);
        _validateMerkleRoot(_merkleRoot);
        _validateClaimAmount(_claimAmount);
        _validateClaimDeadline(_claimDeadline);
        _validateWithdrawalSettings(_maxWithdrawalPercent, _withdrawalCooldown);

        TOKEN = IERC20(_token);
        MERKLE_ROOT = _merkleRoot;
        CLAIM_AMOUNT = _claimAmount;
        CLAIM_DEADLINE = _claimDeadline;
        MAX_WITHDRAWAL_PERCENT = _maxWithdrawalPercent;
        WITHDRAWAL_COOLDOWN = _withdrawalCooldown;
        // solhint-disable not-rely-on-time
        lastWithdrawalTime = block.timestamp;
        DEPLOY_TIMESTAMP = block.timestamp;
        // solhint-enable not-rely-on-time
    }

    function _validateTokenAddress(address _token) private pure {
        if (_token == address(0)) {
            revert InvalidTokenAddress();
        }
    }

    function _validateMerkleRoot(bytes32 _merkleRoot) private pure {
        bytes32 zeroRoot = bytes32(0);
        bytes32 onesRoot = bytes32(type(uint256).max);
        if (_merkleRoot == zeroRoot || _merkleRoot == onesRoot) {
            revert InvalidMerkleRoot();
        }

        bytes4 prefix = bytes4(_merkleRoot);
        if (prefix == bytes4(0) || prefix == bytes4(type(uint32).max)) {
            revert InvalidMerkleRootPrefix();
        }
    }

    function _validateClaimAmount(uint256 _claimAmount) private pure {
        if (_claimAmount == 0) {
            revert InvalidClaimAmount();
        }
    }

    function _validateClaimDeadline(uint256 _claimDeadline) private view {
        // solhint-disable not-rely-on-time
        if (_claimDeadline < block.timestamp) {
            revert InvalidClaimDeadline();
        }
        if (_claimDeadline > block.timestamp + MAX_CLAIM_DEADLINE) {
            revert InvalidClaimDeadline();
        }
        // solhint-enable not-rely-on-time
    }

    function _validateWithdrawalSettings(uint256 _maxWithdrawalPercent, uint256 _withdrawalCooldown) private pure {
        if (_maxWithdrawalPercent == 0 || _maxWithdrawalPercent > 100) {
            revert InvalidWithdrawalPercentage();
        }
        if (_withdrawalCooldown == 0) {
            revert InvalidWithdrawalCooldown();
        }
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
        // solhint-disable not-rely-on-time
        if (block.timestamp > CLAIM_DEADLINE) {
            revert ClaimPeriodEnded();
        }
        // solhint-enable not-rely-on-time
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
       * @dev Updates state only after successful transfer verification to prevent reentrancy
       * @dev IMPORTANT: Only supports standard ERC20 tokens without transfer fees or hooks
       * @dev Tokens with transfer fees will fail this transfer verification check
       */
    function _transferTokens(address recipient, uint256 amount) internal {
        if (recipient == address(0)) {
            revert InvalidRecipient();
        }
        uint256 balanceBefore = TOKEN.balanceOf(recipient);
        TOKEN.safeTransfer(recipient, amount);
        uint256 balanceAfter = TOKEN.balanceOf(recipient);

        uint256 actualReceived = balanceAfter - balanceBefore;
        if (actualReceived < amount) {
            revert InsufficientTokensReceived();
        }

        totalClaimed += amount;
        // solhint-disable not-rely-on-time
        emit TokensTransferred(recipient, amount, block.timestamp);
        // solhint-enable not-rely-on-time
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
        _validateEmergencyWithdrawTiming();
        _validateWithdrawalParameters(recipient, amount);

        uint256 unclaimedAmount = _getUnclaimedAmount();
        _validateWithdrawalAmount(amount, unclaimedAmount);
        _updateWithdrawalTracking(amount, unclaimedAmount);

        cumulativeWithdrawn += amount;

        // solhint-disable not-rely-on-time
        emit EmergencyWithdraw(recipient, amount, cumulativeWithdrawn, block.timestamp);
        // solhint-enable not-rely-on-time
        TOKEN.safeTransfer(recipient, amount);
    }

    function _validateEmergencyWithdrawTiming() private view {
        // solhint-disable not-rely-on-time
        if (block.timestamp < CLAIM_DEADLINE + EMERGENCY_WITHDRAWAL_DELAY) {
            revert EmergencyWithdrawalNotAvailable();
        }
        // solhint-enable not-rely-on-time
    }

    function _validateWithdrawalParameters(address recipient, uint256 amount) private pure {
        if (recipient == address(0)) {
            revert InvalidRecipient();
        }
        if (amount == 0) {
            revert InvalidWithdrawalAmount();
        }
    }

    function _getUnclaimedAmount() private view returns (uint256) {
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

        return unclaimedAmount;
    }

    function _validateWithdrawalAmount(uint256 amount, uint256 unclaimedAmount) private view {
        if (amount > unclaimedAmount) {
            revert WithdrawalExceedsUnclaimed();
        }
        uint256 maxEmergencyWithdraw = (unclaimedAmount * MAX_WITHDRAWAL_PERCENT) / 100;
        if (amount > maxEmergencyWithdraw) {
            revert EmergencyWithdrawalExceedsLimit();
        }
    }

    function _updateWithdrawalTracking(uint256 amount, uint256 unclaimedAmount) private {
        // solhint-disable not-rely-on-time
        uint256 timeSinceLastWithdrawal = block.timestamp - lastWithdrawalTime;
        // solhint-enable not-rely-on-time

        uint256 maxWithdrawalThisPeriod = (unclaimedAmount * MAX_WITHDRAWAL_PERCENT) / 100;
        if (timeSinceLastWithdrawal >= WITHDRAWAL_COOLDOWN) {
            totalWithdrawn = amount;
        } else {
            if (totalWithdrawn + amount > maxWithdrawalThisPeriod) {
                revert WithdrawalExceedsLimit();
            }
            totalWithdrawn += amount;
        }

        // solhint-disable not-rely-on-time
        lastWithdrawalTime = block.timestamp;
        // solhint-enable not-rely-on-time

        if (cumulativeWithdrawn + amount > unclaimedAmount) {
            revert CumulativeWithdrawalExceedsAvailable();
        }
    }
}
