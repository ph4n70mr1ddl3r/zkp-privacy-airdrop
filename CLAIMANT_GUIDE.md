# Claimant Guide for ZKP Privacy Airdrop

## Overview

This guide explains the complete process for eligible users to claim ZKP tokens privately. The system allows 65,249,064 Ethereum addresses that paid at least **0.004 ETH in gas fees** (from genesis until December 31, 2025) to claim 1,000 ZKP tokens each on the **Optimism network** without revealing which specific address they own.

**Deployed on Optimism**: The airdrop contract is deployed on Optimism (Layer 2), making gas fees 10-100x cheaper than Ethereum mainnet.

## Step 1: Check Eligibility

### Download the Accounts List
```bash
# Download the list of eligible addresses
gdown 1yvgsuDMhamUoKAfH59iuyDtm7x5mnHRX
```

**File**: `accounts.csv` (1.216 GiB)
**Contents**: 65,249,064 Ethereum addresses, one per line
**Criteria**: Addresses that paid ≥0.004 ETH in gas fees by Dec 31, 2025

### Verify Your Address
Check if your Ethereum address is in the list:
```bash
grep -i "0xYourAddressHere" accounts.csv
```

If your address is in the list, you're eligible to claim 1,000 ZKP tokens.

## Step 2: Prepare Credentials

You need two pieces of information:

1. **Private Key**: The Ethereum private key for your qualified address
2. **Recipient Address**: A **new Ethereum address** to receive the tokens (for maximum privacy, use a fresh address)

**Important**: The recipient address can be different from your qualified address. This enhances privacy since the two addresses are not cryptographically linked.

## Step 3: Generate Proof (Offline)

### Install the CLI Tool
```bash
cargo install zkp-airdrop-cli
```

### Generate Your Proof
```bash
zkp-airdrop generate-proof \
  --private-key $PRIVATE_KEY \
  --recipient $RECIPIENT_ADDRESS \
  --merkle-tree accounts.csv \
  --output proof.json
```

**What happens**:
1. The CLI reads `accounts.csv` to build the Merkle tree in memory
2. It proves (using zero-knowledge) that your private key corresponds to an address in the tree
3. It generates `proof.json` containing:
   - ZK proof that your address is eligible
   - Nullifier (prevents double-claiming)
   - Recipient address
   - Merkle root

**Privacy**: Your original qualified address is **never revealed** in the proof.

## Step 4: Claim Tokens (Choose Your Path)

### Option A: Use a Relayer (Free Gas)

**What are Relayers?**
- **Optional API services** (no frontend/UI) that pay gas fees on your behalf
- Funded by community donations
- Validate proofs off-chain first (to avoid wasting gas)
- Anyone can run a relayer - just deploy the API service
- CLI tool interacts directly with the REST API endpoints

**Submit via Relayer**:
```bash
zkp-airdrop submit \
  --proof proof.json \
  --relayer-url https://relayer.zkp-airdrop.io \
  --wait
```

**Deploy Your Own Relayer**:
```bash
cd relayer
cargo run --release
# API available at http://localhost:8080
# Configure with environment variables
```

**Benefits**:
- No ETH needed for gas
- Simple, one-command submission
- Community-supported

**Considerations**:
- Relayer sees your proof (but not your private key)
- Timing metadata could be observed

### Option B: Direct Submission (Maximum Privacy)

**Submit Directly to Contract**:
```bash
zkp-airdrop submit-direct \
  --proof proof.json \
  --rpc-url $OPTIMISM_RPC_URL \
  --private-key $RECIPIENT_PRIVATE_KEY \
  --wait
```

**Requirements**:
- ETH on Optimism for gas fees (~700,000 gas, but much cheaper than Ethereum)
- Optimism RPC endpoint (Alchemy, Infura, or public RPC)

**Benefits**:
- Maximum privacy (no third-party involved)
- No metadata leakage to relayers
- Complete control over transaction

## Key Principles

### 1. Contract Immutability
- The airdrop contract is **immutable** after deployment
- No admin keys, no upgrades, no changes possible
- Once deployed, the rules are fixed forever

### 2. Permissionless Access
- Anyone with a valid proof can call the contract
- No whitelists, no approvals needed
- Truly decentralized and censorship-resistant

### 3. Privacy by Design
- Your qualified address is never revealed on-chain
- Multiple claims from same address cannot be linked
- Recipient address is not tied to qualified address

### 4. Relayers are Optional
- You can **always** submit directly
- Relay services are conveniences, not requirements
- Multiple relayers available (no single point of failure)

## Frequently Asked Questions

### Q: What if I lose my private key?
**A**: You cannot claim without the private key. The system requires cryptographic proof of ownership.

### Q: Can I claim to multiple recipient addresses?
**A**: Yes, but each claim requires a separate proof with a different recipient address.

### Q: What's the gas cost?
**A**: Approximately 700,000 gas. On Optimism at 0.001 gwei, this is about 0.0007 ETH ($~2.10 at $3,000 ETH) - 50x cheaper than Ethereum mainnet.

### Q: How long do I have to claim?
**A**: 90 days from contract deployment. After that, unclaimed tokens are burned.

### Q: Can someone else claim my tokens?
**A**: No, only someone with your private key can generate a valid proof.

### Q: Is my privacy guaranteed?
**A**: The cryptography guarantees your qualified address is hidden. However, timing analysis and recipient address linking are possible (see Privacy Analysis document).

### Q: What if relayers run out of funds?
**A**: You can always submit directly and pay your own gas.

## Security Considerations

### Private Key Safety
- Never share your private key
- Generate proof offline if possible
- Use hardware wallets for key management

### Recipient Address
- Use a fresh address for maximum privacy
- Consider using privacy-enhanced wallets
- Be aware that token movements from recipient address can be traced

### Proof Generation
- Verify the CLI tool's integrity (checksums, signatures)
- Generate proofs in secure environments
- Delete proof.json after successful claim

## Getting Help

- **Documentation**: See `/docs` for technical details
- **Community**: Join Discord/Telegram for support
- **Code**: All code is open source and auditable
- **Verification**: You can verify the Merkle tree construction independently

## Summary

1. **Check**: Verify your address is in `accounts.csv`
2. **Prepare**: Have private key and recipient address ready
3. **Generate**: Create `proof.json` using the CLI
4. **Claim**: Submit via relayer (free) or directly (private)

The system balances privacy, accessibility, and decentralization to create a fair airdrop for active Ethereum users.