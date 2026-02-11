require("@nomicfoundation/hardhat-toolbox");

module.exports = {
  solidity: {
    version: "0.8.20",
    settings: {
      optimizer: {
        enabled: true,
        runs: 200
      },
      evmVersion: "paris"
    }
  },
  paths: {
    sources: "./src",
    tests: "./test",
    cache: "./cache",
    artifacts: "./artifacts"
  },
  networks: {
    hardhat: {
      chainId: 10
    },
    "optimism-sepolia": {
      url: process.env.OPTIMISM_SEPOLIA_RPC_URL
        ? process.env.OPTIMISM_SEPOLIA_RPC_URL
        : (process.env.ALCHEMY_API_KEY
          ? `https://opt-sepolia.g.alchemy.com/v2/${process.env.ALCHEMY_API_KEY}`
          : "https://sepolia.optimism.io"),
      chainId: 11155420,
      accounts: (() => {
        if (!process.env.PRIVATE_KEY) return [];
        const privateKey = process.env.PRIVATE_KEY.trim();

        if (!privateKey.startsWith('0x')) {
          throw new Error("CRITICAL: Private key must start with '0x' prefix");
        }
        if (privateKey.length !== 66) {
          throw new Error("CRITICAL: Private key must be 32 bytes (64 hex chars + '0x' prefix)");
        }

        const insecureKeys = [
          "0x0000000000000000000000000000000000000000000000000000000000000000000",
          "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
          "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d",
          "0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a",
        ];
        if (insecureKeys.includes(privateKey.toLowerCase())) {
          throw new Error("CRITICAL: Using insecure test private key! Please set a secure PRIVATE_KEY environment variable.");
        }
        return [privateKey];
      })()
    },
    optimism: {
      url: process.env.OPTIMISM_RPC_URL
        ? process.env.OPTIMISM_RPC_URL
        : (process.env.ALCHEMY_API_KEY
          ? `https://opt-mainnet.g.alchemy.com/v2/${process.env.ALCHEMY_API_KEY}`
          : "https://mainnet.optimism.io"),
      chainId: 10,
      accounts: (() => {
        if (!process.env.PRIVATE_KEY) return [];
        const privateKey = process.env.PRIVATE_KEY.trim();

        if (!privateKey.startsWith('0x')) {
          throw new Error("CRITICAL: Private key must start with '0x' prefix");
        }
        if (privateKey.length !== 66) {
          throw new Error("CRITICAL: Private key must be 32 bytes (64 hex chars + '0x' prefix)");
        }

        const insecureKeys = [
          "0x0000000000000000000000000000000000000000000000000000000000000000000",
          "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
          "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d",
          "0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a",
        ];
        if (insecureKeys.includes(privateKey.toLowerCase())) {
          throw new Error("CRITICAL: Using insecure test private key! Please set a secure PRIVATE_KEY environment variable.");
        }
        return [privateKey];
      })()
    }
  }
};
