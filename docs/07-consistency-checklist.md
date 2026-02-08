# Consistency Checklist

**Version**: 1.0.3
**Last Updated**: 2026-02-08

## Version History
| Version | Date | Changes | Author |
|---------|------|---------|--------|
| 1.0.4 | 2026-02-08 | Fixed cross-reference paths in multiple documents, updated roadmap and privacy analysis to v1.0.1 | Documentation Review |
| 1.0.3 | 2026-02-08 | Updated version control table to reflect all documents now referencing spec v1.5.1 | Documentation Review |
| 1.0.2 | 2026-02-08 | Updated to reflect docs/00-specification.md v1.5.1 (gas price terminology fix, removed dead references) | Documentation Review |
| 1.0.1 | 2026-02-08 | Fixed gas price randomization formula in automated verification script (random_integer(0, 5) not random(0, 6)) | Documentation Review |
| 1.0.0 | 2026-02-07 | Initial version with comprehensive cross-references | Documentation Review |  

This document serves as a verification guide to ensure all documentation and implementations remain consistent with the [Unified Specification v1.5.1](./00-specification.md), which is the single source of truth.

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
| Verify proof gas | 300,000 | `00-specification.md:689` |
| Storage/transfer gas | 200,000 | `00-specification.md:690` |
| Claim gas (total) | 500,000 | `00-specification.md:691` |
| Relayer buffer | 200,000 | `00-specification.md:692` |
| Estimated claim gas | 700,000 | `00-specification.md:693` |
| Max claim gas | 1,000,000 | `00-specification.md:694` |
| Optimism chain ID | 10 | 04-api-reference.md:175 |
| Gas price cap | 0.1 gwei | All docs |

## Critical Cross-Reference Checks

### 1. Nullifier Calculation
**Source**: `docs/00-specification.md:154-160` (lines 154-160)

Must match:
- `docs/00-specification.md:155` (line 155) - `nullifier = Poseidon("zkp_airdrop_nullifier_v1" || private_key || zeros)`
- `docs/02-technical-specification.md:101` (line 101) - `poseidon_hash("zkp_airdrop_nullifier_v1" || private_key || zeros)`
- `docs/04-api-reference.md:864-876` (lines 864-876) - `Poseidon("zkp_airdrop_nullifier_v1" || private_key || zeros)`
- `docs/05-security-privacy.md:239-248` (lines 239-248) - `poseidon_hash("zkp_airdrop_nullifier_v1" || private_key || zeros)`

**Formula**: `nullifier = Poseidon("zkp_airdrop_nullifier_v1" || private_key || zeros)`
where:
- `private_key` is 32-byte Ethereum private key
- `"zkp_airdrop_nullifier_v1"` is 23-byte ASCII domain separator
- `zeros` is 41 bytes of zeros (padding to reach 96 bytes total: 23 + 32 + 41)
- Input length must be exactly 96 bytes (23 + 32 + 41 = 96)

### 2. Proof Format
**Source**: `docs/00-specification.md:255-269` (lines 255-269)

Must match:
- `docs/04-api-reference.md:40-69` (lines 40-69) - JSON structure
- `docs/04-api-reference.md:770-861` (lines 770-861) - JSON schema

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
**Source**: `docs/00-specification.md:187-195`

Must match:
- Primary format: **decimal strings**
- Alternative format: hex strings with `0x` prefix
- Must be < BN128 prime modulus

### 4. Merkle Tree Structure
**Source**: `docs/00-specification.md:37-44` (lines 37-44)

Must match:
- Height: 26 levels
- Leaves: 65,249,064
- Hash: Poseidon(BN128 scalar field)
- Leaf: `Poseidon(address)` where address padded to 32 bytes
- Empty leaf: `Poseidon(32 zero bytes)`

### 5. Rate Limiting
**Source**: `docs/00-specification.md:415-424` (lines 415-424)

Must match:
- Per nullifier: 1 request per 60 seconds (all endpoints)
- Per IP: 100 requests per 60 seconds (all endpoints)
- Global: 1,000 requests per 60 seconds (all endpoints)
- Burst allowance: 2x limit for 10 seconds
- Endpoint-specific limits:
  - `POST /api/v1/submit-claim`: 1 request per 60 seconds per nullifier
  - `GET /api/v1/check-status/{nullifier}`: 10 requests per 60 seconds per nullifier
  - `GET /api/v1/merkle-path/{address}`: 60 requests per 60 seconds per IP
  - Other endpoints: 100 requests per 60 seconds per IP

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
| `authorizeRelayer` | `authorizeRelayer(address relayer)` | `00-specification.md:224` |
| `donate` | `donate() external payable` | `00-specification.md:225` |
| `withdraw` | `withdraw(uint256 amount) external` | `00-specification.md:226` |
| `isAuthorized` | `isAuthorized(address relayer) returns (bool)` | `00-specification.md:227` |
| `balanceOf` | `balanceOf(address relayer) returns (uint256)` | `00-specification.md:228` |
| `defaultRelayer` | `defaultRelayer() returns (address)` | `00-specification.md:229` |

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

### 9. Error Codes
Check that all error codes match `docs/04-api-reference.md:347-361`:

| Code | HTTP Status | User Message |
|------|-------------|--------------|
| `INVALID_PROOF` | 400 | "The provided proof is invalid. Please regenerate the proof with correct inputs." |
| `INVALID_NULLIFIER` | 400 | "Invalid nullifier format. Must be 64-character hex string with 0x prefix." |
| `ALREADY_CLAIMED` | 400 | "Tokens have already been claimed with this nullifier. Each qualified account can only claim once." |
| `RATE_LIMITED` | 429 | "Rate limit exceeded. Please try again in {retry_after} seconds." |
| `INSUFFICIENT_FUNDS` | 503 | "Relayer temporarily unavailable due to insufficient funds. Please try another relayer or submit directly to the contract." |
| `CONTRACT_ERROR` | 502 | "Contract interaction failed. Please try again later." |
| `NETWORK_ERROR` | 502 | "Ethereum network error. Please check your connection and try again." |
| `INTERNAL_ERROR` | 500 | "Internal server error. Please try again later." |
| `ADDRESS_NOT_FOUND` | 404 | "Address not found in qualified accounts list. Please verify your address is in the Merkle tree." |
| `INVALID_ADDRESS` | 400 | "Invalid Ethereum address. Must be 20-byte hex string with 0x prefix." |
| `INVALID_FIELD_ELEMENT` | 400 | "Invalid field element. Must be less than BN128 prime modulus." |
| `PROOF_EXPIRED` | 400 | "Proof expired. Please regenerate proof with current Merkle root." |
| `CLAIM_PERIOD_ENDED` | 400 | "Claim period has ended. Tokens can no longer be claimed." |

### 10. CLI Commands
Check that all CLI commands match `docs/04-api-reference.md:885-1072`:

| Command | Arguments | Output |
|---------|-----------|--------|
| `generate-proof` | `--private-key`, `--recipient`, `--merkle-tree`, `--output` | proof.json |
| `verify-proof` | `--proof`, `--merkle-root` | validation result |
| `submit` | `--proof`, `--relayer-url`, `--recipient`, `--wait` | transaction hash |
| `check-status` | `--nullifier`, `--relayer-url` | claim status |
| `download-tree` | `--source`, `--output`, `--format` | Merkle tree file |
| `config` | `show`, `set`, `reset` | configuration |

## 11. Data Format Consistency

### 11.1 Address Format
- **Must**: `0x` prefix + 40 hex characters
- **Must**: Valid Ethereum address checksum (EIP-55 optional)
- **Example**: `0x1234567890123456789012345678901234567890`

### 11.2 Hash Format (32-byte)
- **Must**: `0x` prefix + 64 hex characters
- **Example**: `0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef`

### 11.3 Field Element Format
- **Primary**: Decimal string (no `0x` prefix)
- **Alternative**: Hex string with `0x` prefix (for developer convenience)
- **Must**: Less than BN128 prime modulus: `0 <= x < 21888242871839275222246405745257275088548364400416034343698204186575808495617`
- **Example (decimal)**: `"13862987149607610235678184535533251295074929736392939725598345555223684473689"`
- **Example (hex)**: `"0x1eab1f9d8c9a0e3a9a1b9c8d7a6b5c4d3e2f1a0b9c8d7e6f5a4b3c2d1e0f9a8b7c6d"`

### 11.4 Proof JSON Structure
- **a**: Array of 2 field elements (decimal strings preferred, hex strings with 0x prefix alternative)
- **b**: 2x2 array of field elements (decimal strings preferred, hex strings with 0x prefix alternative)
- **c**: Array of 2 field elements (decimal strings preferred, hex strings with 0x prefix alternative)
- **public_signals**: Array of 3 field elements `[merkle_root, recipient, nullifier]` (decimal or hex strings)
- **nullifier**: 32-byte hex string with 0x prefix (64 hex characters)
- **recipient**: 20-byte hex string with 0x prefix (40 hex characters, EIP-55 checksum optional)
- **merkle_root**: 32-byte hex string with 0x prefix (64 hex characters)
- **generated_at**: ISO 8601 timestamp
- **Field Element Validation**: All field elements must be valid BN128 field elements: `0 <= x < p` where `p = 21888242871839275222246405745257275088548364400416034343698204186575808495617`

### 11.5 Merkle Tree Binary Format
**Source**: `00-specification.md:36-45`

- **Header**: 16 bytes (`"ZKPT"` magic + version + height + num_leaves + root_hash)
- **Leaf Data**: 20 bytes per address × 65,249,064 leaves
- **Total size**: 1.216 GiB (addresses only) or 4.00 GiB (full tree)
- **See storage requirements table in `00-specification.md:36-45` for complete details**

## 12. Security Parameter Consistency

### 12.1 Cryptographic Parameters
| Parameter | Value | Check in |
|-----------|-------|----------|
| ZK security level | 128 bits | All security docs |
| Hash security | 128 bits (Poseidon) | All specs |
| Trusted setup participants | ≥10 | `00-specification.md:391` |
| Audit requirements | 3+ independent firms | `00-specification.md:392` |

### 12.2 Gas Estimates (Optimism)
**Source**: `00-specification.md:405-410` and `00-specification.md:684-694` (single source of truth)

| Operation | Gas Units | Notes |
|-----------|-----------|-------|
| Proof verification | 300,000 | Groth16 on BN128 |
| Storage + transfer | 200,000 | Token mint + nullifier storage |
| Base claim gas | 500,000 | Verification + storage/transfer |
| Relayer buffer | 200,000 | Gas price fluctuations |
| Estimated total | 700,000 | With buffer |
| Maximum cap | 1,000,000 | Absolute max with 100% buffer |
| **Gas price cap** | **0.1 gwei** | **Optimism-specific** |

### 12.3 Rate Limits
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

## 12. Common Inconsistency Points

### 12.1 Gas Price Cap
**Incorrect**: 50 gwei (Ethereum mainnet typical)
**Correct**: 0.1 gwei (Optimism-specific)

### 12.2 Chain ID
**Incorrect**: 1 (Ethereum mainnet)
**Correct**: 10 (Optimism mainnet)

### 12.3 Field Element Format
**Incorrect**: Always hex with `0x` prefix
**Correct**: **Decimal strings primary (canonical format)**, hex with `0x` prefix alternative (developer convenience)
**Verification**: Check 00-specification.md:271 states "Primary Format: Decimal strings (not hex) - **This is the canonical format**"

### 12.4 Gas Price Randomization Formula
**Incorrect**: Inconsistent descriptions of random factor generation
**Correct**: `random_factor = random_integer(0, 5) / 100.0` where `random_integer(min, max)` returns integer in inclusive range [min, max] (0, 1, 2, 3, 4, or 5)
**Verification**: Check 00-specification.md:498-499 for exact implementation

### 12.5 Nullifier Calculation
**Incorrect**: `Poseidon(private_key)` without domain separator
**Correct**: `Poseidon("zkp_airdrop_nullifier_v1" || private_key || zeros)` with domain separator and 41 bytes zeros
**Note**: Input must be exactly 96 bytes: 23 bytes domain separator + 32 bytes private key + 41 bytes zeros

### 12.6 Proof Array Sizes
**Incorrect**: 3-element arrays for Groth16 on BN128
**Correct**: 2-element arrays for `a` and `c`, 2x2 for `b`

### 12.7 Precomputed Proofs Storage
**Correct**: 868 bytes per leaf (~50.8 GiB total) 
**Calculation**: 832 bytes (26 × 32-byte siblings) + 32 bytes (leaf hash) + 4 bytes (26 bits packed into 4 bytes for path indices) = 868 bytes total
**Total**: 65,249,064 leaves × 868 bytes = ~50.8 GiB
**Verification**: Check 00-specification.md:56 for exact storage calculation

### 12.8 Terminology Consistency
**Inconsistent**: Mixed use of "claimant", "user", "account holder"
**Correct**: Standardize to:
- **Claimant**: Person/organization claiming tokens (has private key)
- **Recipient**: Address receiving tokens (may differ from claimant's original address)
- **User**: General term when specificity not needed
**Verification**: Check all documents use consistent terminology

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
# Check gas price randomization formula
grep -r "random_factor" docs/ | grep -E "0-5%|\[0.00, 0.05\]|inclusive"
# Verify: random_factor ∈ [0.00, 0.05] inclusive, generated as random_integer(0, 5) / 100.0

# Check nullifier calculation
grep -r "zkp_airdrop_nullifier_v1" docs/ | wc -l
grep -r "Poseidon.*zkp_airdrop_nullifier_v1.*private_key.*zeros" docs/ | wc -l

# Check field element format
grep -r "decimal strings" docs/00-specification.md
grep -r "hex strings" docs/00-specification.md | grep "alternative"
```

## Version Control

| Document | Current Version | Last Updated | Based On |
|----------|----------------|--------------|----------|
| `00-specification.md` | 1.5.1 | 2026-02-08 | - (source of truth) |
| `01-overview.md` | 1.1.1 | 2026-02-08 | `00-specification.md` v1.5.1 |
| `02-technical-specification.md` | 1.1.1 | 2026-02-08 | `00-specification.md` v1.5.1 |
| `03-implementation-roadmap.md` | 1.0.1 | 2026-02-08 | `02-technical-specification.md` v1.1.1 |
| `04-api-reference.md` | 1.1.1 | 2026-02-08 | `00-specification.md` v1.5.1 |
| `05-security-privacy.md` | 1.0.1 | 2026-02-08 | `00-specification.md` v1.5.1 + `02-technical-specification.md` v1.1.1 |
| `06-privacy-analysis.md` | 1.0.1 | 2026-02-08 | `05-security-privacy.md` v1.0.1 |
| `07-consistency-checklist.md` | 1.0.4 | 2026-02-08 | All documents |

**Rule**: When `00-specification.md` version changes, update all dependent document versions and update this table.