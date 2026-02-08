# PLONK Migration - Critical Documentation Updates

**Status**: Phase 1 Complete ✅
**Date**: 2026-02-08
**Migration From**: Groth16 (circuit-specific trusted setup)
**Migration To**: PLONK (universal Powers of Tau - no ceremony needed)

## Executive Summary

We have completed **Phase 1** of the PLONK migration: updating critical foundation documentation. The key change is that **NO TRUSTED SETUP CEREMONY IS REQUIRED** - we will use the existing Perpetual Powers of Tau setup (1000+ participants).

## Phase 1: Completed Updates ✅

### 1. docs/02-technical-specification.md

#### Section 1.2: Proof System Selection ✅
- Changed from Groth16 to PLONK
- Added rationale for using existing Powers of Tau
- Added trade-off comparison table
- Documented proof size, verification time, and gas costs

#### Section 1.3: Trusted Setup ✅
- Changed to document that NO ceremony is required
- Added Powers of Tau reference and download instructions
- Explained security guarantees
- Removed circuit-specific ceremony details

#### Section 1.5: Proof Verification Flow ✅
- Updated to mention PLONK verification instead of Groth16

#### Section 4.1: Dependencies ✅
- Changed `ark-groth16` to `ark-plonk`
- Updated comments to reflect PLONK

### 2. docs/00-specification.md

#### Storage Requirements Table ✅
- Changed proof size from ~200 bytes (Groth16) to ~500 bytes (PLONK)
- Updated complete proof package size from ~1,032 bytes to ~1,332 bytes

#### Smart Contracts Section ✅
- Updated proof verification gas: 300K → 900K (3x increase)
- Updated total claim gas: 500K → 1,100K
- Updated estimated claim gas: 700K → 1,300K
- Updated max claim gas: 1M → 1.5M
- Added note about PLONK trade-offs

#### Section 9.1: Cryptographic Standards ✅
- Changed ZK Proofs from Groth16 to PLONK
- Added trusted setup note about Powers of Tau

#### Section 10.3: Gas Estimates ✅
- Updated all gas constants for PLONK verification
- Added trade-off analysis table
- Updated cost comparison
- Added comprehensive comparison of PLONK vs Groth16

#### Glossary Sections ✅
- Updated Proof Package description
- Updated gas estimates in section 11.5
- Changed BN128 curve description to mention PLONK

## Key Changes Summary

### Proof System
| Aspect | Groth16 (OLD) | PLONK (NEW) |
|---------|----------------|----------------|
| Trusted Setup | Circuit-specific ceremony | None (uses existing Powers of Tau) |
| Proof Size | ~200 bytes | ~500 bytes |
| Verification Gas | ~300,000 | ~900,000 (3x) |
| Setup Complexity | High (10+ participants) | Low (download existing setup) |
| Flexibility | Low (new ceremony for changes) | High (no new setup needed) |

### Economic Impact
| Metric | Groth16 | PLONK | Impact |
|--------|----------|---------|---------|
| Proof Size | 200 bytes | 500 bytes | +2.5x larger |
| Verification Gas | 300K | 900K | +3x higher cost |
| Total Claim Gas | 700K | 1.1M | +1.57x higher cost |
| Optimism Cost @ 0.001 gwei | $2.10 | $3.30 | +57% more expensive |
| Ethereum L1 Cost @ 50 gwei | $105 | $165 | +57% more expensive |

### Security Impact
| Aspect | Groth16 | PLONK |
|---------|----------|---------|
| Trusted Setup | Required | Not required (uses existing setup) |
| Ceremony Participants | 10+ | 1000+ (already completed) |
| Security Assumption | 1 honest participant | 1 honest participant (same) |
| Toxic Waste | Must be destroyed | Already destroyed |

## Phase 2: Implementation (TODO) ⚠️

### High-Priority Implementation Tasks

#### 1. Circuit Compilation
**Status**: ❌ Not Started
**Effort**: 2-4 hours
**Tasks**:
- [ ] Update circuit compilation to use PLONK instead of Groth16
- [ ] Download Powers of Tau setup files
- [ ] Generate PLONK proving key
- [ ] Generate PLONK verification key
- [ ] Test proof generation with sample inputs

**Commands** (to be implemented):
```bash
# Download Powers of Tau
npx snarkjs powersoftau download bn128 -p 22688242871839275222246405745257275088548364400416034343698204186575808495617 -n 16

# Compile circuit with PLONK
circom src/merkle_membership.circom --r1cs --wasm --sym

# Generate PLONK setup
snarkjs plonk setup merkle_membership.r1cs pot.ptau powersOfTau28_hez_final_18.ptau

# Full prove command
snarkjs plonk fullprove merkle_member.wasm merkle_member.zkey input.json public.json
```

#### 2. Verifier Contract
**Status**: ❌ Not Started
**Effort**: 4-6 hours
**Tasks**:
- [ ] Generate PLONK verifier Solidity code
- [ ] Update `contracts/src/Verifier.sol` for PLONK
- [ ] Test verifier with test proofs
- [ ] Update gas estimates based on actual verification costs

**Expected Changes**:
```solidity
// Groth16 verifier (OLD)
contract Verifier {
    function verifyProof(
        uint[2] calldata _pA,
        uint[2][2] calldata _pB,
        uint[2] calldata _pC,
        uint[3] calldata _pubSignals
    ) public view returns (bool);

    // Pairing operations...
}

// PLONK verifier (NEW)
contract Verifier {
    function verifyProof(
        uint256[8] calldata _proof,
        uint256[8] calldata _instances,
        uint256[9] calldata _verificationKey
    ) public view returns (bool);

    // PLONK verification logic...
}
```

#### 3. CLI Updates
**Status**: ❌ Not Started
**Effort**: 3-5 hours
**Tasks**:
- [ ] Update `cli/Cargo.toml` dependencies
  - Change `ark-groth16 = "0.4"` to `ark-plonk = "0.4"`
- [ ] Update proof generation in `cli/src/crypto.rs`
- [ ] Update proof serialization in `cli/src/types.rs`
- [ ] Test proof generation and verification

#### 4. Relayer Updates
**Status**: ❌ Not Started
**Effort**: 2-3 hours
**Tasks**:
- [ ] Update proof validation in `relayer/src/handlers.rs`
- [ ] Update proof structure in `relayer/src/types.rs`
- [ ] Test API endpoints with PLONK proofs

#### 5. API Documentation
**Status**: ⚠️ Partial (format not yet updated)
**Effort**: 1-2 hours
**Tasks**:
- [ ] Update `docs/04-api-reference.md` proof format section
- [ ] Add PLONK proof structure documentation
- [ ] Update all code examples

**Expected PLONK Proof Format**:
```json
{
  "proof": {
    "A": ["<field_element>", "<field_element>"],
    "B": [["<field_element>", "<field_element>"], ["<field_element>", "<field_element>"]],
    "C": ["<field_element>", "<field_element>"],
    "Z": ["<field_element>", "<field_element>", "<field_element>"],
    "T1": ["<field_element>", "<field_element>"],
    "T2": ["<field_element>", "<field_element>"],
    "T3": ["<field_element>", "<field_element>"],
    "WXi": [["<field_element>", "<field_element>", ...]
  },
  "public_signals": ["<merkle_root>", "<recipient>", "<nullifier>"],
  "nullifier": "0x...",
  "recipient": "0x...",
  "merkle_root": "0x...",
  "generated_at": "2024-01-15T10:30:00Z"
}
```

#### 6. Test Suite Updates
**Status**: ❌ Not Started
**Effort**: 2-3 hours
**Tasks**:
- [ ] Update test proofs to use PLONK format
- [ ] Update contract tests for PLONK verifier
- [ ] Update integration tests
- [ ] Verify all tests pass

#### 7. Documentation Final Updates
**Status**: ⚠️ In Progress
**Effort**: 2-4 hours
**Tasks**:
- [x] Update critical foundation specs (Phase 1) ✅
- [ ] Update API reference with actual PLONK format
- [ ] Update QUICKSTART.md with PLONK commands
- [ ] Update README.md with gas cost information
- [ ] Add PLONK migration notes to all docs
- [ ] Create PLONK migration guide

## Phase 3: Final Polish (TODO) 📝

### Documentation Review
- [ ] Review all docs for consistency
- [ ] Verify all examples work
- [ ] Update all code samples
- [ ] Add PLONK troubleshooting section

### Deployment Preparation
- [ ] Measure actual gas costs on testnet
- [ ] Update gas estimates based on real measurements
- [ ] Verify proof size with actual circuit
- [ ] Test complete end-to-end flow

## Questions & Decisions Needed

### 1. Powers of Tau File Management
**Question**: Where should Powers of Tau files be stored?
**Options**:
- A. Include in repo (large files, ~100MB+)
- B. Download at build time (requires internet)
- C. Store in separate repo/data bucket
- D. Use CDN (fastest, but external dependency)

**Recommendation**: Option C (separate data bucket)

### 2. PLONK Parameter Choices
**Question**: What PLONK parameters to use?
**Options**:
- A. Default snarkjs PLONK (easiest, but less optimized)
- B. Optimized PLONK parameters (better performance, more complexity)
- C. TurboPLONK (faster proofs, more complex circuit)

**Recommendation**: Start with Option A, optimize later if needed

### 3. Gas Optimization
**Question**: Is 1.1M gas acceptable?
**Options**:
- A. Accept as-is (simpler implementation)
- B. Optimize verifier (reduce to ~700K)
- C. Consider alternative proof systems

**Recommendation**: Start with Option A, consider B if gas costs are prohibitive

## Implementation Timeline Estimate

### Phase 2 (Implementation): 2-3 weeks
- Circuit compilation: 2-3 days
- Verifier contract: 3-4 days
- CLI updates: 2-3 days
- Relayer updates: 1-2 days
- API docs: 1 day
- Testing: 2-3 days
- Buffer: 2-3 days

### Phase 3 (Final Polish): 1 week
- Documentation review: 2-3 days
- Deployment testing: 2-3 days
- Bug fixes: 2-3 days

### Total: 3-4 weeks

## Success Criteria

### Phase 2 Complete When:
- ✅ Circuit compiles with PLONK
- ✅ PLONK verifier contract deployed and tested
- ✅ CLI generates valid PLONK proofs
- ✅ Relayer accepts and verifies PLONK proofs
- ✅ All tests pass with PLONK proofs
- ✅ API documentation updated with PLONK format

### Phase 3 Complete When:
- ✅ All documentation is consistent
- ✅ Gas costs are accurately measured
- ✅ End-to-end testing passes
- ✅ Ready for testnet deployment

## Risks & Mitigations

### Risk 1: Higher Gas Costs
**Impact**: Users pay more for claims
**Mitigation**:
- Accept higher costs in exchange for no ceremony
- Monitor testnet performance
- Consider optimizations if costs are prohibitive

### Risk 2: PLONK Implementation Complexity
**Impact**: Development takes longer than expected
**Mitigation**:
- Start with simple snarkjs PLONK
- Use existing libraries (arkworks, snarkjs)
- Incremental approach (prove first, optimize later)

### Risk 3: Tooling Limitations
**Impact**: PLONK tooling less mature than Groth16
**Mitigation**:
- Test all tooling early
- Have fallback plan (Groth16 still possible)
- Contribute to open-source projects if issues found

## Next Steps

1. **Review this document** and approve approach
2. **Decide on Powers of Tau file management**
3. **Begin Phase 2 implementation** starting with circuit compilation
4. **Update documentation in parallel** with implementation
5. **Test thoroughly** before testnet deployment

## References

- Perpetual Powers of Tau: https://www.powersoftau.eth/
- PLONK Protocol: https://eprint.iacr.org/2019/953
- Arkworks PLONK: https://github.com/arkworks-rs/poly-commitment
- Snarkjs PLONK: https://github.com/iden3/snarkjs

---

**Document Version**: 1.0
**Last Updated**: 2026-02-08
**Status**: Phase 1 Complete, Phase 2 Pending
**Next Action**: Begin circuit compilation with PLONK
