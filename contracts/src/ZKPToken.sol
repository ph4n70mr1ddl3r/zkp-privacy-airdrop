// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

/**
 * @title ZKPToken
 * @notice ERC20 token for ZKP Privacy Airdrop
 * @dev Standard ERC20 with minting and burning capabilities, with a maximum supply cap
 * Maximum supply: 65,249,064,000 tokens (18 decimals)
 */
contract ZKPToken is ERC20, Ownable {
    uint256 public constant MAX_SUPPLY = 65_249_064_000 * 10**18;
    uint256 public immutable deployTimestamp;
    uint256 public mintCount;
    bool public mintingPaused;
    uint8 private constant DECIMALS = 18;

    event TokensMinted(
        address indexed to,
        uint256 amount,
        uint256 totalSupply,
        uint256 indexed mintId,
        uint256 timestamp
    );
    event TokensBurned(address indexed from, uint256 amount, uint256 totalSupply);
    event MintingPaused(address indexed account);
    event MintingUnpaused(address indexed account);

    constructor() ERC20("ZKP Token", "ZKP") Ownable(msg.sender) {
        deployTimestamp = block.timestamp;
    }

    function decimals() public pure override returns (uint8) {
        return DECIMALS;
    }



    /**
     * @notice Mint new tokens to a specified address
     * @param to Address to receive the minted tokens
     * @param amount Amount of tokens to mint
     * @dev Only callable by owner, enforces MAX_SUPPLY cap
     */
    function mint(address to, uint256 amount) external onlyOwner {
        require(!mintingPaused, "Minting is paused");
        require(to != address(0), "Invalid recipient address");
        require(amount > 0, "Amount must be greater than 0");
        require(totalSupply() + amount <= MAX_SUPPLY, "Minting would exceed MAX_SUPPLY");

        mintCount++;

        _mint(to, amount);
        emit TokensMinted(to, amount, totalSupply(), mintCount, block.timestamp);
    }

    /**
     * @notice Pause minting of new tokens
     * @dev Only callable by owner
     */
    function pauseMinting() external onlyOwner {
        mintingPaused = true;
        emit MintingPaused(msg.sender);
    }

    /**
     * @notice Unpause minting of new tokens
     * @dev Only callable by owner
     */
    function unpauseMinting() external onlyOwner {
        mintingPaused = false;
        emit MintingUnpaused(msg.sender);
    }

    /**
     * @notice Burn tokens from caller's balance
     * @param amount Amount of tokens to burn
     * @dev Reduces total supply
     */
    function burn(uint256 amount) external {
        require(amount > 0, "Amount must be greater than 0");
        require(balanceOf(msg.sender) >= amount, "Insufficient balance");

        _burn(msg.sender, amount);
        emit TokensBurned(msg.sender, amount, totalSupply());
    }
}
