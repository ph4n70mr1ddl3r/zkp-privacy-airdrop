# ZKP Privacy Airdrop - Unified Specification

## Version: 1.0.0
## Date: 2026-02-06

This document provides a single source of truth for all technical specifications, constants, and interfaces for the ZKP Privacy Airdrop system.

## 1. System Constants

### 1.1 Token Distribution
- **Qualified Accounts**: 65,000,000 (65M) Ethereum addresses
- **Token Name**: ZKP
- **Token Symbol**: ZKP
- **Token Decimals**: 18
- **Claim Amount per Account**: 1,000 ZKP tokens
- **Total Token Supply**: 65,000,000,000 ZKP (65M × 1,000)
- **Claim Period**: 90 days from contract deployment

### 1.2 Merkle Tree
- **Tree Height**: 26 levels (supports up to 2^26 = 67,108,864 leaves)
- **Hash Function**: Poseidon (BN128 scalar field)
- **Leaf Representation**: `Poseidon(address)` where address is 20 bytes
- **Empty Leaf Value**: `Poseidon(0x0000000000000000000000000000000000000000)`
- **Total Nodes**: 134,217,727 (2^27 - 1)
- **Tree Root Size**: 32 bytes (BN128 field element)

### 1.3 Storage Requirements
- **Full Tree Storage**: ~4.3 GB (134,217,727 nodes × 32 bytes)
- **Compressed Tree Storage**: ~8.3 GB (65,000,000 leaves × 128 bytes per path)
- **Proof Data per Claim**: 832 bytes (26 × 32 bytes for Merkle path)
- **Groth16 Proof Size**: ~200 bytes
- **Total Proof Package**: ~1,032 bytes (proof + public signals)

## 2. Cryptographic Specifications

### 2.1 Zero-Knowledge Proof Circuit

#### Circuit: `merkle_membership.circom`
**Public Inputs** (3):
1. `merkle_root`: bytes32 - Merkle root committed on-chain
2. `recipient`: address - Ethereum address receiving tokens
3. `nullifier`: bytes32 - Unique identifier to prevent double claims

**Private Inputs**:
- `private_key`: bytes32 - Ethereum private key (secp256k1 scalar)
- `merkle_path`: bytes32[26] - Sibling nodes along Merkle path
- `merkle_path_indices`: uint256 - 26-bit packed indices (0=left, 1=right)

**Circuit Logic**:
1. Validate private key is valid secp256k1 scalar: `private_key < SECP256K1_ORDER`
2. Derive public key: `(pub_x, pub_y) = secp256k1_mul(G, private_key)`
3. Derive Ethereum address: `address = keccak256(pub_bytes[1:])[12:32]`
4. Compute leaf: `leaf = Poseidon(address)`
5. Verify Merkle proof using Poseidon hash
6. Compute nullifier: `nullifier = Poseidon(private_key || recipient)`
7. Verify computed nullifier matches public input

#### Poseidon Hash Parameters
- **Prime Field**: BN128 (0x30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd47)
- **Width**: 3 (capacity 2, rate 1)
- **Rounds**: 8 full rounds, 57 partial rounds
- **Alpha**: 5
- **MDS Matrix**: Precomputed for BN128 field
- **Security Level**: 128-bit

### 2.2 Nullifier Generation
```
nullifier = Poseidon(
    private_key (32 bytes) || 
    recipient (20 bytes) ||
    padding (12 bytes of zeros)
)
```
- **Total Input**: 64 bytes (padded to fit 4 field elements)
- **Output**: 32 bytes (1 field element)
- **Properties**: Deterministic, collision-resistant, unlinkable

### 2.3 Field Element Encoding
All field elements are encoded as:
- **BN128 Field**: Integers modulo `p = 21888242871839275222246405745257275088548364400416034343698204186575808495617`
- **Serialization**: Big-endian 32-byte representation
- **Validation**: Must be `0 <= x < p`

## 3. Smart Contract Interfaces

### 3.1 PrivacyAirdrop Contract
```solidity
interface IPrivacyAirdrop {
    struct Proof {
        uint256[2] a;
        uint256[2][2] b;
        uint256[2] c;
    }
    
    function claim(
        Proof calldata proof,
        bytes32 nullifierHash,
        address recipient
    ) external;
    
    function isClaimed(bytes32 nullifierHash) external view returns (bool);
    function merkleRoot() external view returns (bytes32);
    function claimAmount() external view returns (uint256);
    function claimDeadline() external view returns (uint256);
}
```

### 3.2 RelayerRegistry Contract
```solidity
interface IRelayerRegistry {
    function donate() external payable;
    function withdraw(uint256 amount) external;
    function isAuthorized(address relayer) external view returns (bool);
    function balanceOf(address relayer) external view returns (uint256);
}
```

## 4. Data Formats

### 4.1 Proof JSON Format
```json
{
  "proof": {
    "a": ["<field_element>", "<field_element>"],
    "b": [["<field_element>", "<field_element>"], ["<field_element>", "<field_element>"]],
    "c": ["<field_element>", "<field_element>"]
  },
  "public_signals": ["<merkle_root>", "<recipient>", "<nullifier>"],
  "nullifier_hash": "0x...",
  "recipient": "0x...",
  "merkle_root": "0x...",
  "generated_at": "ISO8601_TIMESTAMP"
}
```

### 4.2 Merkle Tree Binary Format
```
[Header (16 bytes)]
  magic: "ZKPT" (0x5A4B5054)
  version: 1
  height: 26
  reserved: 0x0000
  num_leaves: 65000000 (0x03E0D240)
  root_hash: [32 bytes]

[Leaf Data Section]
  For i in 0..num_leaves-1:
    address: [20 bytes]  // Ethereum address
    leaf_hash: [32 bytes] // Poseidon(address)
    path_data: [128 bytes] // 26 × 32-bit indices + 26 × 32-byte siblings
```

### 4.3 API Request/Response Formats
See API Reference (docs/04-api-reference.md) for detailed schemas.

## 5. System Architecture

### 5.1 Component Specifications

#### Rust CLI
- **Version**: Rust 1.70+
- **Dependencies**: arkworks, circom-compat, secp256k1, ethers-rs
- **Proof Generation Time**: < 5 seconds (target)
- **Memory Usage**: < 1 GB (with tree cache)

#### Relayer Service
- **Framework**: Actix-web or Axum (Rust)
- **Database**: PostgreSQL (metrics only)
- **Cache**: Redis (rate limiting)
- **Concurrency**: 100+ concurrent requests
- **Uptime Target**: 99.9%

#### Smart Contracts
- **Solidity Version**: 0.8.19+
- **Verification Gas**: ~300,000 gas (target)
- **Claim Gas**: ~500,000 gas (target)

### 5.2 Rate Limiting
- **Per Nullifier**: 1 request per 60 seconds
- **Per IP Address**: 100 requests per minute
- **Global**: 1,000 requests per minute
- **Burst Allowance**: 2x limit for 10 seconds

### 5.3 Security Parameters
- **ZK Security Level**: 128 bits
- **Hash Security**: 128 bits (Poseidon)
- **Trusted Setup Participants**: Minimum 10 independent parties
- **Audit Requirements**: 3+ independent security firms
- **Bug Bounty**: Yes, with substantial rewards

## 6. Deployment Specifications

### 6.1 Network Configuration
- **Mainnet**: Ethereum
- **Testnet**: Sepolia
- **RPC Requirements**: Archive node access
- **Gas Price Strategy**: EIP-1559 with 10% premium

### 6.2 Infrastructure Requirements
- **Relayer Servers**: 3+ instances (medium/large)
- **Database**: PostgreSQL RDS with read replicas
- **Cache**: Redis Cluster
- **CDN**: Cloudflare for static assets
- **Monitoring**: Prometheus + Grafana + AlertManager

### 6.3 Key Management
- **Relayer Private Keys**: AWS KMS or HashiCorp Vault
- **Trusted Setup Keys**: Multi-party computation ceremony
- **Deployment Keys**: Multi-sig with 3/5 signers

## 7. Testing Specifications

### 7.1 Test Coverage Targets
- **Unit Tests**: 90%+ coverage
- **Integration Tests**: Full claim workflow
- **Load Tests**: 10,000 concurrent claims
- **Security Tests**: Formal verification + audits

### 7.2 Performance Targets
- **Proof Generation**: < 5 seconds
- **Claim Processing**: < 30 seconds (end-to-end)
- **API Response**: < 100ms (p95)
- **Database Queries**: < 10ms (p95)

## 8. Change Management

### 8.1 Versioning
- **API Version**: v1 (initial release)
- **Circuit Version**: v1 (immutable after trusted setup)
- **Contract Version**: v1 (immutable after deployment)
- **CLI Version**: Semantic versioning

### 8.2 Upgrade Paths
- **Contracts**: Proxy pattern for future upgrades
- **Circuits**: New trusted setup required for changes
- **Relayer**: Rolling updates with zero downtime
- **CLI**: Backward compatible for 6 months

## 9. Compliance & Standards

### 9.1 Cryptographic Standards
- **ZK Proofs**: Groth16 (BN128)
- **Hash Functions**: Poseidon (ZK-friendly), Keccak-256 (address derivation)
- **Elliptic Curves**: secp256k1 (Ethereum), BN128 (pairing)
- **Randomness**: Cryptographically secure RNG

### 9.2 Code Standards
- **Solidity**: Solidity Style Guide
- **Rust**: Rustfmt + Clippy
- **Testing**: Property-based testing for cryptographic code
- **Documentation**: All public APIs documented

## 10. Constants Reference

### 10.1 Numerical Constants
```solidity
uint256 constant CLAIM_AMOUNT = 1000 * 10**18; // 1000 ZKP tokens
uint256 constant CLAIM_DEADLINE = 90 days;
uint256 constant MERKLE_TREE_HEIGHT = 26;
uint256 constant MAX_LEAVES = 2**26; // 67,108,864
```

### 10.2 Hash Constants
```solidity
// Poseidon parameters for BN128
uint256 constant POSEIDON_WIDTH = 3;
uint256 constant POSEIDON_FULL_ROUNDS = 8;
uint256 constant POSEIDON_PARTIAL_ROUNDS = 57;
uint256 constant POSEIDON_ALPHA = 5;

// secp256k1 constants
uint256 constant SECP256K1_ORDER = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141;
```

### 10.3 Gas Estimates
```solidity
uint256 constant VERIFY_PROOF_GAS = 300_000;
uint256 constant CLAIM_GAS = 500_000;
uint256 constant RELAYER_BUFFER = 1_000_000; // Additional buffer
```

---

*This specification is the authoritative source for all implementations. Any discrepancies between this document and other documentation should be resolved in favor of this specification.*