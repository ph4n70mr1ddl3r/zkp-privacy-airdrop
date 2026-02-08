// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

contract ZKPToken is ERC20, Ownable {
    uint256 public constant MAX_SUPPLY = 65_249_064_000 * 10**18; // 65,249,064,000 ZKP tokens

    event TokensMinted(address indexed to, uint256 amount, uint256 totalSupply);

    constructor() ERC20("ZKP Token", "ZKP") {
        _mint(msg.sender, MAX_SUPPLY);
        emit TokensMinted(msg.sender, MAX_SUPPLY, MAX_SUPPLY);
    }

    function mint(address to, uint256 amount) external onlyOwner {
        require(to != address(0), "Invalid recipient address");
        require(amount > 0, "Amount must be greater than 0");
        require(totalSupply() + amount <= MAX_SUPPLY, "Minting would exceed MAX_SUPPLY");

        _mint(to, amount);
        emit TokensMinted(to, amount, totalSupply());
    }
}
