# ZKP Privacy Airdrop

A privacy-preserving ERC20 token airdrop system using zero-knowledge proofs. Allows 65M qualified Ethereum accounts to claim ZKP tokens without revealing their identity.

## Features

- **Privacy**: Claim without revealing which address is in the qualified list
- **Scalable**: Supports 65 million qualified accounts
- **Decentralized**: Multiple relayers can participate
- **Secure**: Formally verified circuits and audited contracts
- **User-friendly**: Simple Rust CLI for proof generation

## Architecture

```
Claimant → Rust CLI → ZK Proof → Relayer → Airdrop Contract → ZKP Tokens
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
  --format binary

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

- [Project Overview](docs/01-overview.md)
- [Technical Specification](docs/02-technical-specification.md)
- [Implementation Roadmap](docs/03-implementation-roadmap.md)
- [API Reference](docs/04-api-reference.md)
- [Security & Privacy](docs/05-security-privacy.md)

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
