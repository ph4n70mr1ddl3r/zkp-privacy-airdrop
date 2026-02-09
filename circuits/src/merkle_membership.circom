pragma circom 2.0.0;

include "../node_modules/circomlib/circuits/poseidon.circom";
include "../node_modules/circomlib/circuits/bitify.circom";
include "../node_modules/circomlib/circuits/mux1.circom";
include "./circomlib/circuits/escalarmulany.circom";
include "./circomlib/circuits/ecc.circom";

// Salt value for nullifier generation to prevent precomputation attacks
// This value should be generated securely and kept constant for the circuit's lifetime
// Changing this value requires recomputing all nullifiers and regenerating verification keys
// Current value: 87953108768114088221452414019732140257409482096940319490691914651639977587459
// Randomly generated 256-bit value
const NULLIFIER_SALT = 87953108768114088221452414019732140257409482096940319490691914651639977587459;

template MerklePathVerifier(n) {
    signal input leaf;
    signal input root;
    signal input paths_siblings[n];
    signal input paths_enabled[n];
    
    signal levels[n];
    
    component mux[n];
    component poseidon[n];
    
    poseidon[0].in[0] <== leaf;
    poseidon[0].in[1] <== paths_siblings[0];
    poseidon[0].in[2] <== 0;
    
    mux[0] = Mux1();
    mux[0].c[0] <== poseidon[0].out;
    mux[0].c[1] <== paths_siblings[0];
    mux[0].s <== paths_enabled[0];
    levels[0] <== mux[0].out;
    
    for (var i = 1; i < n; i++) {
        poseidon[i].in[0] <== levels[i-1];
        poseidon[i].in[1] <== paths_siblings[i];
        poseidon[i].in[2] <== 0;
        
        mux[i] = Mux1();
        mux[i].c[0] <== poseidon[i].out;
        mux[i].c[1] <== paths_siblings[i];
        mux[i].s <== paths_enabled[i];
        levels[i] <== mux[i].out;
    }
    
    root === levels[n-1];
}

template ScalarMul() {
    signal input in;
    signal output out[2];
    
    component ec = ECMul();
    ec.in <== in;
    out[0] <== ec.out[0];
    out[1] <== ec.out[1];
}

template PubKeyToAddress() {
    signal input pub_key_x;
    signal input pub_key_y;
    signal output address;
    
    component hash = Keccak(2, 256, 512);
    hash.in[0] <== pub_key_x;
    hash.in[1] <== pub_key_y;
    hash.in[2] <== 0;
    hash.in[3] <== 0;
    hash.in[4] <== 0;
    hash.in[5] <== 0;
    hash.in[6] <== 0;
    hash.in[7] <== 0;
    
    address <== hash.out[0];
}

template MerkleMembership() {
    signal private input private_key;
    signal input merkle_root;
    signal input recipient;
    signal input nullifier;
    
    signal private input merkle_path[26];
    signal private input merkle_path_indices;
    
    signal pub_key_x;
    signal pub_key_y;
    signal address;
    signal leaf;
    
    signal computed_nullifier;
    signal computed_address_field;
    
    component mul = ScalarMul();
    mul.in <== private_key;
    pub_key_x <== mul.out[0];
    pub_key_y <== mul.out[1];
    
    component pubkey_to_addr = PubKeyToAddress();
    pubkey_to_addr.pub_key_x <== pub_key_x;
    pubkey_to_addr.pub_key_y <== pub_key_y;
    address <== pubkey_to_addr.address;
    
    component poseidon_leaf = Poseidon(3);
    poseidon_leaf.in[0] <== address;
    poseidon_leaf.in[1] <== 0;
    poseidon_leaf.in[2] <== 0;
    leaf <== poseidon_leaf.out;
    
    component merkle_verifier = MerklePathVerifier(26);
    merkle_verifier.leaf <== leaf;
    merkle_verifier.root <== merkle_root;
    for (var i = 0; i < 26; i++) {
        merkle_verifier.paths_siblings[i] <== merkle_path[i];
        merkle_verifier.paths_enabled[i] <== (merkle_path_indices >> i) & 1;
    }
    
    computed_address_field <== address;
    computed_address_field === recipient;
    
    component poseidon_nullifier = Poseidon(3);
    poseidon_nullifier.in[0] <== private_key;
    poseidon_nullifier.in[1] <== NULLIFIER_SALT;
    poseidon_nullifier.in[2] <== 0;
    computed_nullifier <== poseidon_nullifier.out;
    computed_nullifier === nullifier;
}

component main = MerkleMembership();
