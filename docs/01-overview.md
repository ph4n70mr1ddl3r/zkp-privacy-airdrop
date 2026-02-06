# ZKP Privacy Airdrop - Project Overview

## Project Summary

A privacy-preserving token airdrop system that allows 65,000,000 qualified Ethereum accounts to claim ZKP tokens without revealing their identity on-chain. The system uses zero-knowledge proofs (ZKPs) to verify claim eligibility while maintaining complete anonymity.

## Key Components

### 1. Smart Contracts (Solidity)
- **ZKP Token Contract**: ERC20 token contract for the ZKP token
- **Airdrop Contract**: Main contract handling proof verification and token distribution
- **Relayer Registry**: Contract managing relayer funding and operations

### 2. Rust CLI Tool
- Command-line interface for generating ZK proofs
- Takes private key and recipient address as inputs
- Generates proof of Merkle tree membership without revealing the specific leaf

### 3. Web Relayer Service
- Accepts ZK proofs from claimants
- Validates proofs off-chain before submission
- Submits valid proofs to the airdrop contract (pays gas)
- Accepts donations to fund gas costs (community-funded)
- Multiple independent relayers for redundancy
- Rate limiting and anti-spam measures
- Anyone can run a relayer - no central authority

### 4. Merkle Tree Infrastructure
- 65M account Merkle tree
- Off-chain storage and verification
- Tree root commitment on-chain

## Privacy Guarantees

- **Identity Privacy**: Claimant's original address is never revealed
- **Claim Unlinkability**: Multiple claims from the same account cannot be linked
- **Recipient Privacy**: Recipient address is not tied to the original qualified account
- **Zero Knowledge**: Proof reveals nothing except that the prover knows a private key in the list

## Architecture Flow

```
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│   Qualified     │     │    Rust CLI      │     │   Web Relayer   │
│    Account      │────▶│  (Proof Gen)     │────▶│    Service      │
│  (Private Key)  │     │                  │     │                 │
└─────────────────┘     └──────────────────┘     └────────┬────────┘
                                                          │
                                                          ▼
                                               ┌──────────────────┐
                                               │  Airdrop Contract │
                                               │  (Verify & Mint)  │
                                               └──────────────────┘
```

## Token Economics

- **Token Name**: ZKP
- **Token Type**: ERC20 (18 decimals)
- **Total Supply**: 65,000,000,000 ZKP (65,000,000 accounts × 1000 ZKP each)
- **Claim Amount**: 1000 ZKP per qualified account
- **Distribution Method**: Privacy-preserving claim
- **Claim Period**: 90 days from deployment

## Success Criteria

1. Private key holders can generate valid proofs
2. Relayer successfully validates and submits proofs
3. Tokens are minted to recipient addresses
4. No correlation possible between qualified list and claim transactions
5. System scales to handle 65,000,000 potential claimants
6. Relayer ecosystem remains funded through community donations
