# Optimism Migration Summary

## Overview
Changed the deployment target from Ethereum mainnet to Optimism (Layer 2) for significantly lower gas fees while maintaining the same qualified addresses list from Ethereum mainnet.

## Key Changes Made

### 1. Network Configuration
- **Mainnet**: Changed from Ethereum → Optimism
- **Testnet**: Changed from Sepolia → Optimism Sepolia
- **Chain ID**: Updated from 1 → 10 (Optimism mainnet)
- **Testnet Chain ID**: 11155420 (Optimism Sepolia)

### 2. Gas Price Adjustments
- **Typical gas price**: Changed from 50 gwei (Ethereum) → 0.001 gwei (Optimism) for cost calculations
- **Maximum gas price cap**: Changed from 50 gwei → 0.1 gwei (Optimism is much cheaper, cap prevents excessive fees)
- **Gas price strategy**: Updated to reflect Optimism's EIP-1559 structure
- **Cost comparison added**: ~50x cheaper on Optimism

### 3. RPC Endpoints
- **Mainnet RPC**: `https://opt-mainnet.g.alchemy.com/v2/...`
- **Testnet RPC**: `https://opt-sepolia.g.alchemy.com/v2/...`
- **CLI network options**: Changed from `(mainnet, sepolia)` → `(optimism, optimism-sepolia)`

### 4. Documentation Updates
- **README.md**: Added Optimism mention in architecture diagram
- **00-specification.md**: Updated network config and gas price strategy
- **01-overview.md**: Added Optimism deployment note
- **02-technical-specification.md**: Updated config.yaml and gas prices
- **03-implementation-roadmap.md**: Updated testnet deployment to Optimism Sepolia
- **04-api-reference.md**: Updated network references and CLI options
- **CLAIMANT_GUIDE.md**: Added Optimism benefits and updated RPC references

### 5. Important Clarifications
- **Qualified addresses**: Still from Ethereum mainnet (paid ≥0.004 ETH gas fees)
- **Deployment**: On Optimism network
- **Gas units**: Same (~700,000 gas per claim)
- **Gas costs**: 10-100x cheaper due to Optimism's lower gas prices
- **User experience**: Same process, just cheaper gas fees

## Technical Details

### Gas Cost Comparison
| Network | Gas Units | Gas Price | Cost (ETH) | Cost at $3,000 ETH |
|---------|-----------|-----------|------------|-------------------|
| Ethereum | 700,000 | 50 gwei | 0.035 ETH | ~$105 |
| Optimism | 700,000 | 0.001 gwei | 0.0007 ETH | ~$2.10 |

**Savings**: ~50x cheaper on Optimism

### RPC Configuration
```yaml
# Optimism Mainnet
network:
  rpc_url: "https://opt-mainnet.g.alchemy.com/v2/..."
  chain_id: 10

# Optimism Sepolia (Testnet)
network:
  rpc_url: "https://opt-sepolia.g.alchemy.com/v2/..."
  chain_id: 11155420
```

### CLI Network Options
```bash
# Old
--network <NETWORK>  # Options: mainnet, sepolia

# New
--network <NETWORK>  # Options: optimism, optimism-sepolia
```

## Benefits of Optimism Deployment

### 1. **Lower Gas Costs**
- 10-100x cheaper than Ethereum mainnet
- Makes direct claims affordable for users
- Reduces relayer operational costs

### 2. **EVM Compatibility**
- Same smart contracts (no changes needed)
- Same tooling (Hardhat, Foundry, etc.)
- Same developer experience

### 3. **Security**
- Inherits Ethereum's security via optimistic rollups
- Proven technology with significant TVL
- Regular fraud proofs and challenges

### 4. **Ecosystem**
- Large existing user base
- Strong developer community
- Well-integrated wallets and tools

### 5. **User Experience**
- Faster transaction confirmation (~2 seconds)
- Cheaper transactions enable more use cases
- Same Ethereum addresses and keys work

## Migration Considerations

### 1. **Address Compatibility**
- Same Ethereum addresses work on Optimism
- Users can use existing Ethereum private keys
- No need for new wallets or key management

### 2. **Bridge Requirements**
- Users need ETH on Optimism for direct claims
- Can bridge from Ethereum mainnet or other L2s
- Relayers can fund Optimism wallets directly

### 3. **Monitoring**
- Different block explorers (Optimism Etherscan)
- Different RPC endpoints
- Different gas price oracles

### 4. **Tooling Updates**
- Update Hardhat/Foundry configurations
- Update deployment scripts
- Update monitoring and alerting

## Files Updated

1. **README.md** - Architecture and overview
2. **docs/00-specification.md** - Network config and gas prices
3. **docs/01-overview.md** - Project summary
4. **docs/02-technical-specification.md** - RPC config and gas settings
5. **docs/03-implementation-roadmap.md** - Testnet deployment
6. **docs/04-api-reference.md** - API and CLI network options
7. **CLAIMANT_GUIDE.md** - User instructions and cost comparison

## Testing Strategy
1. **Unit Tests**: Same (no changes needed)
2. **Integration Tests**: Use Optimism Sepolia testnet
3. **Gas Testing**: Verify costs on Optimism vs Ethereum
4. **Bridge Testing**: Test ETH bridging for relayer funding

## Next Steps
1. Update deployment scripts for Optimism
2. Test contract deployment on Optimism Sepolia
3. Update monitoring for Optimism metrics
4. Document bridging process for users
5. Update gas estimation in relayer for Optimism prices