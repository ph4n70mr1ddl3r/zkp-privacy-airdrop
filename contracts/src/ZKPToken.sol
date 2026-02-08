// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

contract ZKPToken is ERC20, Ownable {
    uint256 public constant MAX_SUPPLY = 65_249_064_000 * 10**18; // 65,249,064,000 ZKP tokens

    constructor() ERC20("ZKP Token", "ZKP") {
        _mint(msg.sender, MAX_SUPPLY);
    }

    function mint(address to, uint256 amount) external onlyOwner {
        _mint(to, amount);
    }
}
