# Technical Specification

## 1. Zero-Knowledge Proof System

### 1.1 Circuit Design

**Circuit**: `merkle_membership.circom`

The circuit proves:
1. Knowledge of private key `sk` corresponding to public key/address `pk`
2. `pk` is a leaf in the Merkle tree with root `root`
3. Knowledge of Merkle path from `pk` to `root`

**Public Inputs**:
- `merkle_root`: The Merkle root committed on-chain (32 bytes, BN128 field element)
- `recipient`: The address receiving tokens (20 bytes, packed as field element)
- `nullifier`: Unique identifier to prevent double-claims (32 bytes, field element)

**Private Inputs**:
- `private_key`: The claimant's Ethereum private key (32 bytes, 2 field elements)
- `merkle_path`: Sibling nodes along the Merkle path (26 × 32 bytes, 26 field elements)
- `merkle_path_indices`: Left/right indicators for path (26 bits, packed as field element)

**Circuit Constraints**:
```
// 1. Private key validation (must be valid secp256k1 scalar)
assert(private_key < secp256k1_order)

// 2. Derive public key from private key
public_key_x, public_key_y = secp256k1_scalar_mul(base_point_x, base_point_y, private_key)

// 3. Derive Ethereum address from public key
public_key_bytes = encode_uncompressed(public_key_x, public_key_y)  // 65 bytes
address_hash = keccak256(public_key_bytes[1:])                     // Remove 0x04 prefix
address = address_hash[12:32]                                      // Last 20 bytes

// 4. Compute leaf hash for Merkle tree
leaf = poseidon_hash(address)

// 5. Verify Merkle proof
current_hash = leaf
for i in 0..25:
    if merkle_path_indices[i] == 0:
        current_hash = poseidon_hash(current_hash, merkle_path[i])
    else:
        current_hash = poseidon_hash(merkle_path[i], current_hash)
assert(current_hash == merkle_root)

// 6. Compute nullifier (private_key || recipient padded to 64 bytes)
nullifier_input = private_key || recipient || [0x00; 12]  // 64 bytes total
computed_nullifier = poseidon_hash(nullifier_input)
assert(computed_nullifier == nullifier)
```

**Circuit Statistics**:
- **Constraints**: ~500,000
- **Proof Size**: ~200 bytes (Groth16 proof only)
- **Verification Time**: ~3ms on-chain
- **Trusted Setup**: Phase 2 ceremony required
- **Hash Functions**: Poseidon (for Merkle tree and nullifier), Keccak256 (for address derivation)
- **Curve**: BN128 (Ethereum compatible)

**Poseidon Hash Parameters**:
- **Prime Field**: BN128 scalar field (modulus: 21888242871839275222246405745257275088548364400416034343698204186575808495617)
- **Width**: 3 (capacity 2, rate 1)
- **Full Rounds**: 8
- **Partial Rounds**: 57
- **Alpha**: 5
- **Security Level**: 128 bits
- **MDS Matrix**: Precomputed for BN128 field

### 1.2 Proof System Selection

**Recommended**: Groth16 (BN128 curve)

**Rationale**:
- Smallest proof size (~200 bytes)
- Fastest verification (~3ms on-chain)
- Gas efficient for Ethereum
- Mature tooling (snarkjs, circom)

**Alternative**: PLONK or STARKs for transparent setup

### 1.3 Trusted Setup

**Phase 1**: Universal Powers of Tau ceremony (can reuse existing)
**Phase 2**: Circuit-specific trusted setup

**Security Considerations**:
- Multi-party computation for Phase 2 with at least 10 independent participants
- Publicly verifiable ceremony transcripts
- Secure parameter generation with distributed key generation
- Toxic waste destruction ceremony

### 1.4 Field Element Encoding

All field elements in the BN128 scalar field must be encoded as:

1. **Field Element Representation**: Integer in range `[0, p-1]` where `p = 21888242871839275222246405745257275088548364400416034343698204186575808495617`
2. **Byte Encoding**: Big-endian 32-byte representation
3. **Validation**: Must be less than field modulus `p`
4. **Zero Padding**: For inputs smaller than 32 bytes, left-pad with zeros
5. **Address Encoding**: Ethereum addresses (20 bytes) are padded to 32 bytes with 12 leading zeros

**Example**:
```
address = 0x1234... (20 bytes)
padded_address = 0x0000000000000000000000001234... (32 bytes)
field_element = BigInt(padded_address) mod p
```

### 1.5 Proof Verification Flow

1. **Off-chain verification** (optional, by relayer):
   - Validate proof structure and field elements
   - Check public signals match expected format
   - Verify nullifier hasn't been used
   - Estimate gas cost

2. **On-chain verification** (required, by contract):
   - Verify Groth16 proof using verifier contract
   - Check `nullifierHashes[nullifierHash] == false`
   - Check `block.timestamp < claimDeadline`
   - Transfer tokens to recipient
   - Emit `Claimed` event

## 2. Merkle Tree Specifications

### 2.1 Tree Structure

- **Height**: 26 levels (supports 2^26 = 67M leaves)
- **Hash Function**: Poseidon (ZK-friendly)
- **Leaf Hashing**: H(address) where address is bytes20
- **Empty Leaves**: Hash of zero address or specific null value

### 2.2 Tree Construction

```python
# Pseudocode
tree = MerkleTree(height=26, hash_func=poseidon)
for address in qualified_accounts:
    tree.insert(poseidon_hash(address))
root = tree.root()
```

### 2.3 Storage Requirements

- **Number of Nodes**: 2^27 - 1 = 134,217,727 nodes
- **Full Tree**: ~4.3GB (134,217,727 nodes × 32 bytes)
- **Compressed (pruned)**: ~8.3GB (65,000,000 leaves × 128 bytes for path data)
- **Proof Size**: 26 × 32 bytes = 832 bytes per path

### 2.4 Distribution Strategy

Options:
1. **Full Tree**: Host as downloadable file (~4.3GB)
2. **API Service**: Provide Merkle paths on-demand via API
3. **IPFS**: Distributed storage with pinning
4. **Torrent**: P2P distribution for resilience

### 2.5 File Formats

#### Merkle Tree Binary Format
```rust
// Header (16 bytes)
struct TreeHeader {
    magic: [u8; 4],        // "ZKPT" (0x5A4B5054)
    version: u8,           // Format version (1)
    height: u8,            // Tree height (26)
    reserved: [u8; 2],     // Reserved for future use
    num_leaves: u32,       // Number of leaves (65,000,000)
    root_hash: [u8; 32],   // Merkle root
}

// Leaf Data (20 bytes per leaf)
struct LeafData {
    address: [u8; 20],     // Ethereum address
}

// Complete file format:
// [TreeHeader][LeafData 0][LeafData 1]...[LeafData N]
// Total size: 16 + (65,000,000 * 20) ≈ 1.3GB

// Alternative: Pruned tree with only hashes
// [TreeHeader][LeafHash 0][LeafHash 1]...[LeafHash N]
// Total size: 16 + (65,000,000 * 32) ≈ 2.1GB
```

#### Merkle Tree JSON Format (API)
```json
{
  "version": 1,
  "height": 26,
  "num_leaves": 65000000,
  "root": "0x1234...",
  "leaves": [
    {
      "index": 0,
      "address": "0x00000000000000c0d7d3017b342ff039b55b0879",
      "leaf_hash": "0xabcd...",
      "path": ["0x...", "0x...", ...],
      "indices": [0, 1, 0, ...]
    }
  ]
}
```

#### Compressed Format
For efficient distribution, use a compressed format:
- **Zstandard compression**: ~70% compression ratio
- **Chunked downloads**: 100MB chunks for progressive loading
- **Checksums**: SHA256 for each chunk

## 3. Smart Contract Architecture

### 3.1 ZKP Token Contract

```solidity
contract ZKPToken is ERC20, Ownable {
    constructor(uint256 initialSupply) ERC20("ZKP Token", "ZKP") {
        _mint(msg.sender, initialSupply);
    }
    
    function mint(address to, uint256 amount) external onlyOwner {
        _mint(to, amount);
    }
}
```

### 3.2 Airdrop Contract

```solidity
contract PrivacyAirdrop {
    // State variables
    bytes32 public immutable merkleRoot;
    mapping(bytes32 => bool) public nullifierHashes;
    IERC20 public immutable token;
    uint256 public immutable claimAmount;
    uint256 public immutable claimDeadline;
    
    // Groth16 verifier contract
    IVerifier public immutable verifier;
    
    // Events
    event Claimed(bytes32 indexed nullifierHash, address indexed recipient);
    
    // ZK Proof structure
    struct Proof {
        uint[2] a;
        uint[2][2] b;
        uint[2] c;
    }
    
    constructor(
        address _token,
        bytes32 _merkleRoot,
        uint256 _claimAmount,
        uint256 _claimDeadline,
        address _verifier
    ) {
        token = IERC20(_token);
        merkleRoot = _merkleRoot;
        claimAmount = _claimAmount;
        claimDeadline = _claimDeadline;
        verifier = IVerifier(_verifier);
    }
    
    function claim(
        Proof calldata proof,
        bytes32 nullifierHash,
        address recipient
    ) external {
        require(block.timestamp < claimDeadline, "Claim period ended");
        require(!nullifierHashes[nullifierHash], "Already claimed");
        
        // Verify proof
        uint[3] memory publicSignals = [
            uint256(merkleRoot),
            uint256(uint160(recipient)),
            uint256(nullifierHash)
        ];
        
        require(verifier.verifyProof(
            proof.a,
            proof.b,
            proof.c,
            publicSignals
        ), "Invalid proof");
        
        // Mark as claimed
        nullifierHashes[nullifierHash] = true;
        
        // Transfer tokens
        require(token.transfer(recipient, claimAmount), "Token transfer failed");
        
        emit Claimed(nullifierHash, recipient);
    }
    
    function estimateClaimGas(
        Proof calldata proof,
        bytes32 nullifierHash,
        address recipient
    ) external view returns (uint256) {
        // Gas estimation for claim transaction
        // This is approximate and may vary based on network conditions
        return 500000; // Base gas estimate
    }
}
```

### 3.3 Relayer Registry Contract

```solidity
contract RelayerRegistry {
    mapping(address => bool) public authorizedRelayers;
    mapping(address => uint256) public relayerBalances;
    
    event RelayerAuthorized(address indexed relayer);
    event DonationReceived(address indexed donor, uint256 amount);
    event FundsWithdrawn(address indexed relayer, uint256 amount);
    
    function authorizeRelayer(address relayer) external onlyOwner {
        authorizedRelayers[relayer] = true;
        emit RelayerAuthorized(relayer);
    }
    
    function donate() external payable {
        relayerBalances[authorizedRelayers[msg.sender] ? msg.sender : defaultRelayer] += msg.value;
        emit DonationReceived(msg.sender, msg.value);
    }
    
    function withdraw(uint256 amount) external {
        require(authorizedRelayers[msg.sender], "Not authorized");
        require(relayerBalances[msg.sender] >= amount, "Insufficient balance");
        relayerBalances[msg.sender] -= amount;
        payable(msg.sender).transfer(amount);
        emit FundsWithdrawn(msg.sender, amount);
    }
}
```

### 3.4 Gas Optimizations

- Use `calldata` for proof arrays
- Pack variables where possible
- Use `immutable` for constants
- Consider `claimBatch` for multiple claims (if needed)
- Gas refunds for relayers

## 4. Rust CLI Tool Specification

### 4.1 Dependencies

```toml
[dependencies]
ethers = "2.0"
circom-compat = "0.5"
ark-circom = "0.5"
ark-bn254 = "0.4"
ark-groth16 = "0.4"
ark-serialize = "0.4"
serde = { version = "1.0", features = ["derive"] }
clap = { version = "4.0", features = ["derive"] }
tokio = { version = "1.37", features = ["full"] }
reqwest = { version = "0.12", features = ["json"] }
sha3 = "0.10"
hex = "0.4"
anyhow = "1.0"
zeroize = "1.7"  # For secure memory clearing
secp256k1 = "0.28"  # For ECDSA operations
```

### 4.2 CLI Interface

```bash
# Generate proof
zkp-airdrop generate-proof \
  --private-key <PRIVATE_KEY> \
  --recipient <RECIPIENT_ADDRESS> \
  --merkle-tree <TREE_FILE_OR_API> \
  --output <PROOF_FILE>

# Verify proof locally
zkp-airdrop verify-proof \
  --proof <PROOF_FILE> \
  --merkle-root <ROOT>

# Submit to relayer
zkp-airdrop submit \
  --proof <PROOF_FILE> \
  --relayer-url <RELAYER_ENDPOINT> \
  --recipient <RECIPIENT_ADDRESS>

# Check claim status
zkp-airdrop check-status \
  --nullifier <NULLIFIER_HASH>
```

### 4.3 Core Functions

```rust
pub fn generate_proof(
    private_key: &[u8; 32],
    recipient: Address,
    merkle_tree: &MerkleTree,
) -> Result<ProofData> {
    // 1. Derive Ethereum address from private key
    let public_key = secp256k1::PublicKey::from_secret_key(private_key);
    let public_key_bytes = public_key.serialize_uncompressed();
    let address_bytes = keccak256(&public_key_bytes[1..])[12..32].to_vec(); // Last 20 bytes
    let address = Address::from_slice(&address_bytes);
    
    // 2. Compute leaf hash (Poseidon hash of address)
    let leaf = poseidon_hash(&address_bytes);
    
    // 3. Find Merkle path
    let (path, indices) = merkle_tree.get_path(&leaf)?;
    
    // 4. Generate nullifier (Poseidon hash of private_key || recipient)
    let mut nullifier_input = private_key.to_vec();
    nullifier_input.extend_from_slice(recipient.as_bytes());
    let nullifier = poseidon_hash(&nullifier_input);
    
    // 5. Build circuit inputs
    let inputs = CircuitInputs {
        private_key: private_key.to_vec(),
        merkle_path: path,
        merkle_path_indices: indices,
        recipient: recipient.as_bytes().to_vec(),
        merkle_root: merkle_tree.root(),
        nullifier: nullifier.to_vec(),
    };
    
    // 6. Generate proof
    let proof = groth16::generate_proof(&inputs)?;
    
    Ok(ProofData {
        proof,
        public_inputs: vec![
            merkle_tree.root().to_vec(),
            recipient.as_bytes().to_vec(),
            nullifier.to_vec(),
        ],
        nullifier_hash: nullifier,
    })
}
```

### 4.4 Security Features

- Private key never leaves the local machine
- Secure memory handling (zeroize after use)
- No logging of sensitive data
- Offline mode support (generate proof without internet)

## 5. Web Relayer Service

### 5.1 Technology Stack

- **Runtime**: Rust (Actix-web or Axum)
- **Database**: PostgreSQL (for metrics, not proof data)
- **Cache**: Redis (rate limiting)
- **Queue**: Optional (Redis/RabbitMQ) for high throughput
- **Monitoring**: Prometheus + Grafana

### 5.2 API Endpoints

```rust
// POST /api/v1/submit-claim
#[derive(Deserialize)]
struct SubmitClaimRequest {
    proof: ZKProof,
    recipient: Address,
    nullifier_hash: H256,
}

#[derive(Serialize)]
struct SubmitClaimResponse {
    success: bool,
    tx_hash: Option<H256>,
    error: Option<String>,
}

// POST /api/v1/donate
#[derive(Deserialize)]
struct DonateRequest {
    amount: U256,  // In wei
}

// GET /api/v1/stats
#[derive(Serialize)]
struct StatsResponse {
    total_claims: u64,
    total_tokens_distributed: U256,
    relayer_balance: U256,
    pending_claims: u64,
}

// GET /api/v1/health
#[derive(Serialize)]
struct HealthResponse {
    status: String,
    relayer_balance: U256,
    gas_price: U256,
}
```

### 5.3 Relayer Logic

```rust
pub async fn submit_claim(
    request: SubmitClaimRequest,
    state: &AppState,
) -> Result<SubmitClaimResponse> {
    // 1. Rate limiting check
    check_rate_limit(&request.nullifier_hash, state).await?;
    
    // 2. Verify nullifier not already used
    if state.contract.is_nullifier_used(request.nullifier_hash).await? {
        return Err(Error::AlreadyClaimed);
    }
    
    // 3. Verify proof off-chain (optional, contract will verify)
    // This saves gas by rejecting invalid proofs early
    if !verify_proof_offchain(&request.proof, &state.verifying_key)? {
        return Err(Error::InvalidProof);
    }
    
    // 4. Check relayer has enough gas
    let gas_price = state.provider.get_gas_price().await?;
    let estimated_gas = state.contract.estimate_claim_gas(&request).await?;
    let relayer_balance = state.provider.get_balance(state.relayer_address, None).await?;
    
    if relayer_balance < gas_price * estimated_gas {
        return Err(Error::InsufficientRelayerFunds);
    }
    
    // 5. Submit transaction
    let tx_hash = state.contract.claim(
        request.proof,
        request.nullifier_hash,
        request.recipient,
    ).await?;
    
    // 6. Record metrics
    record_claim_submission(&request.nullifier_hash, &tx_hash, state).await;
    
    Ok(SubmitClaimResponse {
        success: true,
        tx_hash: Some(tx_hash),
        error: None,
    })
}
```

### 5.4 Rate Limiting & Anti-Spam

- **Per-nullifier**: 1 request per minute
- **Per-IP**: 10 requests per hour
- **Global**: 100 requests per minute
- **Proof validation**: Must pass off-chain verification

### 5.5 Monitoring & Alerting

Metrics to track:
- Claims submitted per hour/day
- Success/failure rate
- Average gas cost per claim
- Relayer balance
- Response times
- Invalid proof attempts

Alerts for:
- Relayer balance below threshold (e.g., 0.5 ETH)
- High error rate (>5%)
- Unusual claim patterns

## 6. Deployment Architecture

### 6.1 Infrastructure

```
┌─────────────────────────────────────────────────────────────┐
│                        Load Balancer                         │
└───────────────────────────┬─────────────────────────────────┘
                            │
        ┌───────────────────┼───────────────────┐
        ▼                   ▼                   ▼
┌───────────────┐   ┌───────────────┐   ┌───────────────┐
│  Relayer #1   │   │  Relayer #2   │   │  Relayer #N   │
│  (Docker)     │   │  (Docker)     │   │  (Docker)     │
└───────┬───────┘   └───────┬───────┘   └───────┬───────┘
        │                   │                   │
        └───────────────────┼───────────────────┘
                            ▼
                    ┌───────────────┐
                    │   PostgreSQL  │
                    │   (Metrics)   │
                    └───────────────┘
                            ▼
                    ┌───────────────┐
                    │     Redis     │
                    │ (Rate Limit)  │
                    └───────────────┘
```

### 6.2 Environment Configuration

```yaml
# config.yaml
network:
  rpc_url: "https://ethereum-mainnet.infura.io/v3/..."
  chain_id: 1
  
contract:
  airdrop_address: "0x..."
  token_address: "0x..."
  
relayer:
  private_key: "${RELAYER_PRIVATE_KEY}"
  min_balance: "500000000000000000"  # 0.5 ETH
  gas_price_multiplier: 1.1
  max_gas_price: "50000000000"  # 50 gwei
  
rate_limit:
  per_nullifier: 60  # seconds
  per_ip: 10  # requests per hour
  global: 100  # requests per minute
  
merkle_tree:
  source: "https://api.merkle-tree.io/tree.json"
  cache_path: "/data/merkle_tree.bin"
```

## 7. Security Considerations

### 7.1 Smart Contract Security

- **Reentrancy protection**: Use OpenZeppelin's ReentrancyGuard
- **Access control**: Only relayers can submit claims (optional)
- **Overflow protection**: Solidity 0.8+ built-in checks
- **Emergency pause**: Pausable contract
- **Upgradeability**: Consider proxy pattern for bug fixes

### 7.2 ZK Circuit Security

- **Formal verification**: Verify circuit constraints
- **Audits**: Third-party audit of circuits
- **Trusted setup**: Secure MPC ceremony
- **Input validation**: Sanitize all public inputs

### 7.3 Relayer Security

- **Private key management**: Use AWS KMS, HashiCorp Vault, or hardware security modules
- **DDoS protection**: Cloudflare, AWS Shield, or equivalent
- **API authentication**: API keys for monitoring endpoints only (claim submission is permissionless)
- **Request validation**: Strict input validation and sanitization
- **Logging**: No sensitive data in logs (private keys, addresses, proof data)
- **Open source**: All relayer code is publicly auditable
- **Multi-relayer ecosystem**: No single point of failure, users can choose any relayer

### 7.4 Privacy Considerations

- **Timing attacks**: Relayer submits claims in batches with delays
- **Gas price correlation**: Use randomized gas prices
- **Metadata leakage**: HTTPS only, no CORS on sensitive endpoints
- **Blockchain analysis**: Document potential leakage vectors

## 8. Testing Strategy

### 8.1 Unit Tests

- Circuit constraint satisfaction
- Smart contract function testing
- CLI command testing
- Relayer endpoint testing

### 8.2 Integration Tests

- End-to-end claim flow
- Multiple simultaneous claims
- Relayer failure scenarios
- Gas estimation accuracy

### 8.3 Security Tests

- Proof forgery attempts
- Double-spend attempts
- Invalid input handling
- Rate limit bypass attempts

### 8.4 Load Testing

- 1000+ concurrent claims
- Merkle tree query performance
- Relayer throughput
- Database connection pooling

## 9. Maintenance & Operations

### 9.1 Monitoring Dashboard

- Real-time claim statistics
- Relayer health metrics
- Gas price tracking
- Error rate monitoring

### 9.2 Maintenance Tasks

- **Daily**: Check relayer balance, review error logs
- **Weekly**: Analyze claim patterns, optimize gas settings
- **Monthly**: Security review, dependency updates

### 9.3 Incident Response

1. **Relayer out of funds**: Emergency fund allocation, alert donors
2. **Smart contract bug**: Pause contract, deploy fix
3. **Circuit vulnerability**: Halt claims, assess impact
4. **DDoS attack**: Scale infrastructure, implement stricter rate limits
