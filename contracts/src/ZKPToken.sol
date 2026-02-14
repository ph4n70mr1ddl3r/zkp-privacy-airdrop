// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {ReentrancyGuard} from "@openzeppelin/contracts/utils/ReentrancyGuard.sol";

/**
 * @title ZKPToken
 * @author ZKP Airdrop Team
 * @notice ERC20 token for ZKP Privacy Airdrop
 * @dev Standard ERC20 with minting and burning capabilities, with a maximum supply cap
 * Maximum supply: 65,249,064,000 tokens (18 decimals)
 */
contract ZKPToken is ERC20, Ownable, ReentrancyGuard {
    error MintingIsPaused();
    error MintRecipientInvalid();
    error MintToContractNotAllowed();
    error MintAmountInvalid();
    error TokenMaxSupplyReached();
    error MintExceedsMaxSupply();
    error BurnAmountInvalid();
    error BurnInsufficientBalance();

    /// @notice Maximum supply of ZKP tokens
    uint256 public constant MAX_SUPPLY = 65_249_064_000 * 10**18;
    /// @notice Timestamp when contract was deployed
    uint256 public immutable DEPLOY_TIMESTAMP;
    /// @notice Total number of minting operations performed
    uint256 public mintCount;
    /// @notice Whether token minting is currently paused
    bool public mintingPaused;
    uint8 private constant DECIMALS = 18;

    /// @notice Emitted when new tokens are minted
    /// @param to Address receiving the minted tokens
    /// @param amount Amount of tokens minted
    /// @param totalSupply Total supply after minting
    /// @param mintId Sequential ID of this mint operation
    /// @param timestamp Time when minting occurred
    event TokensMinted(
        address indexed to,
        uint256 amount,
        uint256 totalSupply,
        uint256 indexed mintId,
        uint256 timestamp
    );
    /// @notice Emitted when tokens are burned
    /// @param from Address burning the tokens
    /// @param amount Amount of tokens burned
    /// @param totalSupply Total supply after burning
    event TokensBurned(address indexed from, uint256 amount, uint256 totalSupply);
    /// @notice Emitted when token minting is paused
    /// @param account Address that paused minting
    event MintingPaused(address indexed account);
    /// @notice Emitted when token minting is unpaused
    /// @param account Address that unpaused minting
    event MintingUnpaused(address indexed account);
    /// @notice Emitted when maximum supply is reached
    /// @param finalSupply Total supply when cap is reached
    event MaxSupplyReached(uint256 finalSupply);

    /**
     * @notice Initialize the ZKP Token contract
     * @dev Sets up the token with "ZKP Token" name and "ZKP" symbol
     *      Records deployment timestamp and sets msg.sender as the initial owner
     *      Owner can mint tokens, pause minting, and burn tokens (anyone can burn their own)
     */
    constructor() ERC20("ZKP Token", "ZKP") Ownable(msg.sender) {
        // solhint-disable not-rely-on-time
        DEPLOY_TIMESTAMP = block.timestamp;
        // solhint-enable not-rely-on-time
    }

    /**
     * @notice Returns the number of decimals used to get token representation
     * @dev Standard ERC20 decimals function returning 18 for ZKP tokens
     * @return Number of decimals (18)
     */
    function decimals() public pure override returns (uint8) {
        return DECIMALS;
    }

    /**
      * @notice Mint new tokens to a specified address
      * @param to Address to receive the minted tokens
      * @param amount Amount of tokens to mint
      * @dev Only callable by owner, enforces MAX_SUPPLY cap
      */
    function mint(address to, uint256 amount) external onlyOwner nonReentrant {
        if (mintingPaused) {
            revert MintingIsPaused();
        }
        if (to == address(0)) {
            revert MintRecipientInvalid();
        }
        if (to == address(this)) {
            revert MintToContractNotAllowed();
        }
        if (amount == 0) {
            revert MintAmountInvalid();
        }
        if (totalSupply() >= MAX_SUPPLY) {
            revert TokenMaxSupplyReached();
        }
        if (totalSupply() + amount > MAX_SUPPLY) {
            revert MintExceedsMaxSupply();
        }

        _mint(to, amount);
        ++mintCount;
        // solhint-disable not-rely-on-time
        emit TokensMinted(to, amount, totalSupply(), mintCount, block.timestamp);
        // solhint-enable not-rely-on-time

        if (totalSupply() == MAX_SUPPLY) {
            emit MaxSupplyReached(totalSupply());
        }
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
    function burn(uint256 amount) external nonReentrant {
        if (amount == 0) {
            revert BurnAmountInvalid();
        }
        if (balanceOf(msg.sender) < amount) {
            revert BurnInsufficientBalance();
        }

        _burn(msg.sender, amount);
        emit TokensBurned(msg.sender, amount, totalSupply());
    }
}
