# Documentation Consistency Checklist

**Version**: 1.0.0  
**Created**: 2026-02-07  
**Purpose**: Ensure consistency across all documentation files

## Critical Items to Verify

### 1. Cryptographic Constants
- [ ] **Nullifier hash input**: Must be `private_key (32 bytes) || recipient (20 bytes) || padding (12 bytes zeros)` = 64 bytes total
- [ ] **Poseidon parameters**: width=3, full_rounds=8, partial_rounds=57, alpha=5
- [ ] **Field modulus**: BN128 scalar field = `21888242871839275222246405745257275088548364400416034343698204186575808495617`
- [ ] **secp256k1 order**: `0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141`

### 2. Numerical Constants
- [ ] **Qualified accounts**: 65,249,064
- [ ] **Claim amount**: 1,000 ZKP tokens (with 18 decimals = `1000 * 10**18`)
- [ ] **Total supply**: 65,249,064,000 ZKP tokens
- [ ] **Merkle tree height**: 26 levels
- [ ] **Max leaves**: 2^26 = 67,108,864
- [ ] **Claim period**: 90 days

### 2.1 Data Source Verification
- [ ] **Accounts file location**: https://drive.google.com/file/d/1yvgsuDMhamUoKAfH59iuyDtm7x5mnHRX/view?usp=sharing
- [ ] **Download command**: `gdown 1yvgsuDMhamUoKAfH59iuyDtm7x5mnHRX`
- [ ] **File verification**: SHA256 checksum of accounts.csv matches documented value

### 3. Storage Sizes
- [ ] **Full tree storage**: 4.00 GiB (134,217,727 nodes × 32 bytes)
- [ ] **Address-only file**: 1.216 GiB (header + 65,249,064 × 20 bytes)
- [ ] **Hashes-only file**: 1.945 GiB (header + 65,249,064 × 32 bytes)
- [ ] **Proof data per claim**: 832 bytes (26 × 32 bytes for Merkle path)
- [ ] **Groth16 proof size**: ~200 bytes
- [ ] **Total proof package**: ~1,032 bytes

### 4. Gas Estimates
- [ ] **Proof verification**: 300,000 gas
- [ ] **Total claim transaction**: 500,000 gas
- [ ] **Maximum claim gas**: 1,000,000 gas
- [ ] **Relayer buffer**: 200,000 gas

### 5. Data Formats
#### Proof JSON Format
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

#### Field Element Encoding
- **Primary format**: Decimal strings
- **Alternative format**: Hex strings with `0x` prefix
- **Validation**: Must be < BN128 field modulus

#### Address Encoding
- 20-byte hex strings with `0x` prefix (lowercase)
- EIP-55 checksum optional but recommended

#### Hash Encoding
- 32-byte hashes as 64-character hex strings with `0x` prefix (lowercase)

### 6. System Design Decisions
- [ ] **Relayer trust model**: Anyone can submit claims directly; relayers are optional
- [ ] **Access control**: No centralized control in smart contract
- [ ] **Privacy model**: Cryptographic privacy, not complete anonymity
- [ ] **Trusted setup**: Multi-party ceremony required after security audits

### 7. Security Parameters
- [ ] **Rate limits per nullifier**: 1 request per 60 seconds
- [ ] **Rate limits per IP**: 100 requests per 60 seconds
- [ ] **Global rate limit**: 1,000 requests per 60 seconds
- [ ] **Gas price randomization**: Base fee × 1.1 × (1 + random(0, 0.05)), capped at 50 gwei
- [ ] **Minimum relayer balance**: 0.5 ETH (critical), 1.0 ETH (warning)

## Verification Process

### Before Updating Documentation
1. Check if change affects any constants in `00-specification.md`
2. If yes, update `00-specification.md` first
3. Update all other documents that reference the changed constants
4. Run this checklist to verify consistency

### When Adding New Features
1. Add constants to `00-specification.md` first
2. Update relevant technical documentation
3. Update API reference if needed
4. Update security/privacy analysis if applicable
5. Update implementation roadmap if timeline affected

### Regular Reviews
- [ ] Weekly: Check for inconsistencies in constants
- [ ] Monthly: Full documentation review
- [ ] Pre-release: Complete consistency audit

## Common Inconsistency Patterns to Watch For

### 1. Copy-Paste Errors
- Verify all numerical calculations
- Check units (bytes vs GB vs MB)
- Verify token amounts with decimals

### 2. Format Inconsistencies
- JSON structure variations
- Field element encoding (decimal vs hex)
- Date/time formats

### 3. Design Decision Drift
- Security model changes
- Privacy guarantees
- System architecture

### 4. Timeline Conflicts
- Development phases
- Dependency ordering
- Resource allocation

## Tools for Verification

### Manual Checks
```bash
# Check for numerical inconsistencies
grep -n "65,249,064" docs/*.md
grep -n "1,000" docs/*.md
grep -n "nullifier" docs/*.md

# Check file size calculations
grep -n "GB" docs/*.md | grep -i "tree\|storage"
```

### Automated Checks (Future)
Consider implementing:
- Documentation linting
- Constant validation scripts
- Cross-reference checking
- Version compatibility validation

## Version History

### v1.4.0 (2026-02-07)
- **Migrated to Optimism**: Changed deployment from Ethereum mainnet to Optimism L2
- **Updated network config**: Mainnet = Optimism, Testnet = Optimism Sepolia
- **Adjusted gas prices**: Maximum gas price reduced from 50 gwei to 0.1 gwei for Optimism
- **Updated RPC endpoints**: Changed to Optimism RPC URLs
- **Added cost comparison**: Showed 50x gas savings on Optimism
- **Updated CLI options**: Network options changed to optimism/optimism-sepolia

### v1.3.0 (2026-02-07)
- Added detailed claimant workflow perspective to documentation
- Documented accounts qualification criteria: ≥0.004 ETH gas fees paid by Dec 31, 2025
- Clarified contract immutability and direct claim option
- Emphasized relayers as optional, community-funded services
- Created CLAIMANT_GUIDE.md with step-by-step instructions
- Updated architecture diagrams to show both claim paths (relayer vs direct)
- Added explicit notes about contract being immutable and permissionless

### v1.2.0 (2026-02-07)
- Added accounts.csv download information to all relevant documents
- Documented accounts file source: https://drive.google.com/file/d/1yvgsuDMhamUoKAfH59iuyDtm7x5mnHRX/view?usp=sharing
- Added download command: `gdown 1yvgsuDMhamUoKAfH59iuyDtm7x5mnHRX`
- Added data source verification to consistency checklist

### v1.1.0 (2026-02-07)
- Fixed nullifier hash input inconsistency in unified specification (line 93)
- Standardized gas estimates across all documents (300K verification, 200K storage+transfer, 500K total, 200K buffer, 700K estimated, 1M max)
- Clarified relayer trust model (anyone can submit directly, relayers are optional)
- Fixed proof data size ambiguity (936 bytes = 832 Merkle path + 32 leaf hash + 104 indices)
- Standardized storage units to GiB (gibibytes = 2³⁰ bytes)
- Added security fixes week after audit (Week 13) in implementation roadmap
- Updated gas breakdown in technical specification (300K + 200K = 500K, not 300K + 150K + 50K)
- Corrected GB vs GiB usage throughout documentation

### v1.0.0 (2026-02-07)
- Initial consistency checklist created
- Standardized proof JSON format
- Added version headers to all documents

## References

- [Unified Specification](./00-specification.md) - Single source of truth
- [Technical Specification](./02-technical-specification.md) - Detailed implementation
- [API Reference](./04-api-reference.md) - External interfaces
- [Security & Privacy](./05-security-privacy.md) - Security considerations