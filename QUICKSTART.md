# ZKP Privacy Airdrop - Quick Start Guide

This guide will help you get started with the ZKP Privacy Airdrop system.

## Prerequisites

Before you begin, ensure you have the following installed:

- **Rust**: 1.70 or later ([Install](https://rustup.rs/))
- **Node.js**: 18 or later ([Install](https://nodejs.org/))
- **Circom**: 2.1 or later
- **PostgreSQL**: 15 or later (for relayer)
- **Redis**: 7 or later (for relayer)
- **Docker**: (optional, for running relayer with infrastructure)
- **Docker Compose**: (optional)

## Installation

### Option 1: Build from Source

```bash
# Clone the repository
git clone https://github.com/yourorg/zkp-privacy-airdrop.git
cd zkp-privacy-airdrop

# Build all components
make build

# Install CLI tool
cd cli && cargo install --path .
```

### Option 2: Use Docker Compose

```bash
# Build and run the relayer infrastructure
docker-compose up -d
```

## Setting Up the Relayer

### Using Docker Compose (Recommended)

```bash
# Copy example environment file
cp relayer/.env.example .env

# Edit .env with your configuration
nano .env

# Start all services
docker-compose up -d

# Check logs
docker-compose logs -f relayer
```

### Manual Setup

1. **Install PostgreSQL and Redis**

```bash
# On Ubuntu/Debian
sudo apt-get install postgresql redis-server

# On macOS
brew install postgresql redis
```

2. **Set up environment variables**

```bash
export DATABASE_URL="postgresql://postgres:postgres@localhost:5432/zkp_airdrop"
export REDIS_URL="redis://localhost:6379"
export RPC_URL="https://optimism.drpc.org"
export RELAYER_PRIVATE_KEY="your-private-key-here"
```

3. **Run the relayer**

```bash
cd relayer
cargo run --release
```

## Building the Merkle Tree

The Merkle tree must be built from the qualified accounts list before the system can be used.

```bash
# Download the accounts file
gdown 1yvgsuDMhamUoKAfH59iuyDtm7x5mnHRX

# Build the Merkle tree
cd tree-builder
cargo run --release \
  --accounts-file ../accounts.csv \
  --output merkle_tree.bin \
  --verify

# The output will show the Merkle root, which you'll need for contract deployment
```

## Using the CLI

### Generate a Proof

```bash
# Set your private key (you can also use --private-key-file or --private-key-stdin)
export PRIVATE_KEY=0x1234...

# Generate a proof
zkp-airdrop generate-proof \
  --recipient 0x5678... \
  --merkle-tree merkle_tree.bin \
  --output my_proof.json
```

### Verify a Proof Locally

```bash
zkp-airdrop verify-proof \
  --proof my_proof.json \
  --merkle-root 0x1234...
```

### Submit a Claim via Relayer

```bash
zkp-airdrop submit \
  --proof my_proof.json \
  --relayer-url http://localhost:8080 \
  --recipient 0x5678... \
  --wait
```

### Check Claim Status

```bash
zkp-airdrop check-status \
  --nullifier 0xabc... \
  --relayer-url http://localhost:8080
```

### Download the Merkle Tree

```bash
zkp-airdrop download-tree \
  --source https://api.merkle-tree.io/tree.bin \
  --output merkle_tree.bin \
  --verify
```

## Managing CLI Configuration

```bash
# Show current configuration
zkp-airdrop config show

# Set default relayer URL
zkp-airdrop config set relayer_url https://relayer.zkp-airdrop.io

# Set network
zkp-airdrop config set network optimism

# Reset to defaults
zkp-airdrop config reset
```

## Deploying Smart Contracts

### Compile Contracts

```bash
cd contracts
npm install
npm run compile
```

### Deploy to Testnet

```bash
# Set your private key
export PRIVATE_KEY=0x1234...

# Deploy to Optimism Sepolia
npm run deploy:testnet
```

### Deploy to Mainnet

```bash
# Deploy to Optimism mainnet
npm run deploy:mainnet
```

## Testing

### Run All Tests

```bash
make test
```

### Run Component Tests

```bash
# Contracts
cd contracts && npm test

# CLI
cd cli && cargo test

# Relayer
cd relayer && cargo test

# Tree builder
cd tree-builder && cargo test
```

### Run Integration Tests

```bash
# Start the relayer first
docker-compose up -d

# Run integration tests
make test-integration
```

## Monitoring

### Access Grafana Dashboard

```bash
# Grafana will be available at http://localhost:3000
# Default credentials: admin / admin
```

### Check Relayer Health

```bash
curl http://localhost:8080/api/v1/health
```

### View Prometheus Metrics

```bash
# Prometheus will be available at http://localhost:9090
curl http://localhost:8080/metrics
```

## Troubleshooting

### Relayer won't start

1. Check that PostgreSQL and Redis are running:
```bash
sudo systemctl status postgresql
sudo systemctl status redis
```

2. Check the environment variables:
```bash
echo $DATABASE_URL
echo $REDIS_URL
```

3. Check the logs:
```bash
docker-compose logs relayer
```

### Proof generation fails

1. Verify that the Merkle tree file exists and is valid
2. Check that your private key is 32 bytes (64 hex characters)
3. Ensure the recipient address is a valid Ethereum address

### Transaction submission fails

1. Check that the relayer has sufficient ETH balance
2. Verify the network is correct (Optimism vs Optimism Sepolia)
3. Check that the nullifier hasn't been used before

### Rate limit errors

1. Wait for the rate limit to expire (default: 60 seconds)
2. Check the relayer logs for rate limit details
3. Consider increasing the rate limit in the configuration

## Next Steps

- Read the full documentation in the `/docs` directory
- Review the [API Reference](docs/04-api-reference.md)
- Understand the [Security Considerations](docs/05-security-privacy.md)
- Learn about [Privacy Limitations](docs/06-privacy-analysis.md)

## Support

- Documentation: See `/docs` directory
- Issues: Create an issue on GitHub
- Email: support@zkp-airdrop.io
