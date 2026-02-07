# Consistency Checklist

**Version**: 1.0.0  
**Last Updated**: 2026-02-07  

This document serves as a verification guide to ensure all documentation and implementations remain consistent with the [Unified Specification](./00-specification.md), which is the single source of truth.

## Document Relationships

```
00-specification.md (Source of Truth)
       ↓
01-overview.md (High-level summary)
       ↓
02-technical-specification.md (Detailed implementation)
       ↓
03-implementation-roadmap.md (Development plan)
       ↓
04-api-reference.md (External interfaces)
       ↓
05-security-privacy.md (Security considerations)
       ↓
06-privacy-analysis.md (Privacy limitations)
       ↓
07-consistency-checklist.md (This document)
```

## Core Constants Verification

Check these values match across all documents:

| Constant | Value in 00-specification.md | Verify in Other Docs |
|----------|------------------------------|----------------------|
| Qualified accounts | 65,249,064 | All docs |
| Claim amount | 1,000 ZKP tokens | All docs |
| Token decimals | 18 | All docs |
| Merkle tree height | 26 | All docs |
| Tree leaves | 65,249,064 | All docs |
| Tree capacity | 67,108,864 | All docs |
| Empty leaves | 1,859,800 | All docs |
| Poseidon width | 3 | All docs |
| Poseidon full rounds | 8 | All docs |
| Poseidon partial rounds | 57 | All docs |
| Poseidon alpha | 5 | All docs |
| BN128 prime | 21888242871839275222246405745257275088548364400416034343698204186575808495617 | All docs |
| secp256k1 order | 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141 | All docs |
| Claim period | 90 days | All docs |
| Verify proof gas | 300,000 | All docs |
| Storage/transfer gas | 200,000 | All docs |
| Claim gas (total) | 500,000 | All docs |
| Relayer buffer | 200,000 | All docs |
| Estimated claim gas | 700,000 | All docs |
| Max claim gas | 1,000,000 | All docs |
| Optimism chain ID | 10 | 04-api-reference.md:175 |
| Gas price cap | 0.1 gwei | All docs |

## Critical Cross-Reference Checks

### 1. Nullifier Calculation
**Source**: `docs/00-specification.md:133-150`

Must match:
- `docs/00-specification.md:103` - `nullifier = Poseidon("zkp_airdrop_nullifier_v1" || private_key || zeros)`
- `docs/02-technical-specification.md:489` - `poseidon_hash_with_domain(private_key, "zkp_airdrop_nullifier_v1")`
- `docs/04-api-reference.md:844-876` - `Poseidon("zkp_airdrop_nullifier_v1" || private_key || zeros)`
- `docs/nullifier-fix-summary.md:13-17` - `nullifier = Poseidon("zkp_airdrop_nullifier_v1" || private_key || zeros)`

**Formula**: `nullifier = Poseidon("zkp_airdrop_nullifier_v1" || private_key || zeros)`

### 2. Proof Format
**Source**: `docs/00-specification.md:218-232`

Must match:
- `docs/04-api-reference.md:40-58` - JSON structure
- `docs/04-api-reference.md:770-801` - JSON schema

**Structure**:
```json
{
  "proof": {
    "a": ["field_element", "field_element"],  // 2 elements
    "b": [["field_element", "field_element"], ["field_element", "field_element"]],  // 2x2
    "c": ["field_element", "field_element"]   // 2 elements
  },
  "public_signals": ["merkle_root", "recipient", "nullifier"],  // 3 elements
  "nullifier": "0x...",
  "recipient": "0x...",
  "merkle_root": "0x...",
  "generated_at": "ISO8601_TIMESTAMP"
}
```

### 3. Field Element Encoding
**Source**: `docs/00-specification.md:152-161`

Must match:
- Primary format: **decimal strings**
- Alternative format: hex strings with `0x` prefix
- Must be < BN128 prime modulus

### 4. Merkle Tree Structure
**Source**: `docs/00-specification.md:26-45`

Must match:
- Height: 26 levels
- Leaves: 65,249,064
- Hash: Poseidon(BN128 scalar field)
- Leaf: `Poseidon(address)` where address padded to 32 bytes
- Empty leaf: `Poseidon(32 zero bytes)`

### 5. Rate Limiting
**Source**: `docs/00-specification.md:77-86`

Must match:
- Per nullifier: 1 request per 60 seconds
- Per IP: 100 requests per 60 seconds
- Global: 1,000 requests per 60 seconds
- Burst allowance: 2x limit for 10 seconds

## Implementation Consistency

### Smart Contracts
Check that all contract interfaces match:

| Function | Signature | Source |
|----------|-----------|--------|
| `claim` | `claim(Proof calldata proof, bytes32 nullifier, address recipient)` | `00-specification.md:186-190` |
| `isClaimed` | `isClaimed(bytes32 nullifier) returns (bool)` | `00-specification.md:192` |
| `merkleRoot` | `merkleRoot() returns (bytes32)` | `00-specification.md:193` |
| `claimAmount` | `claimAmount() returns (uint256)` | `00-specification.md:194` |
| `claimDeadline` | `claimDeadline() returns (uint256)` | `00-specification.md:195` |
| `estimateClaimGas` | `estimateClaimGas(Proof calldata proof, bytes32 nullifier, address recipient) returns (uint256)` | `00-specification.md:198-202` |

### API Endpoints
Check that all API endpoints match `docs/04-api-reference.md`:

| Endpoint | Method | Rate Limit | Notes |
|----------|--------|------------|-------|
| `/api/v1/submit-claim` | POST | 1/60s per nullifier | Main claim endpoint |
| `/api/v1/check-status/{nullifier}` | GET | 10/60s per nullifier | Status check |
| `/api/v1/merkle-root` | GET | 100/60s per IP | Get current root |
| `/api/v1/contract-info` | GET | 100/60s per IP | Contract addresses |
| `/api/v1/donate` | POST | 100/60s per IP | Fund relayer |
| `/api/v1/stats` | GET | 100/60s per IP | Statistics |
| `/api/v1/health` | GET | 100/60s per IP | Health check |
| `/api/v1/merkle-path/{address}` | GET | 60/60s per IP | Optional tree service |

### CLI Commands
Check that all CLI commands match `docs/04-api-reference.md:885-1072`:

| Command | Arguments | Output |
|---------|-----------|--------|
| `generate-proof` | `--private-key`, `--recipient`, `--merkle-tree`, `--output` | proof.json |
| `verify-proof` | `--proof`, `--merkle-root` | validation result |
| `submit` | `--proof`, `--relayer-url`, `--recipient`, `--wait` | transaction hash |
| `check-status` | `--nullifier`, `--relayer-url` | claim status |
| `download-tree` | `--source`, `--output`, `--format` | Merkle tree file |
| `config` | `show`, `set`, `reset` | configuration |

## Data Format Consistency

### 1. Address Format
- **Must**: `0x` prefix + 40 hex characters
- **Must**: Valid Ethereum address checksum (EIP-55 optional)
- **Example**: `0x1234567890123456789012345678901234567890`

### 2. Hash Format (32-byte)
- **Must**: `0x` prefix + 64 hex characters
- **Example**: `0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef`

### 3. Field Element Format
- **Primary**: Decimal string (no `0x` prefix)
- **Alternative**: Hex string with `0x` prefix (for developer convenience)
- **Must**: Less than BN128 prime modulus
- **Example (decimal)**: `"13862987149607610235678184535533251295074929736392939725598345555223684473689"`
- **Example (hex)**: `"0x1eab1f9d8c9a0e3a9a1b9c8d7a6b5c4d3e2f1a0b9c8d7e6f5a4b3c2d1e0f9a8b7c6d"`

### 4. Proof JSON Structure
- **a**: Array of 2 field elements (decimal strings)
- **b**: 2x2 array of field elements (decimal strings)
- **c**: Array of 2 field elements (decimal strings)
- **public_signals**: Array of 3 field elements `[merkle_root, recipient, nullifier]`
- **nullifier**: 32-byte hex string
- **recipient**: 20-byte hex string (address)
- **merkle_root**: 32-byte hex string
- **generated_at**: ISO 8601 timestamp

### 5. Merkle Tree Binary Format
- **Header**: 16 bytes (`"ZKPT"` magic + version + height + num_leaves + root_hash)
- **Leaf Data**: 20 bytes per address × 65,249,064 leaves
- **Total size**: 1.216 GiB (addresses only) or 4.00 GiB (full tree)

## Security Parameter Consistency

### Cryptographic Parameters
| Parameter | Value | Check in |
|-----------|-------|----------|
| ZK security level | 128 bits | All security docs |
| Hash security | 128 bits (Poseidon) | All specs |
| Trusted setup participants | ≥10 | `00-specification.md:391` |
| Audit requirements | 3+ independent firms | `00-specification.md:392` |

### Gas Estimates (Optimism)
| Operation | Gas Units | Notes |
|-----------|-----------|-------|
| Proof verification | 300,000 | Groth16 on BN128 |
| Storage + transfer | 200,000 | Token mint + nullifier storage |
| Base claim gas | 500,000 | Verification + storage/transfer |
| Relayer buffer | 200,000 | Gas price fluctuations |
| Estimated total | 700,000 | With buffer |
| Maximum cap | 1,000,000 | Absolute max with 100% buffer |
| **Gas price cap** | **0.1 gwei** | **Optimism-specific** |

### Rate Limits
| Limit | Value | Check in |
|-------|-------|----------|
| Per nullifier | 1 request per 60 seconds | All rate limit sections |
| Per IP | 100 requests per 60 seconds | All rate limit sections |
| Global | 1,000 requests per 60 seconds | All rate limit sections |
| Burst factor | 2x for 10 seconds | All rate limit sections |

## Implementation Verification Checklist

### Before Deployment
- [ ] All constants match `00-specification.md`
- [ ] Nullifier calculation uses `"zkp_airdrop_nullifier_v1"` domain separator
- [ ] Proof format matches JSON schema
- [ ] Field elements use decimal strings as primary format
- [ ] Address validation includes checksum (EIP-55)
- [ ] Merkle tree height is 26 with 65,249,064 leaves
- [ ] Poseidon parameters: width=3, full_rounds=8, partial_rounds=57, alpha=5
- [ ] BN128 curve used throughout
- [ ] Gas estimates match specification
- [ ] Rate limits implemented correctly
- [ ] Optimism chain ID is 10 (not 1)

### After Deployment
- [ ] Contract addresses match documentation
- [ ] API endpoints respond correctly
- [ ] CLI tool generates valid proofs
- [ ] Relayer submits claims successfully
- [ ] Rate limiting works as specified
- [ ] Privacy measures (random delays, gas randomization) active
- [ ] Monitoring alerts configured
- [ ] Documentation updated with actual deployment details

## Common Inconsistency Points

### 1. Gas Price Cap
**Incorrect**: 50 gwei (Ethereum mainnet typical)
**Correct**: 0.1 gwei (Optimism-specific)

### 2. Chain ID
**Incorrect**: 1 (Ethereum mainnet)
**Correct**: 10 (Optimism mainnet)

### 3. Field Element Format
**Incorrect**: Always hex with `0x` prefix
**Correct**: Decimal strings primary, hex alternative

### 4. Nullifier Calculation
**Incorrect**: `Poseidon(private_key)` without domain separator
**Correct**: `Poseidon("zkp_airdrop_nullifier_v1" || private_key || zeros)` with domain separator

### 5. Proof Array Sizes
**Incorrect**: 3-element arrays for Groth16 on BN128
**Correct**: 2-element arrays for `a` and `c`, 2x2 for `b`

### 6. Merkle Path Size
**Incorrect**: Variable or incorrect size
**Correct**: 832 bytes (26 × 32 bytes for siblings)

## Update Procedure

When updating specifications:

1. **Start with `00-specification.md`** - Update the single source of truth
2. **Run consistency check** using this document
3. **Update dependent documents** in order:
   - `01-overview.md` (high-level changes)
   - `02-technical-specification.md` (implementation details)
   - `04-api-reference.md` (API interfaces)
   - `05-security-privacy.md` (security implications)
   - `06-privacy-analysis.md` (privacy impact)
4. **Update implementation code** to match new specifications
5. **Run tests** to verify consistency
6. **Update this checklist** if needed

## Automated Verification

Consider implementing automated checks:

```bash
# Example verification script
#!/bin/bash

# Check constants match
grep -r "65,249,064" docs/ | wc -l
grep -r "gas price cap" docs/ | grep -E "0\.1 gwei|100000000"

# Check nullifier calculation
grep -r "zkp_airdrop_nullifier_v1" docs/ | wc -l

# Check field element format
grep -r "decimal strings" docs/00-specification.md
grep -r "hex strings" docs/00-specification.md | grep "alternative"
```

## Version Control

| Document | Current Version | Last Updated |
|----------|----------------|--------------|
| `00-specification.md` | 1.5.0 | 2026-02-07 |
| `01-overview.md` | 1.1.0 | 2026-02-07 |
| `02-technical-specification.md` | 1.1.0 | 2026-02-07 |
| `03-implementation-roadmap.md` | 1.0.0 | 2026-02-07 |
| `04-api-reference.md` | 1.1.0 | 2026-02-07 |
| `05-security-privacy.md` | 1.0.0 | 2026-02-07 |
| `06-privacy-analysis.md` | 1.0.0 | 2026-02-07 |
| `07-consistency-checklist.md` | 1.0.0 | 2026-02-07 |

**Rule**: When `00-specification.md` version changes, update all dependent document versions.