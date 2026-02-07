# ZKP Privacy Airdrop

A privacy-preserving ERC20 token airdrop system using zero-knowledge proofs. Allows 65,249,064 qualified Ethereum accounts to claim ZKP tokens on Optimism without revealing their identity.

## Features

- **Privacy**: Claim without revealing which address is in the qualified list
- **Scalable**: Supports 65,249,064 qualified accounts
- **Decentralized**: Optional relayers or direct contract submission
- **Secure**: Formally verified circuits and audited contracts
- **User-friendly**: Simple Rust CLI for proof generation

## Architecture

```
Claimant → Rust CLI → ZK Proof → [API Relayer (Optional)] → Airdrop Contract → ZKP Tokens
                            ↓                                  (Optimism)
                     Direct Submission (Alternative)

Key Points:
- **Optimism Network**: Deployed on Optimism for low gas fees (~10-100x cheaper than Ethereum)
- **Relayer is a REST API service** with no frontend
- CLI tool submits proofs directly to API endpoints  
- Users can also submit directly to contract (pay own gas)
- Multiple relayers can run independently
- All relayers use same contract - no centralization
```

## Quick Start

### Claimant Workflow

1. **Download the qualified accounts list**:
   ```bash
   gdown 1yvgsuDMhamUoKAfH59iuyDtm7x5mnHRX
   ```
   This file contains 65,249,064 Ethereum addresses that paid at least 0.004 ETH in gas fees from genesis until December 31, 2025.

2. **Prepare your credentials**:
   - **Private Key**: Your Ethereum private key for a qualified account (from the accounts list)
   - **Recipient Address**: A new Ethereum address to receive the tokens (can be different from your qualified account)

3. **Generate proof (private, offline)**:
   ```bash
   # Install CLI
   cargo install zkp-airdrop-cli
   
   # Generate proof using the accounts file
   zkp-airdrop generate-proof \
     --private-key $PRIVATE_KEY \
     --recipient $RECIPIENT_ADDRESS \
     --merkle-tree accounts.csv \
     --output proof.json
   ```
   The `proof.json` file contains everything needed to claim your tokens.

4. **Claim your tokens** (choose one option):

   **Option A: Submit via relayer (free gas)**:
   ```bash
   zkp-airdrop submit \
     --proof proof.json \
     --relayer-url https://relayer.zkp-airdrop.io \
     --wait
   ```
   The relayer is an optional web service that pays gas fees on your behalf, funded by community donations.

   **Option B: Submit directly (pay your own gas)**:
   ```bash
   zkp-airdrop submit-direct \
     --proof proof.json \
     --rpc-url $ETHEREUM_RPC_URL \
     --private-key $RECIPIENT_PRIVATE_KEY \
     --wait
   ```
   You can interact directly with the immutable airdrop contract if you prefer to pay your own gas.

### Important Notes

- **Contract Immutability**: The airdrop contract is immutable after deployment. Anyone with a valid proof can claim tokens.
- **Single Claim Guarantee**: Each qualified account can only claim once, regardless of recipient address.
- **Privacy**: Your original qualified address is never revealed on-chain.
- **Relayers are Optional**: You can always submit directly to the contract if you prefer.
- **Community Funded**: Relay services are funded by donations and provide free gas to claimants.

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
├── relayer/            # API relayer service (no frontend)
├── tree-builder/       # Merkle tree construction
└── tests/              # Test suite
```

## Development

### Prerequisites

- Rust 1.70+
- Node.js 18+
- Circom 2.1+
- gdown (for downloading the accounts file)

### Data Files

The qualified accounts list is available at:
- **Source**: https://drive.google.com/file/d/1yvgsuDMhamUoKAfH59iuyDtm7x5mnHRX/view?usp=sharing
- **Download Command**: `gdown 1yvgsuDMhamUoKAfH59iuyDtm7x5mnHRX`

This file contains 65,249,064 Ethereum addresses (20 bytes each, 1.216 GiB total) that are eligible for the airdrop.

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
