require("@nomicfoundation/hardhat-toolbox");

module.exports = {
  solidity: {
    version: "0.8.19",
    settings: {
      optimizer: {
        enabled: true,
        runs: 200
      },
      evmVersion: "paris"
    }
  },
  networks: {
    hardhat: {
      chainId: 10
    },
    "optimism-sepolia": {
      url: process.env.ALCHEMY_API_KEY
        ? `https://opt-sepolia.g.alchemy.com/v2/${process.env.ALCHEMY_API_KEY}`
        : "https://sepolia.optimism.io",
      chainId: 11155420,
      accounts: process.env.PRIVATE_KEY ? [process.env.PRIVATE_KEY] : []
    },
    optimism: {
      url: process.env.ALCHEMY_API_KEY
        ? `https://opt-mainnet.g.alchemy.com/v2/${process.env.ALCHEMY_API_KEY}`
        : "https://mainnet.optimism.io",
      chainId: 10,
      accounts: process.env.PRIVATE_KEY ? [process.env.PRIVATE_KEY] : []
    }
  }
};
