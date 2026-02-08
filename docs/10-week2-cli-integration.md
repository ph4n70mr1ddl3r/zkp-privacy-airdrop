# Week 2: CLI & Integration Implementation Guide

**Status**: Ready to Start
**Duration**: ~1 week
**Focus**: Implement PLONK proof generation and submission in CLI

---

## Overview

This week will implement PLONK proof generation in the CLI tool and update the relayer to accept PLONK proofs. By the end of the week, users should be able to:

1. Generate PLONK proofs offline
2. Submit PLONK proofs to relayer
3. Verify PLONK proofs locally
4. Check claim status with PLONK nullifiers

---

## Tasks Breakdown

### Task 1: PLONK Proof Generation Module (2-3 hours)

**File**: `cli/src/plonk_prover.rs` (NEW)

**Requirements**:
- Use proving key from Week 1 (`merkle_membership_plonk.zkey`)
- Use circuit WASM (`merkle_membership.wasm`)
- Generate PLONK proofs with 8 field elements
- Handle private key securely (zeroize after use)

**Input**:
- Private key (32 bytes, secp256k1)
- Recipient address (20 bytes, padded)
- Merkle tree (or Merkle path from API)
- Nullifier (computed automatically)

**Output**:
- PLONK proof (8 field elements)
- Public signals (merkle_root, recipient, nullifier)
- Proof JSON file

**Implementation Notes**:
```rust
// Pseudocode for PLONK proof generation
pub async fn generate_plonk_proof(
    private_key: &[u8; 32],
    recipient: Address,
    merkle_tree: &MerkleTree,
    proving_key: &ProvingKey,
    circuit_wasm: &[u8],
) -> Result<PLONKProofData> {
    // 1. Derive address from private key
    let address = derive_address(private_key)?;
    
    // 2. Compute leaf hash (Poseidon)
    let leaf = poseidon_hash(&address.as_bytes());
    
    // 3. Get Merkle path
    let (path, indices) = merkle_tree.get_path(&leaf)?;
    
    // 4. Compute nullifier
    let nullifier = compute_nullifier(private_key);
    
    // 5. Prepare public inputs
    let public_inputs = PublicInputs {
        merkle_root: merkle_tree.root,
        recipient: address_to_field(&recipient),
        nullifier: nullifier_to_field(&nullifier),
    };
    
    // 6. Prepare private inputs
    let private_inputs = PrivateInputs {
        private_key: private_key.to_vec(),
        merkle_path: path,
        merkle_path_indices: indices,
    };
    
    // 7. Generate PLONK proof
    let proof = plonk_prove(
        circuit_wasm,
        proving_key,
        private_inputs,
        public_inputs,
    )?;
    
    // 8. Zeroize sensitive data
    private_key.zeroize();
    
    Ok(PLONKProofData {
        proof,
        public_inputs,
        nullifier,
        recipient,
        merkle_root: merkle_tree.root,
        generated_at: Utc::now().to_rfc3339(),
    })
}
```

**Error Handling**:
- Invalid private key (not secp256k1 scalar)
- Private key not in Merkle tree
- Failed proof generation
- Merkle path not found

---

### Task 2: Update CLI Main Command (1-2 hours)

**File**: `cli/src/main.rs` (MODIFY)

**Changes**:
1. Import PLONK proof generation module
2. Add PLONK-specific flags
3. Update `generate-proof` command to use PLONK

**New Flags**:
```bash
--proof-system plonk  # Use PLONK (default: groth16)
--proving-key <path>   # Path to PLONK proving key
--circuit-wasm <path>    # Path to circuit WASM
```

**Updated Command**:
```bash
zkp-airdrop generate-proof \
  --proof-system plonk \
  --proving-key circuits/merkle_membership_plonk.zkey \
  --circuit-wasm circuits/merkle_membership.wasm \
  --private-key $PRIVATE_KEY \
  --recipient $RECIPIENT_ADDRESS \
  --merkle-tree merkle_tree.bin \
  --output proof_plonk.json
```

**Backward Compatibility**:
- Default to Groth16 for existing users
- Add `--proof-system` flag to switch
- Support both proof systems in CLI

---

### Task 3: Update CLI Submit Command (2 hours)

**File**: `cli/src/commands/submit.rs` (MODIFY)

**Changes**:
1. Accept PLONK proof format
2. Validate PLONK proof structure
3. Update API request format
4. Better error messages for PLONK

**PLONK Validation**:
```rust
fn validate_plonk_proof(proof: &PLONKProof) -> Result<()> {
    // Check proof has 8+ elements
    if proof.A.len() != 2 {
        return Err(anyhow!("PLONK proof.A must have 2 elements"));
    }
    if proof.B.len() != 2 || proof.B.iter().any(|row| row.len() != 2) {
        return Err(anyhow!("PLONK proof.B must have 2x2 elements"));
    }
    if proof.C.len() != 2 {
        return Err(anyhow!("PLONK proof.C must have 2 elements"));
    }
    
    // Check Z, T1, T2, T3 elements exist (for full PLONK)
    if proof.Z.is_empty() || proof.T1.is_empty() {
        return Err(anyhow!("PLONK proof missing Z or T1 element"));
    }
    
    // Validate field elements are in range
    for element in proof.all_elements() {
        let val = parse_field_element(element)?;
        if val >= PRIME_Q {
            return Err(anyhow!("Field element out of range"));
        }
    }
    
    Ok(())
}
```

**Error Messages**:
```
Invalid PLONK proof: proof.A must have 2 elements
Invalid PLONK proof: proof.B must have 2x2 elements
Invalid PLONK proof: field element out of range
```

---

### Task 4: Update Relayer Proof Validation (2 hours)

**File**: `relayer/src/handlers.rs` (MODIFY)

**Changes**:
1. Accept PLONK proof format in API
2. Validate PLONK proof structure
3. Update contract call for PLONK verifier
4. Update gas estimates for PLONK

**API Changes**:
```rust
// Updated submit_claim handler to support PLONK
pub async fn submit_claim(
    req: HttpRequest,
    state: web::Data<AppState>,
    claim: web::Json<SubmitClaimRequest>,  // Proof enum (Groth16 | PLONK)
) -> impl Responder {
    info!("Received {} claim from nullifier: {}", 
        claim.proof.type_name(), claim.nullifier);
    
    // Detect proof system
    match &claim.proof {
        Proof::Groth16(_) => {
            // Use Groth16 verifier
            verify_groth16_proof(&claim, &state).await
        }
        Proof::PLONK(ref plonk_proof) => {
            // Use PLONK verifier
            verify_plonk_proof(plonk_proof, &claim, &state).await
        }
    }
}

async fn verify_plonk_proof(
    proof: &PLONKProof,
    claim: &SubmitClaimRequest,
    state: &AppState,
) -> Result<SubmitClaimResponse> {
    // 1. Validate proof structure
    validate_plonk_proof_structure(proof)?;
    
    // 2. Verify proof structure (not actual proof, just format)
    // Real verification happens on-chain
    
    // 3. Check nullifier
    if state.is_nullifier_used(&claim.nullifier).await {
        return Ok(SubmitClaimResponse {
            success: false,
            error: "Already claimed".to_string(),
            code: Some("ALREADY_CLAIMED".to_string()),
            ..Default::default()
        });
    }
    
    // 4. Check balance
    if !state.has_sufficient_balance().await {
        return Ok(SubmitClaimResponse {
            success: false,
            error: "Insufficient funds".to_string(),
            code: Some("INSUFFICIENT_FUNDS".to_string()),
            ..Default::default()
        });
    }
    
    // 5. Submit to PLONK verifier contract
    let tx_hash = state.submit_plonk_claim(
        proof,
        claim.recipient.clone(),
        claim.nullifier.clone(),
    ).await?;
    
    Ok(SubmitClaimResponse {
        success: true,
        tx_hash: Some(tx_hash),
        ..Default::default()
    })
}
```

---

### Task 5: Integration Testing (2-3 hours)

**File**: `tests/test_plonk_integration.py` (NEW)

**Test Cases**:
1. Generate PLONK proof for valid address
2. Submit PLONK proof to relayer
3. Verify claim status on-chain
4. Test error cases (invalid proof, already claimed)
5. Test proof size limits
6. Test gas costs

**Test Script**:
```python
import pytest
import requests

def test_plonk_proof_generation():
    """Test PLONK proof generation"""
    # Generate proof for valid address
    result = subprocess.run([
        'zkp-airdrop', 'generate-proof',
        '--proof-system', 'plonk',
        '--private-key', '0x1234...',
        '--recipient', '0x5678...',
        '--output', 'test_proof.json'
    ])
    
    assert result.returncode == 0
    assert os.path.exists('test_proof.json')
    
    # Verify proof format
    with open('test_proof.json') as f:
        proof = json.load(f)
        assert 'proof' in proof
        assert len(proof['proof']['A']) == 2

def test_plonk_proof_submission():
    """Test PLONK proof submission to relayer"""
    # Submit PLONK proof
    response = requests.post(
        'http://localhost:8080/api/v1/submit-claim',
        json={
            'proof': {
                'PLONK': generate_plonk_proof()
            },
            'nullifier': '0xabc...',
            'recipient': '0x1234...',
            'merkle_root': '0xdef...'
        }
    )
    
    assert response.status_code == 200
    data = response.json()
    assert data['success'] == True
    assert 'tx_hash' in data

def test_plonk_proof_validation():
    """Test PLONK proof validation"""
    # Test invalid PLONK proof (wrong number of elements)
    response = requests.post(
        'http://localhost:8080/api/v1/submit-claim',
        json={'proof': 'invalid_plonk_proof'}
    )
    
    assert response.status_code == 400
    data = response.json()
    assert data['code'] == 'INVALID_PROOF'
```

---

## Implementation Order

### Day 1: Foundation
- [ ] Create `cli/src/plonk_prover.rs` module
- [ ] Implement PLONK proof generation
- [ ] Test with sample inputs
- [ ] Update `cli/src/types_plonk.rs` with new types

### Day 2: CLI Integration
- [ ] Update `cli/src/main.rs` for PLONK flags
- [ ] Modify `generate-proof` command
- [ ] Update `verify-proof` command
- [ ] Test CLI with PLONK proofs

### Day 3: Submit & Validation
- [ ] Update `cli/src/commands/submit.rs` for PLONK
- [ ] Update `relayer/src/handlers.rs` for PLONK
- [ ] Update `relayer/src/state.rs` for PLONK contract calls
- [ ] Test relayer with PLONK proofs

### Day 4: Integration
- [ ] Create integration test suite
- [ ] Test end-to-end flow
- [ ] Measure actual proof generation time
- [ ] Measure actual gas costs

### Day 5: Polish
- [ ] Error handling improvements
- [ ] Documentation updates
- [ ] Performance optimization
- [ ] Bug fixes

---

## Dependencies

### CLI Dependencies (from Phase 1)
```toml
ark-plonk = "0.4"  # Already added
ark-poly = "0.4"  # For PLONK polynomial operations
```

### Testing Dependencies
```toml
[dev-dependencies]
pytest = "^7.4"
requests = "^2.31"
web3 = "^6.11"
```

---

## Success Criteria

### Day 1 Complete When:
- [ ] PLONK proof generation module created
- [ ] Module compiles without errors
- [ ] Sample proof generates successfully
- [ ] Proof has correct structure (8 field elements)

### Day 2 Complete When:
- [ ] CLI accepts `--proof-system plonk` flag
- [ ] CLI generates PLONK proofs
- [ ] CLI verifies PLONK proofs
- [ ] All CLI commands work with PLONK

### Day 3 Complete When:
- [ ] Relayer accepts PLONK proofs
- [ ] Relayer validates PLONK structure
- [ ] Relayer submits to PLONK verifier
- [ ] API returns appropriate errors for invalid PLONK proofs

### Day 4 Complete When:
- [ ] Integration tests pass
- [ ] End-to-end flow works
- [ ] Gas costs measured (target: ~1.3M)
- [ ] Proof generation time measured (target: <10s)

### Day 5 Complete When:
- [ ] Error handling is robust
- [ ] Performance is acceptable
- [ ] Documentation is updated
- [ ] Code review passes

---

## Testing Checklist

### Unit Tests
- [ ] PLONK proof generation tests
- [ ] Field element validation tests
- [ ] Nullifier computation tests
- [ ] Merkle path lookup tests

### Integration Tests
- [ ] CLI → Relayer flow
- [ ] Relayer → Contract flow
- [ ] Proof verification flow
- [ ] Error handling tests

### Performance Tests
- [ ] Proof generation time: <10s (95th percentile)
- [ ] API response time: <200ms (95th percentile)
- [ ] Contract gas cost: ~1.3M

---

## Deliverables

### Code Files
1. `cli/src/plonk_prover.rs` - PLONK proof generation
2. `cli/src/main.rs` (modified) - Updated CLI
3. `cli/src/commands/submit.rs` (modified) - Updated submit
4. `cli/src/types_plonk.rs` (modified) - Updated types
5. `relayer/src/handlers.rs` (modified) - PLONK support
6. `relayer/src/state.rs` (modified) - PLONK contract calls

### Test Files
7. `tests/test_plonk_integration.py` - Integration tests
8. `tests/test_plonk_proof.json` - Sample PLONK proof

### Documentation
9. `docs/10-week2-cli-integration.md` - This guide
10. `QUICKSTART.md` (updated) - PLONK CLI instructions

---

## Known Issues & Limitations

### Current Limitations
1. **Placeholder PLONK Verifier**: Actual PLONK verification requires full verification key
2. **No Real Proof Generation**: Using mock until actual circuit is compiled
3. **Gas Cost Estimates**: Based on theoretical calculations, not measured

### Workarounds
1. Use placeholder verifier for testing
2. Update with real verification key after Week 1 completion
3. Measure actual costs during Week 3

### Future Improvements
1. Generate real PLONK verification key from verification key JSON
2. Optimize proof generation time (target: <5s)
3. Batch proof generation for multiple claims
4. Add caching for Merkle paths

---

## Time Estimates

| Task | Hours | Days |
|-------|--------|-------|
| Task 1: PLONK Proof Generation | 2-3h | Day 1 |
| Task 2: CLI Main Command | 1-2h | Day 2 |
| Task 3: CLI Submit Command | 2h | Day 2-3 |
| Task 4: Relayer Validation | 2h | Day 3 |
| Task 5: Integration Testing | 2-3h | Day 4 |
| Buffer & Polish | 4h | Day 5 |
| **Total** | **13-18h** | **~3 days** |

---

## Risk Mitigation

### Risk 1: PLONK Complexity Higher Than Expected
**Impact**: Could take longer than 3 days
**Mitigation**: Start with simple PLONK, optimize later

### Risk 2: Gas Costs Exceed Estimates
**Impact**: Users pay more than $3.90 per claim
**Mitigation**: Monitor closely, optimize if needed

### Risk 3: Integration Issues
**Impact**: CLI and relayer don't work together
**Mitigation**: Extensive testing, have rollback plan

---

## Next Steps (Week 3)

After Week 2 completion:
1. Deploy PLONK contracts to testnet
2. Run full integration tests
3. Measure actual gas costs
4. Performance benchmarking
5. User acceptance testing
6. Bug fixes and optimization

---

**Document Version**: 1.0
**Status**: Ready to Start
**Next Action**: Begin Task 1: PLONK Proof Generation Module
