# ZKP Privacy Airdrop - Project Overview

**Version**: 1.1.0  
**Last Updated**: 2026-02-07  
**Based on**: [Unified Specification](./00-specification.md)

> **Note**: This document is derived from the [Unified Specification](./00-specification.md). Any discrepancies should be resolved in favor of the unified specification. For detailed technical specifications, cryptographic constants, and data formats, always refer to the unified specification.

## Project Summary

A privacy-preserving token airdrop system that allows 65,249,064 qualified Ethereum accounts to claim ZKP tokens on Optimism without revealing their identity on-chain. The system uses zero-knowledge proofs (ZKPs) to verify claim eligibility while maintaining complete anonymity.

**Deployed on Optimism**: Leveraging Layer 2 scaling for 10-100x lower gas fees compared to Ethereum mainnet.

### Qualified Accounts Criteria
The 65,249,064 eligible accounts are Ethereum mainnet addresses that paid at least **0.004 ETH in gas fees** from genesis until **December 31, 2025**. This creates a fair and wide distribution to active Ethereum users.

**Accounts File**: Available at https://drive.google.com/file/d/1yvgsuDMhamUoKAfH59iuyDtm7x5mnHRX/view?usp=sharing (download with `gdown 1yvgsuDMhamUoKAfH59iuyDtm7x5mnHRX`)

## Key Components

### 1. Smart Contracts (Solidity)
- **ZKP Token Contract**: ERC20 token contract for the ZKP token
- **Airdrop Contract**: Main contract handling proof verification and token distribution
- **Relayer Registry**: Contract managing relayer funding and operations

### 2. Rust CLI Tool
- Command-line interface for generating ZK proofs
- Takes private key and recipient address as inputs
- Generates proof of Merkle tree membership without revealing the specific leaf

### 3. API Relayer Service
- **Pure API service** with no frontend/UI
- Accepts ZK proofs from claimants via REST API
- Validates proofs off-chain before submission (saves gas)
- Submits valid proofs to the airdrop contract (pays gas)
- Accepts donations to fund gas costs (community-funded)
- Multiple independent relayers for redundancy
- Rate limiting and anti-spam measures
- CLI tool interacts directly with API endpoints
- Anyone can run a relayer - no central authority

### 4. Merkle Tree Infrastructure
- 65,249,064 account Merkle tree (from accounts.csv)
- Off-chain storage and verification
- Tree root commitment on-chain

## Privacy Guarantees

- **Identity Privacy**: Claimant's original address is never revealed
- **Claim Unlinkability**: Multiple claims from the same account cannot be linked
- **Recipient Privacy**: Recipient address is not tied to the original qualified account
- **Zero Knowledge**: Proof reveals nothing except that the prover knows a private key in the list

## User Workflow

### Step-by-Step Process
1. **Check Eligibility**: Verify your address is in `accounts.csv` (65M+ addresses that paid ≥0.004 ETH gas fees by Dec 31, 2025)
2. **Generate Proof**: Use CLI with private key and recipient address → creates `proof.json`
3. **Submit Claim**: Choose either:
   - **Option A**: Use relayer (free gas, community-funded)
   - **Option B**: Submit directly (pay your own gas, maximum privacy)

### Architecture Flow

```
                          ┌─────────────────┐
                          │   Qualified     │
                          │    Account      │
                          │  (Private Key)  │
                          └────────┬────────┘
                                   │
                                   ▼
                          ┌──────────────────┐
                          │    Rust CLI      │
                          │  (Proof Gen)     │─────────────┐
                          └──────────────────┘             │
                                   │                       │
                                   │ (proof.json)          │
                          ┌────────▼────────┐    ┌─────────▼────────┐
                          │   API Relayer   │    │   Direct User    │
                          │  (Optional)     │    │   Submission     │
                          │  • Free gas     │    │  • Pay own gas   │
                          │  • Community    │    │  • Max privacy   │
                          │    funded       │    └─────────┬────────┘
                          │  • REST API     │             │
                          │  • No frontend  │             │
                          └────────┬────────┘             │
                                   │                      │
                                   └──────────┬───────────┘
                                              │
                                              ▼
                                     ┌──────────────────┐
                                     │  Airdrop Contract │
                                     │  (Immutable)      │
                                     │  (Verify & Mint)  │
                                     └──────────────────┘
```

**Key Points**:
- **Contract is immutable** - cannot be modified after deployment
- **Anyone can claim directly** - no requirement to use relayers
- **Relayers are optional** - provide free gas via community donations
- **Proof validation happens off-chain first** - relayers check before submitting to avoid wasting gas

## Token Economics

- **Token Name**: ZKP
- **Token Type**: ERC20 (18 decimals)
- **Total Supply**: 65,249,064,000 ZKP (65,249,064 accounts × 1,000 ZKP each)
- **Claim Amount**: 1000 ZKP per qualified account
- **Distribution Method**: Privacy-preserving claim
- **Claim Period**: 90 days from deployment

## Success Criteria

1. Private key holders can generate valid proofs
2. Relayer successfully validates and submits proofs
3. Tokens are minted to recipient addresses
4. No correlation possible between qualified list and claim transactions
5. System scales to handle 65,249,064 potential claimants
6. Relayer ecosystem remains funded through community donations
