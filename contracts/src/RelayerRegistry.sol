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
    address public defaultRelayer;
    mapping(address => bool) public authorizedRelayers;
    mapping(address => uint256) public relayerBalances;

    event RelayerAuthorized(address indexed relayer);
    event RelayerDeauthorized(address indexed relayer);
    event DonationReceived(address indexed donor, uint256 amount);
    event FundsWithdrawn(address indexed relayer, uint256 amount);

    /**
     * @notice Modifier to restrict access to authorized relayers
     */
    modifier onlyAuthorized() {
        require(authorizedRelayers[msg.sender], "Not authorized");
        _;
    }

    /**
     * @notice Modifier to ensure address is not zero
     * @param addr Address to validate
     */
    modifier validAddress(address addr) {
        require(addr != address(0), "Invalid address");
        _;
    }

    /**
     * @notice Initialize the relayer registry
     * @param _defaultRelayer Address of the default relayer (auto-authorized)
     */
    constructor(address _defaultRelayer) Ownable(msg.sender) {
        require(_defaultRelayer != address(0), "Invalid default relayer: cannot be zero address");
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
        require(authorizedRelayers[relayer], "Relayer not authorized");
        authorizedRelayers[relayer] = false;

        uint256 balance = relayerBalances[relayer];
        if (balance > 0) {
            relayerBalances[relayer] = 0;
            relayerBalances[owner()] += balance;
            emit FundsWithdrawn(relayer, balance);
        }

        emit RelayerDeauthorized(relayer);
    }

    /**
     * @notice Internal function to handle donation logic
     * @param donor Address of the donor
     * @param amount Amount of ETH donated
     */
    function _handleDonation(address donor, uint256 amount) internal {
        require(amount > 0, "Donation must be greater than 0");
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
     */
    function withdraw(uint256 amount) external onlyAuthorized nonReentrant {
        require(relayerBalances[msg.sender] >= amount, "Insufficient balance");
        require(amount > 0, "Amount must be greater than zero");
        uint256 balance = relayerBalances[msg.sender];
        relayerBalances[msg.sender] = 0;
        emit FundsWithdrawn(msg.sender, amount);
        (bool success, ) = payable(msg.sender).call{value: amount}("");
        if (!success) {
            relayerBalances[msg.sender] = balance;
            revert("Transfer failed");
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
}
