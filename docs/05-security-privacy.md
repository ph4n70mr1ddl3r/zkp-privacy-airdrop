# Security & Privacy Considerations

**Version**: 1.0.0  
**Last Updated**: 2026-02-07  
**Based on**: [Technical Specification](../docs/02-technical-specification.md) and [Unified Specification](../docs/00-specification.md)

## Threat Model

### Actors

1. **Claimant**: User with private key in qualified list
2. **Relayer**: Service that submits claims on behalf of users
3. **Attacker**: External party trying to compromise the system
4. **Observer**: Entity monitoring blockchain activity

### Assets

1. **ZKP Tokens**: Value to be distributed
2. **Private Keys**: Control over qualified accounts
3. **Proofs**: Cryptographic evidence of eligibility
4. **Merkle Tree**: List of qualified accounts

### Threats

| Threat | Impact | Likelihood | Risk Level |
|--------|--------|------------|------------|
| Proof forgery | Critical | Low | High |
| Double spending | Critical | Medium | High |
| Relayer compromise | High | Low | Medium |
| Timing analysis | Medium | High | Medium |
| Merkle tree leak | Low | Low | Low |
| Front-running | Medium | Medium | Medium |
| DoS on relayer | Medium | Medium | Medium |

## Security Measures

### 1. Cryptographic Security

#### ZK Circuit Security

**Constraints**:
- Circuit must not allow creation of valid proofs without knowing private key
- Nullifier must be deterministic to prevent double-spends
- Public inputs must be bound to proof to prevent replay attacks

**Mitigations**:
- Formal verification of circuit constraints
- Third-party audit of circuit logic
- Trusted setup ceremony with multiple participants
- Public verification of setup transcripts

#### Hash Function Selection

**Poseidon Hash**:
- ZK-friendly, reduces circuit size
- Security level: 128-bit
- Resistant to algebraic attacks

**Keccak-256**:
- Standard Ethereum hash
- Used for address derivation
- Well-studied and trusted

### 2. Smart Contract Security

#### Access Control

```solidity
// Allow anyone to submit claims for maximum decentralization
// Relayers are optional services for users who don't want to pay gas
// No centralized control - anyone can call the claim() function

// Important: This maximizes censorship resistance but requires
// additional spam prevention measures (gas costs, rate limiting)
```

#### Reentrancy Protection

```solidity
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";

contract PrivacyAirdrop is ReentrancyGuard {
    function claim(...) external nonReentrant {
        // ... claim logic
    }
}
```

#### Integer Overflow

Solidity 0.8+ has built-in overflow checks. For earlier versions:
```solidity
import "@openzeppelin/contracts/utils/math/SafeMath.sol";
```

#### Emergency Controls

```solidity
import "@openzeppelin/contracts/security/Pausable.sol";

contract PrivacyAirdrop is Pausable, Ownable {
    function pause() external onlyOwner {
        _pause();
    }
    
    function claim(...) external whenNotPaused {
        // ...
    }
}
```

### 3. Relayer Security

#### Infrastructure Security

- **Private Key Management**: Use AWS KMS, HashiCorp Vault, or similar
- **Network Security**: Firewall rules, VPC, private subnets
- **DDoS Protection**: Cloudflare, AWS Shield
- **Monitoring**: 24/7 alerting for anomalies

#### Rate Limiting

```rust
// Pseudocode
async fn rate_limit_check(ip: IpAddr, nullifier: Hash) -> Result<()> {
    // Check IP rate limit
    let ip_count = redis.get::<u32>(format!("rate_limit:ip:{}", ip)).await?;
    if ip_count > IP_LIMIT {
        return Err(Error::RateLimited);
    }
    
    // Check nullifier rate limit
    let nullifier_exists = redis.exists(format!("rate_limit:nullifier:{}", nullifier)).await?;
    if nullifier_exists {
        return Err(Error::RateLimited);
    }
    
    // Set nullifier rate limit (60 seconds)
    redis.setex(format!("rate_limit:nullifier:{}", nullifier), 60, true).await?;
    
    Ok(())
}
```

#### Input Validation

```rust
fn validate_proof(proof: &ZKProof) -> Result<()> {
    // Check proof structure
    if proof.pi_a.len() != 3 || proof.pi_b.len() != 3 || proof.pi_c.len() != 3 {
        return Err(Error::InvalidProofStructure);
    }
    
    // Check field element bounds (must be < bn128 prime)
    let prime = BigUint::from_str("21888242871839275222246405745257275088548364400416034343698204186575808495617")?;
    for val in [&proof.pi_a, &proof.pi_b, &proof.pi_c].iter().flatten() {
        let num = BigUint::from_str(val)?;
        if num >= prime {
            return Err(Error::FieldElementTooLarge);
        }
    }
    
    Ok(())
}
```

### 4. CLI Security

#### Secure Memory Handling

```rust
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Zeroize, ZeroizeOnDrop)]
struct PrivateKey([u8; 32]);

impl Drop for PrivateKey {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}
```

#### No Logging of Secrets

```rust
// BAD
log::info!("Generating proof for private key: {}", private_key);

// GOOD
log::info!("Generating proof for address: {}", derive_address(&private_key));
```

#### Secure Input Methods

```rust
// Read from environment variable (secure)
let private_key = std::env::var("PRIVATE_KEY")?;

// Read from file with restricted permissions
let mut file = fs::OpenOptions::new()
    .read(true)
    .mode(0o600)  // Owner read/write only
    .open(private_key_path)?;

// Read from stdin (masked)
use rpassword::read_password;
let private_key = read_password()?;
```

## Privacy Guarantees

### 1. Identity Privacy

**Claim**: The claimant's original address is never revealed on-chain.

**Proof**: 
- Only the Merkle root is public
- The proof demonstrates knowledge of a leaf without revealing which leaf
- The nullifier is derived from private_key and recipient, not the address

**Verification**:
```
H(address) is in Merkle tree
↓
Proof is valid
↓
Recipient receives tokens

At no point is address revealed
```

### 2. Claim Unlinkability

**Claim**: Multiple claims from the same account cannot be linked.

**Implementation**:
```rust
// Nullifier = Poseidon(private_key, recipient)
// Different recipient → Different nullifier
let nullifier = poseidon_hash(&[private_key, recipient_bytes]);
```

**Analysis**:
- Nullifier is unique per (private_key, recipient) pair
- Same private key with different recipients = different nullifiers
- Cannot determine if two claims came from same private key

### 3. Timing Privacy

**Threat**: Analyzing claim timing to correlate with blockchain activity.

**Mitigations**:
- Relayer introduces random delays (1-60 seconds)
- Claims are batched and submitted in random order
- Gas price randomization: Base fee × 1.1 × (1 + random(0, 0.05))
- No immediate confirmation to claimant

```rust
async fn submit_with_delay(proof: ZKProof) {
    let delay = rand::random::<u64>() % 60;
    tokio::time::sleep(Duration::from_secs(delay)).await;
    
    let gas_price = get_randomized_gas_price();
    submit_transaction(proof, gas_price).await;
}

fn get_randomized_gas_price() -> U256 {
    let base_fee = get_base_fee();
    let multiplier = Decimal::from_ratio(11, 10); // 1.1x base
    let random_factor = Decimal::from_ratio(rand::random::<u32>() % 6, 100); // 0-5%
    let randomized_multiplier = multiplier * (Decimal::one() + random_factor);
    let gas_price = base_fee * randomized_multiplier;
    min(gas_price, U256::from(50_000_000_000)) // Cap at 50 gwei
}
```

### 4. Network Privacy

**Threat**: IP address correlation with claims.

**Mitigations**:
- Relayer accepts proofs via HTTPS only
- No logging of IP addresses
- CDN for additional anonymization
- Tor .onion service (optional)

### 5. Metadata Leakage

**Potential Leaks**:
1. **Gas price**: Unique gas price patterns
2. **Timestamp**: Precise claim timing
3. **Transaction nonce**: Sequential patterns
4. **Client version**: User-agent strings

**Mitigations**:
- Randomized gas prices: Base fee × 1.1 × (1 + random(0, 0.05)), capped at 50 gwei
- Batch processing with random delays (1-60 seconds) obscures individual timestamps
- Relayer manages nonces, not claimants
- No identifying headers in requests
- All gas prices rounded to nearest wei (not gwei) to prevent pattern recognition

## Attack Vectors & Defenses

### 1. Proof Forgery

**Attack**: Create valid proof without knowing private key.

**Defense**:
- Soundness of Groth16 (computational infeasible to forge)
- Proper trusted setup
- Formal verification of circuit

### 2. Double Spending

**Attack**: Claim twice with same nullifier.

**Defense**:
```solidity
require(!nullifiers[nullifier], "Already claimed");
nullifiers[nullifier] = true;
```

### 3. Replay Attack

**Attack**: Replay someone else's proof.

**Defense**:
- Recipient address is public input
- Nullifier is unique per recipient
- Cannot replay to different recipient

### 4. Front-Running

**Attack**: Observe proof in mempool and submit first.

**Defense**:
- Relayer submits directly (no mempool)
- Commit-reveal scheme (if decentralized)
- Flashbots/private transactions

### 5. Relayer Censorship

**Attack**: Relayer refuses to submit certain proofs.

**Defense**:
- Contract allows anyone to submit proofs directly
- Relayers are optional convenience services
- Multiple relayers for redundancy
- Self-relay option (claimant pays gas)
- Open-source relayer code

### 6. Denial of Service

**Attack**: Flood relayer with invalid proofs.

**Defense**:
- Rate limiting
- Proof validation before submission
- Resource consumption limits
- Auto-scaling infrastructure

### 7. Merkle Tree Poisoning

**Attack**: Include invalid addresses in tree.

**Defense**:
- Public verification of tree construction
- Open-source tree generation code
- Community audit of qualified list
- On-chain commitment to tree root

### 8. Timing Analysis

**Attack**: Correlate claim time with user activity.

**Defense**:
- Randomized delays
- Batch processing
- Time zone distribution analysis

## Privacy Limitations

### 1. Amount Privacy

**Issue**: All claims are for the same amount (or limited set of amounts).

**Limitation**: If different claim amounts exist, they could correlate with account balances.

**Mitigation**: Fixed claim amount for all participants.

### 2. Recipient Linkability

**Issue**: If recipient address is known to be linked to claimant.

**Example**: User sends tokens from recipient to known address.

**Limitation**: This is outside the scope of the airdrop privacy.

**Mitigation**: User education about post-claim privacy.

### 3. Merkle Tree Size

**Issue**: 65,249,064 leaves may allow some statistical analysis.

**Limitation**: If tree is public, attackers know the set of possible claimants.

**Mitigation**: Large set makes individual identification difficult.

### 4. Relayer Trust

**Issue**: Relayer sees the proof before submission.

**Limitation**: Malicious relayer could theoretically log recipient/nullifier correlations.

**Mitigation**:
- Open-source relayer code
- Multiple independent relayers
- Cryptographic auditing (if needed)

## Security Checklist

### Pre-Deployment

- [ ] Circuit formally verified
- [ ] Smart contracts audited by 3+ firms
- [ ] Trusted setup ceremony completed with 10+ participants
- [ ] Relayer infrastructure penetration tested
- [ ] CLI security audit
- [ ] Bug bounty program configured
- [ ] Emergency pause procedures documented
- [ ] Incident response plan prepared

### Post-Deployment

- [ ] Monitor for unusual claim patterns
- [ ] Track relayer health metrics
- [ ] Community monitoring for attacks
- [ ] Regular security updates
- [ ] Quarterly security reviews
- [ ] Annual comprehensive audit

## Incident Response

### Severity Levels

1. **Critical**: Active exploit, fund drainage possible
2. **High**: Vulnerability found, no active exploit
3. **Medium**: Potential issue, needs investigation
4. **Low**: Minor concern, track for future

### Response Procedures

**Critical Incident**:
1. Pause contract immediately
2. Alert team via emergency channel
3. Assess scope of exposure
4. Deploy fix or migrate
5. Post-mortem and disclosure

**High Incident**:
1. Document vulnerability
2. Develop fix
3. Schedule deployment
4. Monitor for exploitation attempts

### Communication

- **Internal**: Slack/Discord security channel
- **External**: Twitter, Discord, official channels
- **Disclosure**: Follow responsible disclosure practices
