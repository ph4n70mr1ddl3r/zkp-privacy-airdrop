# Nullifier Design Fix Summary

## Problem Identified

**Original Design**: `nullifier = Poseidon(private_key || recipient || padding)`
- **Issue**: Same private key with different recipients → different nullifiers → allows unlimited claims

**First Attempted Fix**: `nullifier = Poseidon(address)` where address derived from private key
- **Issue**: Anyone with address list can compute all nullifiers → complete loss of privacy

## Final Solution

**Correct Design**: `nullifier = Poseidon(private_key)`
- **Deterministic**: Same private key → same nullifier
- **Private**: Requires private key to compute (not derivable from address)
- **Prevents double claims**: Each private key can only claim once
- **Preserves privacy**: Cannot link nullifier to address without private key

## Technical Details

### Nullifier Computation
```
nullifier = Poseidon(private_key)
```
Where:
- `private_key` is 32-byte Ethereum private key (secp256k1 scalar)
- Poseidon hash with width=3 requires 96 bytes input
- Padding options:
  1. Zero padding: `private_key || [0x00]*64`
  2. Domain separation: `"zkp_airdrop_nullifier" || private_key || zeros`
  3. Field element encoding: Encode 32 bytes into 1-2 field elements

### ZK Circuit Constraints
1. Validate `private_key < SECP256K1_ORDER`
2. Compute `address = keccak256(secp256k1_mul(G, private_key)[1:])[12:32]`
3. Compute `leaf = Poseidon(address)`
4. Verify Merkle proof for `leaf`
5. Compute `nullifier = Poseidon(private_key)`
6. Verify `nullifier` matches public input

### Smart Contract Logic
```solidity
function claim(Proof calldata proof, bytes32 nullifier, address recipient) external {
    require(!nullifiers[nullifier], "Already claimed");
    require(verifier.verifyProof(proof, [merkleRoot, recipient, nullifier]), "Invalid proof");
    nullifiers[nullifier] = true;
    token.transfer(recipient, claimAmount);
}
```

## Privacy Guarantees Maintained

1. **Identity Privacy**: Original address never revealed
2. **Single Claim Guarantee**: Each private key can only claim once
3. **Unlinkability**: Cannot determine which nullifier corresponds to which address
4. **Recipient Flexibility**: User can choose any recipient for their single claim

## Security Properties

1. **Collision Resistance**: Different private keys → different nullifiers (Poseidon hash property)
2. **Preimage Resistance**: Cannot recover private key from nullifier
3. **Double-Spend Prevention**: Contract tracks nullifiers, rejects duplicates
4. **Privacy Preservation**: Nullifier doesn't reveal address information

## Implementation Consistency

All components must use the same nullifier computation:
1. **ZK Circuit**: `nullifier = Poseidon(private_key)`
2. **CLI Tool**: `nullifier = Poseidon(private_key)`
3. **Contract Verification**: Checks `nullifiers[nullifier] == false`

## Example

```
Alice has private_key_A in qualified list
↓
Compute nullifier_A = Poseidon(private_key_A)
↓
Alice claims to recipient_X → nullifier_A stored on-chain
↓
Alice tries to claim to recipient_Y → nullifier_A already used → REJECTED
↓
Bob (private_key_B) also in qualified list
↓
Compute nullifier_B = Poseidon(private_key_B) ≠ nullifier_A
↓
Bob claims to recipient_Z → nullifier_B stored → ACCEPTED
```

## Result

- 65,249,064 qualified accounts
- Each can claim exactly 1,000 ZKP tokens
- Each can choose any recipient address
- No account can claim more than once
- Privacy preserved: cannot link claims to original addresses