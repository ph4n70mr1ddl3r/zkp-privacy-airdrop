# ZKP Privacy Airdrop

A privacy-preserving ERC20 token airdrop system using zero-knowledge proofs. Allows 65,249,064 qualified Ethereum accounts to claim ZKP tokens without revealing their identity.

## Features

- **Privacy**: Claim without revealing which address is in the qualified list
- **Scalable**: Supports 65,249,064 qualified accounts
- **Decentralized**: Optional relayers or direct contract submission
- **Secure**: Formally verified circuits and audited contracts
- **User-friendly**: Simple Rust CLI for proof generation

## Architecture

```
Claimant → Rust CLI → ZK Proof → [Relayer (Optional)] → Airdrop Contract → ZKP Tokens
                            ↓
                     Direct Submission (Alternative)
```

## Quick Start

### For Claimants

```bash
# Install CLI
cargo install zkp-airdrop-cli

# Generate proof (private)
zkp-airdrop generate-proof \
  --private-key $PRIVATE_KEY \
  --recipient $RECIPIENT_ADDRESS \
  --merkle-tree https://api.merkle-tree.io/tree.bin \
  --output proof.json

# Submit claim via relayer
zkp-airdrop submit \
  --proof proof.json \
  --relayer-url https://relayer.zkp-airdrop.io \
  --wait
```

### For Relayers

```bash
# Run relayer
cd relayer
cargo run --release

# Or with Docker
docker-compose up -d
```

## Documentation

**Important**: All technical constants, specifications, and data formats are defined in the [Unified Specification](docs/00-specification.md), which is the single source of truth. Other documents reference and elaborate on these specifications.

- [Unified Specification](docs/00-specification.md) - **Single source of truth** for all technical details, constants, and interfaces
- [Project Overview](docs/01-overview.md) - High-level introduction and architecture
- [Technical Specification](docs/02-technical-specification.md) - Detailed implementation specifications
- [Implementation Roadmap](docs/03-implementation-roadmap.md) - Development timeline and phases
- [API Reference](docs/04-api-reference.md) - External interfaces and data formats
- [Security & Privacy](docs/05-security-privacy.md) - Security considerations and threat model
- [Privacy Analysis & Limitations](docs/06-privacy-analysis.md) - Privacy guarantees and limitations
- [Consistency Checklist](docs/07-consistency-checklist.md) - Verification guide for documentation consistency

## Project Structure

```
├── docs/               # Documentation
├── contracts/          # Solidity smart contracts
├── circuits/           # Circom ZK circuits
├── cli/                # Rust CLI tool
├── relayer/            # Web relayer service
├── tree-builder/       # Merkle tree construction
└── tests/              # Test suite
```

## Development

### Prerequisites

- Rust 1.70+
- Node.js 18+
- Circom 2.1+

### Build

```bash
# Build everything
make build

# Run tests
make test

# Run integration tests
make test-integration
```

## Security

This project involves significant value and sensitive cryptographic operations. Please review the [security documentation](docs/05-security-privacy.md) carefully.

For security issues, contact: security@zkp-airdrop.io

## Contributing

1. Read our [technical specification](docs/02-technical-specification.md)
2. Check open issues and roadmap
3. Submit PR with tests

## License

MIT License - see [LICENSE](LICENSE) file

## Acknowledgments

- [Circom](https://github.com/iden3/circom)
- [Arkworks](https://arkworks.rs/)
- [OpenZeppelin](https://openzeppelin.com/)
