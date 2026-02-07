# ZKP Privacy Airdrop - Unified Specification

## Version: 1.5.0
## Date: 2026-02-07

This document provides a single source of truth for all technical specifications, constants, and interfaces for the ZKP Privacy Airdrop system.

## 1. System Constants

### 1.1 Token Distribution
- **Qualified Accounts**: 65,249,064 Ethereum addresses (from accounts.csv)
  - **Selection Criteria**: Ethereum mainnet addresses that paid at least 0.004 ETH in gas fees from genesis until December 31, 2025
  - **Source**: https://drive.google.com/file/d/1yvgsuDMhamUoKAfH59iuyDtm7x5mnHRX/view?usp=sharing
  - **Download Command**: `gdown 1yvgsuDMhamUoKAfH59iuyDtm7x5mnHRX`
- **Deployment Network**: Optimism (Layer 2 on Ethereum)
  - **Benefits**: 10-100x lower gas fees compared to Ethereum mainnet
  - **Address Compatibility**: Same Ethereum addresses work on Optimism
  - **Bridge Required**: Users need ETH on Optimism for direct claims
- **Token Name**: ZKP
- **Token Symbol**: ZKP
- **Token Decimals**: 18
- **Claim Amount per Account**: 1,000 ZKP tokens
- **Total Token Supply**: 65,249,064,000 ZKP (65,249,064 × 1,000)
- **Claim Period**: 90 days from contract deployment

### 1.2 Merkle Tree
- **Tree Height**: 26 levels (supports up to 2^26 = 67,108,864 leaves)
- **Actual Leaves**: 65,249,064 (from accounts.csv)
- **Extra Capacity**: 1,859,800 empty leaves available
- **Hash Function**: Poseidon (BN128 scalar field)
- **Leaf Representation**: `Poseidon(address)` where address is 20 bytes, padded to 32 bytes with 12 leading zeros
- **Empty Leaf Value**: `Poseidon(0x0000000000000000000000000000000000000000000000000000000000000000)` (32 zero bytes)
- **Total Nodes**: 134,217,727 (2^27 - 1)
- **Tree Root Size**: 32 bytes (BN128 field element)

### 1.3 Storage Requirements
- **Full Tree Storage**: 4.00 GiB (134,217,727 nodes × 32 bytes = 4,294,967,264 bytes)
- **Proof Data per Claim**: 832 bytes (26 × 32 bytes for Merkle path siblings)
- **Groth16 Proof Size**: ~200 bytes
- **Total Proof Package**: ~1,032 bytes (Groth16 proof: ~200 bytes + Merkle path: 832 bytes)
- **Precomputed Proofs Storage**: 56.88 GiB (65,249,064 leaves × 936 bytes per leaf including Merkle path siblings (832 bytes), leaf hash (32 bytes), and path indices (104 bytes))
- **Merkle Tree File Size**:
  - Binary format with addresses only: 1.216 GiB (16 byte header + 65,249,064 × 20 bytes = 1,304,981,280 bytes)
  - Binary format with hashes only: 1.945 GiB (16 byte header + 65,249,064 × 32 bytes = 2,087,970,064 bytes)
  - Full tree for local generation: 4.00 GiB (all 134,217,727 nodes × 32 bytes = 4,294,967,264 bytes)

### 1.4 Merkle Tree Generation Process

**Input Data**:
- List of 65,249,064 Ethereum addresses (20 bytes each) from accounts.csv
  - **Source**: https://drive.google.com/file/d/1yvgsuDMhamUoKAfH59iuyDtm7x5mnHRX/view?usp=sharing
  - **Download Command**: `gdown 1yvgsuDMhamUoKAfH59iuyDtm7x5mnHRX`
- Total input size: 65,249,064 × 20 bytes = 1.216 GiB

**Generation Steps**:
1. **Hashing**: Compute Poseidon hash for each address → 65,249,064 leaves (32 bytes each)
2. **Tree Construction**: Build binary Merkle tree bottom-up
3. **Root Computation**: Compute final Merkle root (32 bytes)
4. **Proof Generation**: For each leaf, compute its Merkle path (832 bytes)
5. **Verification**: Independent verification of tree construction

**Computational Requirements**:
- **Memory**: ~32 GiB RAM recommended
- **Storage**: ~10 GiB temporary storage
- **CPU**: Multi-threaded processing required
- **Time**: Estimated 4-8 hours on modern hardware

**Verification & Audit**:
- **Input Data**: Available at https://drive.google.com/file/d/1yvgsuDMhamUoKAfH59iuyDtm7x5mnHRX/view?usp=sharing (download with `gdown 1yvgsuDMhamUoKAfH59iuyDtm7x5mnHRX`)
- **Checksum**: SHA256 of sorted address list (will be published with final tree generation: `sha256sum accounts.csv`)
- **Independent Verification**: Multiple parties recompute tree from same input
- **Proof of Correctness**: Provide sample proofs for random leaves
- **Transparency**: Publish generation script and input checksum

**Distribution**:
- **Primary**: API service providing Merkle paths on-demand (no full tree download needed)
- **Secondary**: CDN with HTTP range requests for full tree (4.00 GiB)
- **Fallback**: IPFS with content-addressed storage (CID: Qm...)
- **Alternative**: Torrent for peer-to-peer distribution
- **Verification Tool**: CLI tool to verify tree integrity and generate proofs offline

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
6. Compute nullifier: `nullifier = Poseidon(private_key (32 bytes) || recipient (20 bytes) || padding (12 bytes of zeros))`
7. Verify computed nullifier matches public input

#### Poseidon Hash Parameters
- **Prime Field**: BN128 scalar field (modulus: 0x30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd47)
- **Width**: 3 (capacity 2, rate 1)
- **Full Rounds**: 8 (4 at beginning, 4 at end)
- **Partial Rounds**: 57
- **Alpha**: 5 (x^5 S-box)
- **MDS Matrix**: Precomputed for BN128 field using the Cauchy matrix method
- **Round Constants**: Generated using SHA256 with domain separation
- **Security Level**: 128-bit against preimage and collision attacks
- **Implementation**: Compatible with circomlib's Poseidon implementation

**Constants Generation**:
- Round constants derived from: `SHA256("poseidon_bn128_t3_f8_p57_" || domain_separator || round_index)`
- MDS matrix generated using Cauchy matrix: `M[i][j] = 1/(x_i + y_j)` where x_i, y_j are distinct field elements
- Domain separation: `"zkp_airdrop_v1"` for all Poseidon instances in the circuit

**Reference Implementation**:
```circom
include "circomlib/poseidon.circom";

component poseidon = Poseidon(3);
poseidon.inputs[0] <== input0;
poseidon.inputs[1] <== input1;
poseidon.inputs[2] <== input2;
output <== poseidon.out;
```

### 2.2 Nullifier Generation
```
nullifier = Poseidon(
    private_key (32 bytes) || 
    recipient (20 bytes) ||
    padding (12 bytes of zeros)
)
```
- **Total Input**: 64 bytes (padded to fit 4 field elements for Poseidon with width 3)
- **Output**: 32 bytes (1 field element)
- **Properties**: Deterministic, collision-resistant, unlinkable
- **Note**: Padding ensures consistent 64-byte input for all nullifier computations

### 2.3 Field Element Encoding
All field elements are encoded as:
- **BN128 Field**: Integers modulo `p = 21888242871839275222246405745257275088548364400416034343698204186575808495617`
- **Serialization**: Big-endian 32-byte representation
- **Validation**: Must be `0 <= x < p`
- **API Format**: Decimal strings (primary)
- **Contract Format**: uint256 (big-endian bytes)
- **CLI Format**: Accepts both decimal and hex (with 0x prefix)
- **Storage**: Hex strings for readability, decimal strings for API transmission

## 3. Smart Contract Interfaces

### 3.1 PrivacyAirdrop Contract

**Key Properties**:
- **Immutable**: Contract cannot be modified after deployment
- **Permissionless**: Anyone with a valid proof can call `claim()`
- **Trustless**: No admin keys or upgradeability
- **Decentralized**: No single point of failure

**Claiming Options**:
1. **Direct Submission**: User pays their own gas and submits proof directly
2. **Relayer Submission**: User submits proof to relayer service, which pays gas (funded by donations)

```solidity
interface IPrivacyAirdrop {
    struct Proof {
        uint256[2] a;
        uint256[2][2] b;
        uint256[2] c;
    }
    
    // Anyone can call claim() - no whitelist required
    // Relayers are optional services that pay gas for users
    function claim(
        Proof calldata proof,
        bytes32 nullifier,
        address recipient
    ) external;
    
    function isClaimed(bytes32 nullifier) external view returns (bool);
    function merkleRoot() external view returns (bytes32);
    function claimAmount() external view returns (uint256);
    function claimDeadline() external view returns (uint256);
    
    // Optional: estimate gas for claim transaction
    function estimateClaimGas(
        Proof calldata proof,
        bytes32 nullifier,
        address recipient
    ) external view returns (uint256);
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
  "nullifier": "0x...",
  "recipient": "0x...",
  "merkle_root": "0x...",
  "generated_at": "ISO8601_TIMESTAMP"
}
```

**Field Element Encoding**:
- **Primary Format**: Decimal strings (not hex)
- **Alternative Format**: Hex strings with `0x` prefix (for developer convenience)
- **Validation**: Must be valid BN128 field elements: `0 <= x < p` where `p = 21888242871839275222246405745257275088548364400416034343698204186575808495617`
- **Examples**: 
  - Decimal: `"13862987149607610235678184535533251295074929736392939725598345555223684473689"`
  - Hex: `"0x1eab1f9d8c9a0e3a9a1b9c8d7a6b5c4d3e2f1a0b9c8d7e6f5a4b3c2d1e0f9a8b7c6d"`
- **Validation Code Example** (Python):
  ```python
  p = 21888242871839275222246405745257275088548364400416034343698204186575808495617
  def validate_field_element(x_str):
      try:
          x = int(x_str) if not x_str.startswith('0x') else int(x_str[2:], 16)
          return 0 <= x < p
      except ValueError:
          return False
  ```

**Address Encoding**:
- **Format**: 20-byte hex strings with `0x` prefix (lowercase)
- **Validation**: Must be valid Ethereum address checksum (EIP-55 optional but recommended)
- **Example**: `"0x1234567890123456789012345678901234567890"`
- **Validation Code Example** (Python):
  ```python
  import re
  
  def validate_address(addr_str):
      # Basic format check
      if not re.match(r'^0x[a-fA-F0-9]{40}$', addr_str):
          return False
      # Optional EIP-55 checksum validation
      return True  # Implement EIP-55 check if needed
  ```

**Hash Encoding**:
- **Format**: 32-byte hashes (Merkle root, nullifier) as 64-character hex strings with `0x` prefix (lowercase)
- **Example**: `"0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"`
- **Validation Code Example** (Python):
  ```python
  import re
  
  def validate_hash(hash_str):
      return bool(re.match(r'^0x[a-fA-F0-9]{64}$', hash_str))
  ```

### 4.2 Merkle Tree Binary Format
```
[Header (16 bytes)]
  magic: "ZKPT" (0x5A4B5054)
  version: 1
  height: 26
  reserved: 0x0000
  num_leaves: 65249064 (0x03E3B4A8)
  root_hash: [32 bytes]

[Leaf Data Section]
  For i in 0..num_leaves-1:
    address: [20 bytes]  // Ethereum address
    leaf_hash: [32 bytes] // Poseidon(address)
    path_data: [936 bytes] // 26 × 32-bit indices (104 bytes) + 26 × 32-byte siblings (832 bytes) = 936 bytes total
```

### 4.3 API Request/Response Formats
See API Reference (docs/04-api-reference.md) for detailed schemas.

### 4.4 Sample Proof Data (Test Vectors)

**Note**: These are example values for testing and documentation. Real values will differ.

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

**Field Element Validation**:
- All proof components (`a`, `b`, `c`) are arrays of 2 field elements each
- `public_signals` is an array of 3 field elements: `[merkle_root, recipient, nullifier]`
- Field elements must be `< p` where `p = 21888242871839275222246405745257275088548364400416034343698204186575808495617`

**Nullifier Calculation** (pseudocode):
```
private_key = 0x1234... (32 bytes)
recipient = 0x5678... (20 bytes)
padding = 0x000000000000000000000000 (12 bytes)
input = private_key || recipient || padding  // 64 bytes total
nullifier = poseidon_hash(input)  // 32 bytes
```

**Byte-Level Example**:
```
private_key = 0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef (32 bytes)
recipient = 0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266 (20 bytes)
padding = 0x000000000000000000000000 (12 bytes)
input = 0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdeff39fd6e51aad88f6f4ce6ab8827279cfffb92266000000000000000000000000 (64 bytes)
nullifier = poseidon_hash(input)  # Result: 32-byte hash
```

## 5. System Architecture

### 5.1 Component Specifications

#### Smart Contracts
- **Solidity Version**: 0.8.19+
- **Proof Verification Gas**: ~300,000 gas (Groth16 verification)
- **Total Claim Transaction Gas**: ~500,000 gas (verification + storage + transfer)
- **Relayer Buffer**: 200,000 gas additional buffer for gas price fluctuations
- **Estimated Gas per Claim**: 700,000 gas (verification + storage + transfer + buffer)
- **Maximum Gas per Claim**: 1,000,000 gas (absolute maximum with 100% buffer)
- **Optimism Cost Advantage**: Gas prices on Optimism are 10-100x cheaper than Ethereum L1, making claims affordable even for users submitting directly

### 5.2 Rate Limiting
- **Per Nullifier**: 1 request per 60 seconds (all endpoints)
- **Per IP Address**: 100 requests per 60 seconds (all endpoints)
- **Global**: 1,000 requests per 60 seconds (all endpoints)
- **Endpoint-Specific Limits**:
  - `POST /api/v1/submit-claim`: 1 request per 60 seconds per nullifier
  - `GET /api/v1/check-status/{nullifier}`: 10 requests per 60 seconds per nullifier
  - `GET /api/v1/merkle-path/{address}`: 60 requests per 60 seconds per IP
  - Other endpoints: 100 requests per 60 seconds per IP
- **Burst Allowance**: 2x limit for 10 seconds

### 5.3 Security Parameters
- **ZK Security Level**: 128 bits
- **Hash Security**: 128 bits (Poseidon)
- **Trusted Setup Participants**: Minimum 10 independent parties
- **Audit Requirements**: 3+ independent security firms
- **Bug Bounty**: Yes, with substantial rewards

### 5.4 Comprehensive Security Plan

#### 5.4.1 Trusted Setup Ceremony
**Phase 1 (Powers of Tau)**:
- Use existing community-trusted setup (e.g., Perpetual Powers of Tau)
- Verify contribution integrity using zk-SNARKs
- Multi-party computation with at least 10 independent participants
- Publicly verifiable transcripts published on IPFS

**Phase 2 (Circuit-Specific)**:
- Dedicated ceremony for merkle_membership circuit
- Participants from diverse backgrounds (academia, industry, community)
- Secure computation environment with air-gapped machines
- Live streaming of ceremony with public verification

#### 5.4.2 Key Management
**Contract Deployment**:
- 3/5 multisig for deployment and upgrades
- Time-locked administrative functions
- Emergency pause mechanism
- Gradual decentralization roadmap

**Relayer Operations**:
- AWS KMS or HashiCorp Vault for private keys
- Automated key rotation every 24 hours
- Hot/warm/cold wallet separation
- Multi-sig for large withdrawals

#### 5.4.3 Disaster Recovery
**Data Backup**:
- Daily backups of Merkle tree and claim state
- Geographic redundancy across 3+ regions
- Versioned backups with 30-day retention
- Regular restoration testing

**Incident Response**:
- 24/7 monitoring and alerting
- Escalation procedures for security incidents
- Communication plan for users during outages
- Post-incident analysis and remediation

#### 5.4.4 Monitoring & Alerting
**Key Metrics**:
- Proof verification success/failure rates
- Claim volume and distribution patterns
- Relayer balance and gas usage
- API response times and error rates

**Alert Thresholds**:
- >5% proof verification failure rate
- Relayer balance < 1 ETH (warning)
- Relayer balance < 0.5 ETH (critical - stop accepting claims)
- API error rate > 1%
- Unusual claim patterns detected

## 6. Deployment Specifications

### 6.1 Network Configuration
- **Mainnet**: Optimism (Layer 2 on Ethereum)
- **Testnet**: Optimism Sepolia
- **RPC Requirements**: Optimism RPC endpoint (public or private)
- **Gas Price Strategy**: 
  - Optimism gas is 10-100x cheaper than Ethereum L1
  - Base: EIP-1559 with 10% premium for reliability
  - Privacy Enhancement: Add random 0-5% variance to break timing correlations
  - Maximum: 0.1 gwei cap to prevent excessive fees (Optimism gas is much cheaper)
  - Relayers should use: `gas_price = min(base_fee * 1.1 * (1 + random(0, 0.05)), 0.1 gwei)`

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
- **Unit Tests**: 90%+ coverage for all components
- **Integration Tests**: Full claim workflow with mocked dependencies
- **Load Tests**: 10,000 concurrent claims with realistic distribution
- **Security Tests**: Formal verification + third-party audits

### 7.2 Performance Targets
- **Proof Generation**: < 5 seconds (95th percentile) on modern hardware (8-core CPU, 16GB RAM)
- **Claim Processing**: < 30 seconds end-to-end (submission to confirmation) including blockchain confirmation
- **API Response**: < 100ms (p95) for all endpoints on typical cloud infrastructure (4 vCPU, 8GB RAM)
- **Database Queries**: < 10ms (p95) for read queries, < 50ms for writes on PostgreSQL with SSDs

### 7.3 Detailed Testing Requirements

#### 7.3.1 Circuit Testing
- **Property-based testing**: Verify circuit constraints for random inputs
- **Edge cases**: Zero values, maximum values, invalid inputs
- **Formal verification**: Prove circuit correctness using automated tools
- **Witness generation**: Test with 1,000+ random valid/invalid witnesses
- **Cross-implementation verification**: Compare outputs with reference implementation

#### 7.3.2 Smart Contract Testing
- **Unit tests**: 100% coverage for critical functions (claim, verify, admin)
- **Integration tests**: Full claim flow with mocked relayer
- **Fuzzing**: Property-based fuzzing with Echidna/Hardhat
- **Gas optimization**: Verify gas usage meets targets
- **Upgrade testing**: Test proxy pattern upgrades

#### 7.3.3 Relayer Testing
- **Load testing**: 10,000+ concurrent requests
- **Stress testing**: System limits and failure modes
- **Security testing**: Penetration testing, DDoS simulation
- **Recovery testing**: Database restore, failover procedures
- **Monitoring testing**: Alert verification, metric collection

#### 7.3.4 End-to-End Testing
- **Full workflow**: Generate proof → submit → verify on-chain
- **Multiple networks**: Test on Optimism Sepolia and Optimism mainnet fork
- **Error handling**: Network failures, insufficient funds, rate limiting
- **User experience**: CLI tool usability, error messages, help text
- **Compatibility**: Test with different Ethereum clients and node versions

#### 7.3.5 Test Vectors
- **Valid proofs**: 100+ test vectors with known inputs/outputs
- **Edge cases**: Zero/null values, maximum field elements, boundary conditions
- **Invalid proofs**: Malformed inputs, wrong signatures, incorrect Merkle paths
- **Integration test vectors**: Complete claim transactions with known outcomes
- **Performance test vectors**: Large batch processing for load testing

#### 7.3.6 Security Testing
- **Third-party audits**: Minimum 2 independent security firms
- **Bug bounty program**: Public program with substantial rewards
- **Code review**: Internal and external review of all critical code
- **Dependency audit**: Regular security updates for all dependencies
- **Incident response testing**: Simulate security incidents and responses

## 8. Change Management

### 8.1 Versioning
- **API Version**: v1 (initial release)
- **Circuit Version**: v1 (immutable after trusted setup)
- **Contract Version**: v1 (immutable after deployment)
- **CLI Version**: Semantic versioning

### 8.2 Upgrade Paths
- **Contracts**: Proxy pattern with transparent proxy for future upgrades
- **Circuits**: New trusted setup required for circuit changes (breaking changes only)
- **Relayer**: Rolling updates with zero downtime, backward-compatible API
- **CLI**: Backward compatible for 6 months, deprecation warnings for older versions

### 8.3 Circuit Versioning Protocol

**Circuit Identifier**: `merkle_membership_v1`
- **Format**: `<circuit_name>_v<major>.<minor>.<patch>`
- **Immutable Properties**: Hash function, curve, security level cannot change within same major version
- **Upgradable Properties**: Circuit optimizations, constraint reductions (patch version changes)

**Breaking Changes (Major Version Bump)**:
- Changes to hash function (Poseidon → different hash)
- Changes to elliptic curve (BN128 → BLS12-381)
- Changes to security level (128-bit → 256-bit)
- Changes to public/private input structure

**Non-Breaking Changes (Minor/Patch Version)**:
- Circuit optimizations (reduced constraints)
- Bug fixes in circuit logic
- Improved efficiency without changing proofs
- Additional safety checks

**Migration Process**:
1. New circuit version deployed with new trusted setup
2. Old contract remains active during migration period
3. New contract deployed with updated verifier
4. Users can claim using either circuit version during transition
5. Old contract disabled after migration period (e.g., 30 days)

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

### 9.3 Legal & Compliance

#### 9.3.1 Regulatory Considerations
- **Jurisdiction**: Project based in Switzerland (crypto-friendly jurisdiction)
- **Token Classification**: Utility token, not a security
- **KYC/AML**: No built-in KYC/AML requirements
- **OFAC Compliance**: No built-in sanctions screening

#### 9.3.2 User Responsibilities
- **Legal Compliance**: Users responsible for complying with local laws
- **Tax Obligations**: Users responsible for tax reporting
- **Age Requirements**: Must be 18+ or age of majority in jurisdiction
- **Prohibited Jurisdictions**: Users from OFAC-sanctioned countries prohibited

#### 9.3.3 Privacy Considerations
- **GDPR Compliance**: No personal data collection by design
- **Data Retention**: Minimal logs, no IP address storage
- **Right to be Forgotten**: Not applicable due to blockchain immutability
- **Data Protection**: End-to-end encryption for all communications

#### 9.3.4 Terms of Service
- **Disclaimer**: Software provided "as is" without warranties
- **Liability Limitation**: No liability for lost funds or technical issues
- **Governing Law**: Swiss law, with arbitration in Zurich
- **Updates**: Terms may be updated with 30-day notice

#### 9.3.5 Risk Disclosures
Users must acknowledge:
1. **Blockchain Risk**: Transactions are irreversible
2. **Privacy Limitations**: See Privacy Analysis document
3. **Technical Risk**: Software may contain bugs
4. **Regulatory Risk**: Laws may change affecting usability
5. **Market Risk**: Token value may fluctuate

## 10. Constants Reference

### 10.1 Numerical Constants
```solidity
uint256 constant CLAIM_AMOUNT = 1000 * 10**18; // 1000 ZKP tokens
uint256 constant CLAIM_DEADLINE = 90 days;
uint256 constant MERKLE_TREE_HEIGHT = 26;
uint256 constant MAX_LEAVES = 2**26; // 67,108,864 (maximum capacity)
uint256 constant TOTAL_QUALIFIED_ACCOUNTS = 65_249_064; // From accounts.csv
uint256 constant TOTAL_TOKEN_SUPPLY = 65_249_064_000 * 10**18; // 65,249,064 × 1000 ZKP tokens
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

**Gas units remain the same on Optimism, but costs are 10-100x cheaper due to lower gas prices.**

```solidity
uint256 constant VERIFY_PROOF_GAS = 300_000;     // Gas for Groth16 proof verification
uint256 constant STORAGE_TRANSFER_GAS = 200_000; // Gas for storage updates and token transfer
uint256 constant CLAIM_GAS = 500_000;            // Total gas for claim transaction (verification + storage + transfer = 300K + 200K)
uint256 constant RELAYER_BUFFER = 200_000;       // Additional buffer for relayers to handle gas price fluctuations
uint256 constant ESTIMATED_CLAIM_GAS = 700_000;  // Estimated gas with buffer (500K + 200K)
uint256 constant MAX_CLAIM_GAS = 1_000_000;      // Maximum gas allowance per claim (absolute maximum with 100% buffer)
```

**Cost Comparison (Approximate)**:
- **Ethereum L1**: ~700,000 gas × 50 gwei = 0.035 ETH ($~105 at $3,000 ETH)
- **Optimism**: ~700,000 gas × 0.001 gwei = 0.0007 ETH ($~2.10 at $3,000 ETH)
- **Savings**: ~50x cheaper on Optimism

## 11. Glossary

### 11.1 Core Concepts
- **Nullifier**: Unique identifier computed as `Poseidon(private_key (32 bytes) || recipient (20 bytes) || padding (12 bytes of zeros))`. Prevents double claims.
- **Merkle Path**: The sibling nodes along the path from a leaf to the Merkle root (26 × 32 bytes = 832 bytes).
- **Field Element**: Integer in the BN128 scalar field modulo `p = 21888242871839275222246405745257275088548364400416034343698204186575808495617`.
- **Public Signals**: Array `[merkle_root, recipient, nullifier]` passed to the verifier, where each is a field element.

### 11.2 Data Formats
- **Decimal String Format**: Primary format for field elements (e.g., `"13862987149607610235678184535533251295074929736392939725598345555223684473689"`).
- **Hex String Format**: Alternative format for field elements with `0x` prefix (e.g., `"0x1eab1f9d8c9a0e3a9a1b9c8d7a6b5c4d3e2f1a0b9c8d7e6f5a4b3c2d1e0f9a8b7c6d"`).
- **Address Format**: 20-byte Ethereum address as hex string with `0x` prefix (e.g., `"0x1234567890123456789012345678901234567890"`).
- **Hash Format**: 32-byte hashes as 64-character hex strings with `0x` prefix.

### 11.3 System Components
- **Relayer**: Optional service that submits proofs on behalf of users and pays gas fees. Users can also submit directly to the contract.
- **Proof Package**: Complete proof data including Groth16 proof (`a`, `b`, `c` arrays) and public signals.
- **Merkle Tree**: Binary tree of height 26 containing 65,249,064 leaves (hashes of qualified addresses).

### 11.4 Cryptographic Parameters
- **Poseidon Hash**: Width=3, full_rounds=8, partial_rounds=57, alpha=5 over BN128 scalar field.
- **BN128 Curve**: Elliptic curve used for Groth16 proof system (Ethereum precompile compatible).
- **secp256k1**: Curve used for Ethereum address derivation from private keys.

### 11.5 Gas Estimates
- **VERIFY_PROOF_GAS**: 300,000 gas for Groth16 proof verification
- **STORAGE_TRANSFER_GAS**: 200,000 gas for storage updates and token transfer
- **CLAIM_GAS**: 500,000 gas total for verification + storage + transfer
- **RELAYER_BUFFER**: 200,000 gas additional buffer for gas price fluctuations
- **ESTIMATED_CLAIM_GAS**: 700,000 gas estimated with buffer (500K + 200K)
- **MAX_CLAIM_GAS**: 1,000,000 gas maximum allowance (absolute maximum with 100% buffer)

---

*This specification is the authoritative source for all implementations. Any discrepancies between this document and other documentation should be resolved in favor of this specification.*