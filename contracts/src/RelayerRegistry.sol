// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {ReentrancyGuard} from "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";

/**
 * @title IRelayerRegistry
 * @notice Interface for relayer registry contract
 */
interface IRelayerRegistry {
    /**
     * @notice Authorize a relayer to use funds
     * @param relayer Address of the relayer to authorize
     */
    function authorizeRelayer(address relayer) external;

    /**
     * @notice Deauthorize a relayer
     * @param relayer Address of the relayer to deauthorize
     */
    function deauthorizeRelayer(address relayer) external;

    /**
     * @notice Donate ETH to the relayer registry
     */
    function donate() external payable;

    /**
     * @notice Withdraw funds from relayer balance
     * @param amount Amount to withdraw
     */
    function withdraw(uint256 amount) external;

    /**
     * @notice Check if a relayer is authorized
     * @param relayer Address to check
     * @return True if authorized
     */
    function isAuthorized(address relayer) external view returns (bool);

    /**
     * @notice Get relayer balance
     * @param relayer Address of the relayer
     * @return Balance in wei
     */
    function balanceOf(address relayer) external view returns (uint256);

    /**
     * @notice Get default relayer address
     * @return Address of the default relayer
     */
    function getDefaultRelayer() external view returns (address);
}

/**
 * @title RelayerRegistry
 * @notice Manages authorized relayers and their fund balances
 * @dev Allows relayers to withdraw donated funds and manages authorization
 */
contract RelayerRegistry is IRelayerRegistry, ReentrancyGuard, Ownable {
    error NotAuthorized();
    error InvalidAddress();
    error InvalidDefaultRelayer();
    error RelayerNotAuthorized();
    error DonationMustBePositive();
    error InsufficientBalance();
    error AmountMustBePositive();
    error TransferFailed();

    address public defaultRelayer;
    mapping(address => bool) public authorizedRelayers;
    mapping(address => uint256) public relayerBalances;

    event RelayerAuthorized(address indexed relayer);
    event RelayerDeauthorized(address indexed relayer);
    event DonationReceived(address indexed donor, uint256 amount);
    event FundsWithdrawn(address indexed relayer, uint256 amount);
    event BalanceTransferredToOwner(address indexed relayer, address indexed owner, uint256 amount);

    /**
     * @notice Modifier to restrict access to authorized relayers
     */
    modifier onlyAuthorized() {
        if (!authorizedRelayers[msg.sender]) {
            revert NotAuthorized();
        }
        _;
    }

    /**
     * @notice Modifier to ensure address is not zero
     * @param addr Address to validate
     */
    modifier validAddress(address addr) {
        if (addr == address(0)) {
            revert InvalidAddress();
        }
        _;
    }

    /**
     * @notice Initialize the relayer registry
     * @param _defaultRelayer Address of the default relayer (auto-authorized)
     */
    constructor(address _defaultRelayer) Ownable(msg.sender) {
        if (_defaultRelayer == address(0)) {
            revert InvalidDefaultRelayer();
        }
        defaultRelayer = _defaultRelayer;
        authorizedRelayers[_defaultRelayer] = true;
        emit RelayerAuthorized(_defaultRelayer);
    }

    /**
     * @notice Authorize a relayer to use funds
     * @param relayer Address of relayer to authorize
     * @dev Only callable by owner
     */
    function authorizeRelayer(address relayer) external onlyOwner validAddress(relayer) {
        authorizedRelayers[relayer] = true;
        emit RelayerAuthorized(relayer);
    }

    /**
     * @notice Deauthorize a relayer
     * @param relayer Address of relayer to deauthorize
     * @dev Only callable by owner, transfers remaining balance to owner
     */
    function deauthorizeRelayer(address relayer) external onlyOwner validAddress(relayer) {
        if (!authorizedRelayers[relayer]) {
            revert RelayerNotAuthorized();
        }
        authorizedRelayers[relayer] = false;

        uint256 balance = relayerBalances[relayer];
        if (balance > 0) {
            relayerBalances[relayer] = 0;
            relayerBalances[owner()] += balance;
            emit FundsWithdrawn(relayer, balance);
            emit BalanceTransferredToOwner(relayer, owner(), balance);
        }

        emit RelayerDeauthorized(relayer);
    }

    function _handleDonation(address donor, uint256 amount) internal {
        if (amount == 0) {
            revert DonationMustBePositive();
        }
        relayerBalances[defaultRelayer] += amount;
        emit DonationReceived(donor, amount);
    }

    /**
     * @notice Donate ETH to default relayer
     * @dev ETH is added to default relayer's balance
     */
    function donate() external payable {
        _handleDonation(msg.sender, msg.value);
    }

    /**
     * @notice Withdraw funds from caller's balance
     * @param amount Amount of ETH to withdraw in wei
     * @dev Only callable by authorized relayers
     * @dev Uses checks-effects-interactions pattern to prevent reentrancy attacks
     */
    function withdraw(uint256 amount) external onlyAuthorized nonReentrant {
        if (relayerBalances[msg.sender] < amount) {
            revert InsufficientBalance();
        }
        if (amount == 0) {
            revert AmountMustBePositive();
        }
        relayerBalances[msg.sender] -= amount;
        emit FundsWithdrawn(msg.sender, amount);
        (bool success, ) = payable(msg.sender).call{value: amount}("");
        if (!success) {
            revert TransferFailed();
        }
    }

    /**
     * @notice Check if a relayer is authorized
     * @param relayer Address to check
     * @return True if authorized
     */
    function isAuthorized(address relayer) external view returns (bool) {
        return authorizedRelayers[relayer];
    }

    /**
     * @notice Get relayer balance
     * @param relayer Address of relayer
     * @return Balance in wei
     */
    function balanceOf(address relayer) external view returns (uint256) {
        return relayerBalances[relayer];
    }

    /**
     * @notice Get default relayer address
     * @dev Returns the default relayer that receives donations
     * @return Address of default relayer
     */
    function getDefaultRelayer() external view returns (address) {
        return defaultRelayer;
    }

    /**
      * @notice Receive ETH as a donation
      * @dev ETH is added to default relayer's balance
      */
    receive() external payable {
        _handleDonation(msg.sender, msg.value);
    }

    /**
      * @notice Fallback function for receiving ETH as a donation
      * @dev ETH is added to default relayer's balance
      */
    fallback() external payable {
        _handleDonation(msg.sender, msg.value);
    }
}
