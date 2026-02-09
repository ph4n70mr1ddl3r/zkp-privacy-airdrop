// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

contract ZKPToken is ERC20, Ownable {
    uint256 public constant MAX_SUPPLY = 65_249_064_000 * 10**18;
    uint256 public immutable deployTimestamp;
    uint256 public mintCount;
    bool public mintingPaused;

    event TokensMinted(address indexed to, uint256 amount, uint256 totalSupply, uint256 indexed mintId, uint256 timestamp);
    event TokensBurned(address indexed from, uint256 amount, uint256 totalSupply);
    event MintingPaused(address indexed account);
    event MintingUnpaused(address indexed account);

    constructor() ERC20("ZKP Token", "ZKP") Ownable(msg.sender) {
        deployTimestamp = block.timestamp;
    }

    function mint(address to, uint256 amount) external onlyOwner {
        require(!mintingPaused, "Minting is paused");
        require(to != address(0), "Invalid recipient address");
        require(amount > 0, "Amount must be greater than 0");
        require(totalSupply() + amount <= MAX_SUPPLY, "Minting would exceed MAX_SUPPLY");

        mintCount++;

        _mint(to, amount);
        emit TokensMinted(to, amount, totalSupply(), mintCount, block.timestamp);
    }

    function pauseMinting() external onlyOwner {
        mintingPaused = true;
        emit MintingPaused(msg.sender);
    }

    function unpauseMinting() external onlyOwner {
        mintingPaused = false;
        emit MintingUnpaused(msg.sender);
    }

    function burn(uint256 amount) external {
        require(amount > 0, "Amount must be greater than 0");
        require(balanceOf(msg.sender) >= amount, "Insufficient balance");

        _burn(msg.sender, amount);
        emit TokensBurned(msg.sender, amount, totalSupply());
    }
}
