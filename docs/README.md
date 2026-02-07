# ZKP Privacy Airdrop Documentation

## Overview

This is a comprehensive documentation suite for the ZKP Privacy Airdrop project - a privacy-preserving token distribution system using zero-knowledge proofs.

**Important**: [00-specification.md](00-specification.md) is the single source of truth for all technical constants, specifications, and data formats. Other documents reference and elaborate on these specifications.

## Quick Start

1. **Read the Unified Specification**: [00-specification.md](00-specification.md) - **Start here for authoritative constants and interfaces**
2. **Read the Overview**: [01-overview.md](01-overview.md)
3. **Technical Details**: [02-technical-specification.md](02-technical-specification.md)
4. **Implementation Plan**: [03-implementation-roadmap.md](03-implementation-roadmap.md)
5. **API Reference**: [04-api-reference.md](04-api-reference.md)
6. **Security & Privacy**: [05-security-privacy.md](05-security-privacy.md)
7. **Privacy Analysis**: [06-privacy-analysis.md](06-privacy-analysis.md)
8. **Consistency Guide**: [07-consistency-checklist.md](07-consistency-checklist.md)
9. **Recent Fixes**: [documentation-fixes-summary.md](documentation-fixes-summary.md)

## Project Structure

```
docs/
├── README.md                          # This file
├── 00-specification.md                # Unified specification (single source of truth)
├── 01-overview.md                     # Project overview and goals
├── 02-technical-specification.md      # Detailed technical specs
├── 03-implementation-roadmap.md       # Development timeline
├── 04-api-reference.md                # API and CLI reference
├── 05-security-privacy.md             # Security considerations
├── 06-privacy-analysis.md             # Privacy limitations and analysis
├── 07-consistency-checklist.md        # Documentation consistency guide
└── documentation-fixes-summary.md     # Summary of documentation fixes

contracts/                             # Smart contracts
├── ZKPToken.sol
├── PrivacyAirdrop.sol
├── RelayerRegistry.sol
└── Verifier.sol

circuits/                              # ZK circuits
└── merkle_membership.circom

cli/                                   # Rust CLI tool
└── src/
    ├── main.rs
    ├── proof_generator.rs
    ├── merkle_tree.rs
    └── submitter.rs

relayer/                               # API relayer service (no frontend)
└── src/
    ├── main.rs
    ├── api.rs
    ├── contract.rs
    └── rate_limiter.rs

tree-builder/                          # Merkle tree construction
└── src/
    └── build_tree.rs

tests/                                 # Test suite
├── unit/
├── integration/
└── e2e/
```

## Key Features

- **Privacy**: Claim tokens without revealing your identity
- **Scalability**: Supports 65,249,064 qualified accounts
- **Decentralization**: Anyone can run a relayer
- **Security**: Audited contracts and formally verified circuits
- **User-Friendly**: Simple CLI tool for proof generation

## System Architecture

```
┌──────────────┐      ┌──────────────┐      ┌──────────────┐
│   Qualified  │──────▶│  Rust CLI    │──────▶│ API Relayer  │
│   Account    │      │  (ZK Proof)  │      │  (Optional)  │
└──────────────┘      └──────────────┘      └──────┬───────┘
                                                    │
                           ┌────────────────────────┘
                           ▼
                    ┌──────────────┐
                    │   Airdrop    │
                    │   Contract   │
                    └──────────────┘

**Note**: Users can also submit directly to the contract, bypassing the relayer.
Relayers are API services with no frontend - CLI tools interact directly via REST API.
```

## Getting Started

### Prerequisites

- Rust 1.70+
- Node.js 18+ (for contracts)
- Circom 2.1+
- Solidity 0.8+

### Installation

```bash
# Clone repository
git clone https://github.com/yourorg/zkp-airdrop
cd zkp-airdrop

# Build CLI
cd cli
cargo build --release

# Install contract dependencies
cd ../contracts
npm install

# Compile circuits
cd ../circuits
make compile
```

### Usage

```bash
# Generate proof
./target/release/zkp-airdrop generate-proof \
  --private-key $PRIVATE_KEY \
  --recipient $RECIPIENT \
  --merkle-tree ./tree.bin

# Submit to relayer
./target/release/zkp-airdrop submit \
  --proof ./proof.json \
  --relayer-url https://relayer.zkp-airdrop.io \
  --recipient $RECIPIENT \
  --wait
```

## Contributing

1. Read the [technical specification](02-technical-specification.md)
2. Check the [implementation roadmap](03-implementation-roadmap.md) for open tasks
3. Follow security guidelines in [security-privacy.md](05-security-privacy.md)
4. Submit PR with comprehensive tests

## Security

For security issues, please email security@zkp-airdrop.io instead of using the issue tracker.

## License

MIT License - See [LICENSE](../LICENSE) for details

## Resources

- **Website**: https://zkp-airdrop.io
- **Documentation**: https://docs.zkp-airdrop.io
- **Discord**: https://discord.gg/zkp-airdrop
- **Twitter**: https://twitter.com/zkp_airdrop

## Acknowledgments

- [Circom](https://github.com/iden3/circom) - ZK circuit compiler
- [Arkworks](https://arkworks.rs/) - Rust ZK libraries
- [OpenZeppelin](https://openzeppelin.com/) - Secure contract library
