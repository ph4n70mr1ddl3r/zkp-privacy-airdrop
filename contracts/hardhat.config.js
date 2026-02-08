{
  "solidity": "0.8.19",
  "optimizer": {
    "enabled": true,
    "runs": 200
  },
  "evmVersion": "paris",
  "networks": {
    "hardhat": {
      "chainId": 10
    },
    "optimism-sepolia": {
      "url": "https://opt-sepolia.g.alchemy.com/v2/${ALCHEMY_API_KEY}",
      "chainId": 11155420,
      "accounts": ["${PRIVATE_KEY}"]
    },
    "optimism": {
      "url": "https://opt-mainnet.g.alchemy.com/v2/${ALCHEMY_API_KEY}",
      "chainId": 10,
      "accounts": ["${PRIVATE_KEY}"]
    }
  }
}
