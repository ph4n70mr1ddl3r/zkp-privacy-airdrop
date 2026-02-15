# PLONK Proof Generation Guide

## Overview

The ZKP Privacy Airdrop uses PLONK (Permutations over Lagrange-bases for Oecumenical Noninteractive arguments of Knowledge) zero-knowledge proofs to enable privacy-preserving token claims. This document provides guidance on implementing PLONK proof generation.

## Current Status

**IMPORTANT**: The CLI tool currently does not include a fully functional PLONK proof generator. The `generate_plonk_proof()` function returns an error indicating that proving infrastructure needs to be set up.

### Why This is the Case

PLONK proof generation requires:

1. **Compiled Circuit**: The Circom circuit must be compiled to R1CS
2. **Proving Key**: A universal SRS (Structured Reference String) or circuit-specific proving key
3. **Witness Computation**: Computing witness values from private inputs
4. **Prover Library**: Integration with snarkJS or arkworks-plonk

These components are external to the Rust codebase and require separate setup.

## Recommended Approach

### Option 1: Use snarkJS (Recommended for Production)

The most battle-tested approach is to use snarkJS for PLONK proof generation:

```bash
# 1. Install snarkJS
npm install -g snarkjs

# 2. Compile the circuit (if not already done)
cd circuits
npm install
npx circom src/merkle_membership.circom --r1cs --wasm

# 3. Generate PLONK setup (universal SRS)
# For testing, use the Powers of Tau from snarkJS
npx snarkjs plonk setup merkle_membership.r1cs ptau_final.ptau plonk.zkey

# 4. Export verification key for Solidity
npx snarkjs zkey export solidityverifier plonk.zkey verifier.sol

# 5. Generate proof from inputs
npx snarkjs plonk prove merkle_membership.wasm plonk.zkey input.json public.json proof.json
```

### Option 2: Use arkworks-plonk (For Rust Integration)

For a pure Rust implementation:

```rust
// Add to Cargo.toml
[dependencies]
ark-plonk = "0.4"
ark-bn254 = "0.4"
ark-poly-commit = "0.4"

// Implementation example
use ark_plonk::prelude::*;

pub fn generate_plonk_proof_rust(
    proving_key: &ProvingKey<BN254>,
    private_inputs: &[Fr],
    public_inputs: &[Fr],
) -> Result<Proof<BN254>, Error> {
    // Create prover instance
    let prover = Prover::new(proving_key.clone());

    // Compute witness
    let mut cs = ConstraintSystem::new();
    let _witness = compute_witness(&mut cs, private_inputs, public_inputs);

    // Generate proof
    let proof = prover.prove(&cs)?;
    Ok(proof)
}
```

## Minimal Tree Builder Alternative

For quick testing, the tree-builder can generate proofs with pre-computed witness:

```bash
# Build Merkle tree and generate proof
tree-builder generate-proof \
  --private-key 0x... \
  --tree merkle_tree.bin \
  --output proof.json \
  --recipient 0x...
```

## Security Considerations

### Proving Key Security

- **Universal SRS**: Use the official Powers of Tau ceremony from Ethereum Foundation
- **Circuit-Specific**: Run trusted setup with proper participants
- **Verification**: Always verify proving key hashes

### Witness Security

- **Private Key**: Never log or serialize private keys
- **Memory**: Zeroize sensitive data after proof generation
- **Side Channels**: Use constant-time operations for key handling

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plonk_proof_structure() {
        let proof = InternalPlonkProof::minimal();
        assert_eq!(proof.a.len(), 2);
        assert_eq!(proof.b.len(), 2);
        assert_eq!(proof.c.len(), 2);
    }

    #[test]
    fn test_proof_verification() {
        // Test proof generation and verification
        let private_key = [0x12u8; 32];
        // ... generate and verify proof
    }
}
```

### Integration Tests

Test the full proof flow:

1. Generate proof from known inputs
2. Verify proof against known public inputs
3. Submit proof to relayer (if configured)
4. Verify on-chain claim succeeds

## Performance Expectations

Based on similar PLONK implementations:

| Operation | Expected Time | Notes |
|-----------|---------------|-------|
| Witness Computation | 1-5 seconds | Depends on circuit size |
| Proof Generation | 5-30 seconds | PLONK is slower than Groth16 |
| Proof Verification | 1-3 seconds | On-chain verification |
| Memory Usage | 100-500 MB | Depends on circuit complexity |

## Troubleshooting

### Common Issues

1. **"Circuit not found"**: Ensure R1CS file exists in correct path
2. **"Invalid witness"**: Check private key format and Merkle path
3. **"Proof verification failed"**: Verify public inputs match circuit expectations
4. **"Out of memory"**: Reduce batch size or increase RAM allocation

### Debug Mode

Enable detailed logging:

```bash
RUST_LOG=debug zkp-airdrop generate-proof --private-key ...
```

## References

- [PLONK Whitepaper](https://eprint.iacr.org/2019/953)
- [snarkJS Documentation](https://github.com/iden3/snarkjs)
- [arkworks PLONK](https://github.com/arkworks-rs/plonk)
- [Circom Documentation](https://docs.circom.io/)

## Next Steps

1. **Short Term**: Use tree-builder CLI for proof generation
2. **Medium Term**: Integrate snarkJS CLI with Rust wrapper
3. **Long Term**: Implement full arkworks-plonk integration

## Contributing

If you implement PLONK proof generation:

1. Add comprehensive unit tests
2. Document performance characteristics
3. Include security review findings
4. Update this document with implementation details
