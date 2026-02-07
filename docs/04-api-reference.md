# API Reference

**Version**: 1.1.0  
**Last Updated**: 2026-02-07  
**Based on**: [Unified Specification](../docs/00-specification.md)

> **Note**: All data formats and constants are defined in the [Unified Specification](../docs/00-specification.md).
> 
> **Implementation Status**: This is a specification document. Implementation timeline is detailed in the [Implementation Roadmap](./03-implementation-roadmap.md).

## Relayer API

**Note**: The relayer is a REST API service with no frontend/UI. The CLI tool interacts directly with these endpoints.

Base URL: `https://relayer.zkp-airdrop.io/api/v1`

### Authentication

Most endpoints are public. Monitoring endpoints may require API key:
```
Authorization: Bearer <API_KEY>
```

### Endpoints

#### Submit Claim

Submits a zero-knowledge proof to claim tokens. The relayer validates the proof off-chain before submitting to the contract to save gas costs.

**Note**: The smart contract allows anyone to submit claims directly. Relayers are optional services that pay gas fees on behalf of users. Users can also submit claims directly to the contract if they prefer to pay their own gas.

```http
POST /api/v1/submit-claim
Content-Type: application/json
```

**Request Body**:
```json
{
  "proof": {
    "a": [
      "13862987149607610235678184535533251295074929736392939725598345555223684473689",
      "15852461416563938980812664669205586669275551636381044234656441244716521728494"
    ],
    "b": [
      [
        "5271136692644488661472090380084300860023621341105994559822360935366466488598",
        "13087383351388148199576676131705235587076492997725459455618630929222583122567"
      ],
      [
        "11577348146760796615264785176417792290215623721746201176452539864784075498810",
        "17893729721208687510330180286659033807865545010318659797787524576669037031211"
      ]
    ],
    "c": [
      "12509499717138495769595382836457599601032647877926581706768198432092263516957",
      "12074485721490120286767132312724602681882230534725439082885525839982480799988"
    ]
  },
  "public_signals": [
    "12506683786903428657580826970343219399794309499408177282243657255115537496844",
    "14595410858393345558982908440260919580883831523172723621649567175847460824507",
    "1092345678901234567890123456789012345678901234567890123456789012"
  ],
  "nullifier": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
  "recipient": "0x1234567890123456789012345678901234567890",
  "merkle_root": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
  "generated_at": "2024-01-15T10:30:00Z"
}
```

**Note**: The `public_signals` array contains three elements in this order:
1. `merkle_root` (field element in decimal string format)
2. `recipient` (address converted to field element in decimal string format)  
3. `nullifier` (field element in decimal string format)

The `nullifier` and `merkle_root` fields are also provided as hex strings for convenience, but the `public_signals` array uses decimal string format for field elements.

**Response** (200 OK):
```json
{
  "success": true,
  "tx_hash": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
  "status": "pending",
  "estimated_confirmation": "2024-01-15T10:30:00Z"
}
```

**Response** (400 Bad Request):
```json
{
  "success": false,
  "error": "Invalid proof format",
  "code": "INVALID_PROOF"
}
```

**Response** (429 Too Many Requests):
```json
{
  "success": false,
  "error": "Rate limit exceeded. Try again in 60 seconds.",
  "code": "RATE_LIMITED",
  "retry_after": 60
}
```

**Response** (503 Service Unavailable):
```json
{
  "success": false,
  "error": "Relayer temporarily unavailable due to insufficient funds",
  "code": "INSUFFICIENT_FUNDS",
  "donation_address": "0x..."
}
```

#### Check Claim Status

Check if a nullifier has been used to claim tokens.

```http
GET /api/v1/check-status/{nullifier}
```

**Response** (200 OK):
```json
{
  "nullifier": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
  "claimed": true,
  "tx_hash": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
  "recipient": "0x1234567890123456789012345678901234567890",
  "timestamp": "2024-01-15T10:30:00Z",
  "block_number": 18972345
}
```

**Response** (200 OK - Not Claimed):
```json
{
  "nullifier": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
  "claimed": false
}
```

#### Get Merkle Root

Returns the current Merkle root used by the airdrop contract.

```http
GET /api/v1/merkle-root
```

**Response** (200 OK):
```json
{
  "merkle_root": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
  "block_number": 18972300,
  "timestamp": "2024-01-15T10:00:00Z"
}
```

#### Get Contract Info

Returns information about the deployed contracts.

```http
GET /api/v1/contract-info
```

**Response** (200 OK):
```json
{
  "network": "optimism",
  "chain_id": 1,
  "contracts": {
    "airdrop": {
      "address": "0x1234567890123456789012345678901234567890",
      "deployed_at": "2024-01-01T00:00:00Z",
      "block_number": 18970000
    },
    "token": {
      "address": "0xabcdef1234567890abcdef1234567890abcdef12",
      "symbol": "ZKP",
      "decimals": 18
    },
    "relayer_registry": {
      "address": "0x..."
    }
  },
  "claim_amount": "1000000000000000000000", // 1000 ZKP tokens (18 decimals)
  "claim_deadline": "2024-04-01T00:00:00Z"
}
```

#### Donate

Send ETH to fund the relayer's gas costs.

```http
POST /api/v1/donate
Content-Type: application/json
```

**Request Body**:
```json
{
  "amount": "1000000000000000000",
  "donor": "0x1234567890123456789012345678901234567890"
}
```

**Note**: Donations can be sent directly to the relayer's Ethereum address. This endpoint provides the donation address and tracks donation intents for transparency.

**Response** (200 OK):
```json
{
  "donation_address": "0x1234567890123456789012345678901234567890",
  "amount_received": "1000000000000000000",
  "tx_hash": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
  "thank_you": "Thank you for supporting privacy!"
}
```

#### Get Relayer Stats

Returns statistics about the relayer's operations.

```http
GET /api/v1/stats
```

**Response** (200 OK):
```json
{
  "total_claims": 15420,
  "successful_claims": 15400,
  "failed_claims": 20,
  "total_tokens_distributed": "15400000000000000000000000",
  "unique_recipients": 15400,
  "average_gas_price": "25000000000",
  "total_gas_used": "3080000000000000000",
  "relayer_balance": "5000000000000000000",
  "uptime_percentage": 99.98,
  "response_time_ms": {
    "p50": 150,
    "p95": 500,
    "p99": 1200
  }
}
```

#### Health Check

Check if the relayer is operational.

```http
GET /api/v1/health
```

**Response** (200 OK):
```json
{
  "status": "healthy",
  "timestamp": "2024-01-15T10:30:00Z",
  "version": "1.0.0",
  "services": {
    "database": "connected",
    "redis": "connected",
    "optimism_node": "connected",
    "relayer_wallet": {
      "address": "0x...",
      "balance": "5000000000000000000",
      "sufficient": true
    }
  }
}
```

**Response** (503 Service Unavailable):
```json
{
  "status": "unhealthy",
  "timestamp": "2024-01-15T10:30:00Z",
  "services": {
    "database": "connected",
    "redis": "connected",
    "optimism_node": "disconnected",
    "relayer_wallet": {
      "address": "0x...",
      "balance": "10000000000000000",
      "sufficient": false
    }
  },
  "errors": [
    "Ethereum node connection failed",
    "Relayer balance below threshold"
  ]
}
```

#### Get Merkle Path (Optional)

If running a full API service, return the Merkle path for an address.

```http
GET /api/v1/merkle-path/{address}
```

**Response** (200 OK):
```json
{
  "address": "0x1234567890123456789012345678901234567890",
  "leaf_index": 1234567,
  "merkle_path": [
    "0xabc...",
    "0xdef...",
    "0xghi..."
  ],
  "path_indices": [0, 1, 0, 1, 1, 0, ...],
  "root": "0x1234567890abcdef..."
}
```

**Response** (404 Not Found):
```json
{
  "error": "Address not found in Merkle tree",
  "code": "ADDRESS_NOT_FOUND"
}
```

### Rate Limits

Rate limits are applied per endpoint to prevent abuse while maintaining service availability:

- **Submit Claim**: 1 request per 60 seconds per nullifier
- **Check Claim Status**: 10 requests per 60 seconds per nullifier  
- **Get Merkle Path**: 60 requests per 60 seconds per IP address
- **General API**: 100 requests per 60 seconds per IP address
- **Burst Allowance**: 2x limit for 10 seconds

All rate limits are shared across all relayers in the ecosystem. Exceeding limits returns HTTP 429 with `Retry-After` header.

### Error Codes

| Code | Description |
|------|-------------|
| `INVALID_PROOF` | The ZK proof is malformed or invalid |
| `INVALID_NULLIFIER` | Nullifier hash is invalid |
| `ALREADY_CLAIMED` | This nullifier has already been used |
| `RATE_LIMITED` | Too many requests, retry after delay |
| `INSUFFICIENT_FUNDS` | Relayer has insufficient ETH for gas |
| `CONTRACT_ERROR` | Smart contract interaction failed |
| `NETWORK_ERROR` | Ethereum network error |
| `INTERNAL_ERROR` | Internal server error |
| `ADDRESS_NOT_FOUND` | Address not in Merkle tree |

### Error Handling & Recovery

#### Retry Strategies
- **Rate Limit Errors (429)**: Wait for the duration specified in `Retry-After` header
- **Network Errors**: Retry with exponential backoff (1s, 2s, 4s, 8s, 16s, 32s, 64s, 128s)
- **Temporary Failures** (insufficient funds, contract errors): Retry after 1 minute
- **Permanent Failures** (invalid proof, already claimed): Do not retry

#### Recovery Procedures
1. **Proof Generation Failures**:
   - Regenerate proof with same inputs
   - Verify private key and recipient address format
   - Check Merkle tree file integrity

2. **Submission Failures**:
   - For relayer submission: Try alternative relayer or submit directly
   - For direct submission: Check gas price, nonce, and network connectivity
   - Verify contract address and chain ID

3. **Common Issues**:
   - **Gas Price Too High**: Wait for lower network congestion
   - **Nonce Issues**: Reset transaction nonce
   - **Insufficient Balance**: Add ETH to relayer or user wallet
   - **Network Congestion**: Increase gas price or wait

## CLI Reference

### Installation

```bash
# Install from source
git clone https://github.com/yourorg/zkp-airdrop
cd zkp-airdrop/cli
cargo build --release

# Or install via cargo
cargo install zkp-airdrop-cli
```

### Global Options

```
--config <FILE>     Path to config file [default: ~/.zkp-airdrop/config.toml]
--network <NETWORK> Network to use (optimism, optimism-sepolia) [default: optimism]
-v, --verbose       Enable verbose output
-q, --quiet         Suppress output except errors
-h, --help          Print help
-V, --version       Print version
```

### Commands

#### `generate-proof`

Generate a zero-knowledge proof for claiming tokens.

```bash
zkp-airdrop generate-proof \
  --private-key <PRIVATE_KEY> \
  --recipient <RECIPIENT_ADDRESS> \
  --merkle-tree <TREE_SOURCE> \
  --output <OUTPUT_FILE>
```

**Arguments**:
- `--private-key <KEY>`: Ethereum private key (hex, with or without 0x)
  - Can also use env var: `PRIVATE_KEY`
  - Or read from file: `--private-key-file <PATH>`
  - Or read from stdin: `--private-key-stdin`
- `--recipient <ADDRESS>`: Address to receive tokens (must be valid Ethereum address)
- `--merkle-tree <SOURCE>`: Path to Merkle tree file or API URL
  - Supports: local file path, HTTP/HTTPS URL, IPFS hash
- `--output <FILE>`: Output file for proof JSON [default: proof.json]
- `--format <FORMAT>`: Output format (json, hex, raw) [default: json]
  - `json`: JSON format with proof and public signals
  - `hex`: Hexadecimal strings for smart contract calls
  - `raw`: Binary format for direct submission

**Example**:
```bash
# Using environment variable
export PRIVATE_KEY=0x1234...
zkp-airdrop generate-proof \
  --recipient 0x5678... \
  --merkle-tree https://api.merkle-tree.io/tree.bin \
  --output my-proof.json

# Reading from stdin
echo "0x1234..." | zkp-airdrop generate-proof \
  --private-key-stdin \
  --recipient 0x5678... \
  --merkle-tree ./merkle_tree.bin
```

**Output** (JSON format):
```json
{
  "proof": {
    "a": ["...", "..."],
    "b": [["...", "..."], ["...", "..."]],
    "c": ["...", "..."]
  },
  "public_signals": ["...", "...", "..."],
  "nullifier": "0x...",
  "recipient": "0x...",
  "merkle_root": "0x...",
  "generated_at": "2024-01-15T10:30:00Z"
}
```

#### `verify-proof`

Verify a proof locally without submitting.

```bash
zkp-airdrop verify-proof \
  --proof <PROOF_FILE> \
  --merkle-root <ROOT>
```

**Arguments**:
- `--proof <FILE>`: Path to proof JSON file
- `--merkle-root <ROOT>`: Expected Merkle root (hex)
- `--verification-key <FILE>`: Path to verification key [optional]

**Example**:
```bash
zkp-airdrop verify-proof \
  --proof my-proof.json \
  --merkle-root 0x1234...
```

**Output**:
```
✓ Proof is valid
  Nullifier: 0x...
  Recipient: 0x...
  Merkle Root: 0x...
```

#### `submit`

Submit a proof to a relayer API endpoint.

```bash
zkp-airdrop submit \
  --proof <PROOF_FILE> \
  --relayer-url <URL> \
  --recipient <ADDRESS>
```

**Arguments**:
- `--proof <FILE>`: Path to proof JSON file
- `--relayer-url <URL>`: Relayer endpoint URL (e.g., https://relayer.zkp-airdrop.io)
- `--recipient <ADDRESS>`: Recipient address (must match proof)
- `--wait`: Wait for transaction confirmation
- `--timeout <SECONDS>`: Timeout for confirmation [default: 120]

**Example**:
```bash
zkp-airdrop submit \
  --proof my-proof.json \
  --relayer-url https://relayer.zkp-airdrop.io \
  --recipient 0x5678... \
  --wait
```

**Output**:
```
Submitting claim...
✓ Claim submitted successfully
  Transaction: 0x...
  Status: Confirmed
  Block: 18972345
  Gas used: 150000
```

#### `check-status`

Check if a claim has been processed.

```bash
zkp-airdrop check-status --nullifier <NULLIFIER_HASH>
```

**Arguments**:
- `--nullifier <HASH>`: Nullifier to check (32-byte hex string)
- `--relayer-url <URL>`: Relayer endpoint URL
- `--rpc-url <URL>`: Ethereum RPC URL (alternative to relayer)

**Example**:
```bash
zkp-airdrop check-status \
  --nullifier 0xabcdef... \
  --relayer-url https://relayer.zkp-airdrop.io
```

**Output** (Claimed):
```
✓ Tokens claimed
  Nullifier: 0xabcdef...
  Recipient: 0x5678...
  Transaction: 0x...
  Block: 18972345
  Timestamp: 2024-01-15 10:30:00 UTC
```

**Output** (Not Claimed):
```
✗ Not claimed
  Nullifier: 0xabcdef...
  You can submit a claim using: zkp-airdrop submit --proof <PROOF>
```

#### `download-tree`

Download the Merkle tree for local use.

```bash
zkp-airdrop download-tree \
  --source <URL> \
  --output <FILE> \
  --format <FORMAT>
```

**Arguments**:
- `--source <URL>`: URL to download tree from
- `--output <FILE>`: Output file path
- `--format <FORMAT>`: Tree format (binary, json, compressed) [default: binary]
- `--verify`: Verify tree integrity after download
- `--resume`: Resume partial download
- `--chunk-size <SIZE>`: Download chunk size in MB [default: 100]

**Example**:
```bash
# Download binary format (default)
zkp-airdrop download-tree \
  --source https://api.merkle-tree.io/tree.bin \
  --output ./merkle_tree.bin \
  --verify

# Download compressed format
zkp-airdrop download-tree \
  --source https://api.merkle-tree.io/tree.bin.zst \
  --output ./merkle_tree.bin.zst \
  --format compressed \
  --chunk-size 50
```

**File Formats**:
- **binary**: Raw binary format with header (see technical spec)
- **json**: JSON format for API responses
- **compressed**: Zstandard-compressed binary format

#### `config`

Manage CLI configuration.

```bash
# Show current config
zkp-airdrop config show

# Set default values
zkp-airdrop config set network optimism
zkp-airdrop config set relayer-url https://relayer.zkp-airdrop.io
zkp-airdrop config set merkle-tree-source https://api.merkle-tree.io/tree.bin

# Reset to defaults
zkp-airdrop config reset
```

### Environment Variables

| Variable | Description |
|----------|-------------|
| `ZKP_AIRDROP_PRIVATE_KEY` | Default private key |
| `ZKP_AIRDROP_NETWORK` | Default network |
| `ZKP_AIRDROP_RELAYER_URL` | Default relayer URL |
| `ZKP_AIRDROP_MERKLE_TREE` | Default Merkle tree source |
| `ZKP_AIRDROP_CONFIG` | Path to config file |

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Invalid arguments |
| 3 | Proof generation failed |
| 4 | Proof verification failed |
| 5 | Network error |
| 6 | Already claimed |
| 7 | Rate limited |
| 8 | Insufficient relayer funds |

## Smart Contract ABI

### PrivacyAirdrop Contract

```json
[
  {
    "inputs": [
      {
        "internalType": "address",
        "name": "_token",
        "type": "address"
      },
      {
        "internalType": "bytes32",
        "name": "_merkleRoot",
        "type": "bytes32"
      },
      {
        "internalType": "uint256",
        "name": "_claimAmount",
        "type": "uint256"
      },
      {
        "internalType": "uint256",
        "name": "_claimDeadline",
        "type": "uint256"
      },
      {
        "internalType": "address",
        "name": "_verifier",
        "type": "address"
      }
    ],
    "stateMutability": "nonpayable",
    "type": "constructor"
  },
  {
    "anonymous": false,
    "inputs": [
      {
        "indexed": true,
        "internalType": "bytes32",
        "name": "nullifier",
        "type": "bytes32"
      },
      {
        "indexed": true,
        "internalType": "address",
        "name": "recipient",
        "type": "address"
      }
    ],
    "name": "Claimed",
    "type": "event"
  },
  {
    "inputs": [
      {
        "components": [
          {
            "internalType": "uint256[2]",
            "name": "a",
            "type": "uint256[2]"
          },
          {
            "internalType": "uint256[2][2]",
            "name": "b",
            "type": "uint256[2][2]"
          },
          {
            "internalType": "uint256[2]",
            "name": "c",
            "type": "uint256[2]"
          }
        ],
        "internalType": "struct PrivacyAirdrop.Proof",
        "name": "proof",
        "type": "tuple"
      },
      {
        "internalType": "bytes32",
        "name": "nullifier",
        "type": "bytes32"
      },
      {
        "internalType": "address",
        "name": "recipient",
        "type": "address"
      }
    ],
    "name": "claim",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  }
]
```

## Data Formats

### Hex String Format
All 32-byte values (hashes, nullifiers, Merkle roots) are represented as:
- `0x` prefix
- 64 hexadecimal characters (0-9, a-f, A-F)
- Example: `0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef`

All 20-byte addresses are represented as:
- `0x` prefix  
- 40 hexadecimal characters
- Example: `0x1234567890123456789012345678901234567890`

### Proof JSON Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "properties": {
    "proof": {
      "type": "object",
      "properties": {
        "a": {
          "type": "array",
          "items": { "type": "string" },
          "minItems": 2,
          "maxItems": 2,
          "description": "Groth16 proof A component (field elements as decimal strings)"
        },
        "b": {
          "type": "array",
          "items": {
            "type": "array",
            "items": { "type": "string" },
            "minItems": 2,
            "maxItems": 2
          },
          "minItems": 2,
          "maxItems": 2,
          "description": "Groth16 proof B component (field elements as decimal strings)"
        },
        "c": {
          "type": "array",
          "items": { "type": "string" },
          "minItems": 2,
          "maxItems": 2,
          "description": "Groth16 proof C component (field elements as decimal strings)"
        }
      },
      "required": ["a", "b", "c"]
    },
    "public_signals": {
      "type": "array",
      "items": { 
        "type": "string",
        "pattern": "^(0x[a-fA-F0-9]{1,64}|[0-9]+)$",
        "description": "Field elements as decimal strings (primary) or hex strings (alternative)"
      },
      "minItems": 3,
      "maxItems": 3,
      "description": "Public signals: [merkle_root, recipient, nullifier] as field elements"
    },
    "nullifier": {
      "type": "string",
      "pattern": "^0x[a-fA-F0-9]{64}$",
      "description": "32-byte nullifier hash as hex string"
    },
    "recipient": {
      "type": "string",
      "pattern": "^0x[a-fA-F0-9]{40}$",
      "description": "20-byte Ethereum address as hex string"
    },
    "merkle_root": {
      "type": "string",
      "pattern": "^0x[a-fA-F0-9]{64}$",
      "description": "32-byte Merkle root as hex string"
    },
    "generated_at": {
      "type": "string",
      "format": "date-time",
      "description": "ISO 8601 timestamp when proof was generated"
    }
  },
  "required": ["proof", "public_signals", "nullifier", "recipient", "merkle_root", "generated_at"],
  "description": "Proof JSON format for ZKP Privacy Airdrop claims. Field elements in proof components and public_signals should be decimal strings (primary) or hex strings with 0x prefix (alternative)."
}
```

## Nullifier Specification

The nullifier is a 32-byte value computed as:
```
nullifier = poseidon_hash(private_key)
```
where `private_key` is the 32-byte Ethereum private key (secp256k1 scalar).

**Important Properties**:
1. **Deterministic**: Same private_key always produces same nullifier
2. **Private**: Requires knowledge of private_key to compute (cannot be derived from address alone)
3. **Unique**: Different private keys produce different nullifiers with high probability
4. **Unlinkable**: Cannot determine which address corresponds to a nullifier without the private key
5. **Double-spend protection**: Each nullifier can only be used once on-chain, ensuring each qualified account can only claim once

**Padding Considerations**:
Poseidon with width=3 requires 3 field elements as input (96 bytes). The 32-byte private key needs to be padded:
- **Option 1**: Zero padding: `private_key || [0x00]*64`
- **Option 2**: Domain separation: `poseidon_hash("zkp_airdrop_nullifier" || private_key)`
- **Option 3**: Split into field elements: Encode 32 bytes into 1-2 field elements, pad remainder with zeros

**Example**:
```python
# Pseudocode for nullifier generation
private_key = 0x1234... (32 bytes)

# Option 1: Zero padding (simplest)
padded_input = private_key + b'\x00' * 64  # 96 bytes total
nullifier = poseidon_hash(padded_input)  # 32 bytes output

# Option 2: Domain separation (recommended for collision resistance)
domain = b"zkp_airdrop_nullifier_v1"
padded_input = domain + private_key + b'\x00' * (96 - len(domain) - 32)
nullifier = poseidon_hash(padded_input)
```

**Security Notes**:
- The nullifier must be computed exactly the same way in the ZK circuit and in the CLI
- Use domain separation to prevent cross-protocol nullifier collisions
- The exact padding scheme is an implementation detail but must be consistent across all components
