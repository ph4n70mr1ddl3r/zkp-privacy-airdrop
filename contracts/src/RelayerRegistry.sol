// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

interface IRelayerRegistry {
    function authorizeRelayer(address relayer) external;
    function donate() external payable;
    function withdraw(uint256 amount) external;
    function isAuthorized(address relayer) external view returns (bool);
    function balanceOf(address relayer) external view returns (uint256);
    function defaultRelayer() external view returns (address);
}

contract RelayerRegistry is IRelayerRegistry, ReentrancyGuard, Ownable {
    address public defaultRelayer;
    mapping(address => bool) public authorizedRelayers;
    mapping(address => uint256) public relayerBalances;

    event RelayerAuthorized(address indexed relayer);
    event DonationReceived(address indexed donor, uint256 amount);
    event FundsWithdrawn(address indexed relayer, uint256 amount);

    modifier onlyAuthorized() {
        require(authorizedRelayers[msg.sender], "Not authorized");
        _;
    }

    modifier validAddress(address addr) {
        require(addr != address(0), "Invalid address");
        _;
    }

    constructor(address _defaultRelayer) Ownable(msg.sender) {
        defaultRelayer = _defaultRelayer;
        authorizedRelayers[_defaultRelayer] = true;
        emit RelayerAuthorized(_defaultRelayer);
    }

    function authorizeRelayer(address relayer) external onlyOwner validAddress(relayer) {
        authorizedRelayers[relayer] = true;
        emit RelayerAuthorized(relayer);
    }

    function donate() external payable {
        relayerBalances[defaultRelayer] += msg.value;
        emit DonationReceived(msg.sender, msg.value);
    }

    function withdraw(uint256 amount) external onlyAuthorized nonReentrant {
        require(relayerBalances[msg.sender] >= amount, "Insufficient balance");
        require(amount > 0, "Amount must be greater than zero");
        relayerBalances[msg.sender] -= amount;
        (bool success, ) = payable(msg.sender).call{value: amount}("");
        require(success, "Transfer failed");
        emit FundsWithdrawn(msg.sender, amount);
    }

    function isAuthorized(address relayer) external view returns (bool) {
        return authorizedRelayers[relayer];
    }

    function balanceOf(address relayer) external view returns (uint256) {
        return relayerBalances[relayer];
    }

    function defaultRelayer() external view returns (address) {
        return defaultRelayer;
    }

    receive() external payable {
        donate();
    }
}
