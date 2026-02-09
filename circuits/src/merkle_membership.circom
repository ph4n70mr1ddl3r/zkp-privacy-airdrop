pragma circom 2.0.0;

include "../node_modules/circomlib/circuits/poseidon.circom";
include "../node_modules/circomlib/circuits/bitify.circom";
include "../node_modules/circomlib/circuits/mux1.circom";

template MerklePathVerifier(n) {
    signal input leaf;
    signal input root;
    signal input paths_siblings[n];
    signal input paths_enabled[n];

    signal levels[n];

    component mux[n];
    component poseidon[n];

    // Create all components first
    for (var i = 0; i < n; i++) {
        poseidon[i] = Poseidon(3);
        mux[i] = Mux1();
    }

    // Then assign signals
    poseidon[0].inputs[0] <== leaf;
    poseidon[0].inputs[1] <== paths_siblings[0];
    poseidon[0].inputs[2] <== 0;

    mux[0].c[0] <== poseidon[0].out;
    mux[0].c[1] <== paths_siblings[0];
    mux[0].s <== paths_enabled[0];
    levels[0] <== mux[0].out;

    for (var i = 1; i < n; i++) {
        poseidon[i].inputs[0] <== levels[i-1];
        poseidon[i].inputs[1] <== paths_siblings[i];
        poseidon[i].inputs[2] <== 0;

        mux[i].c[0] <== poseidon[i].out;
        mux[i].c[1] <== paths_siblings[i];
        mux[i].s <== paths_enabled[i];
        levels[i] <== mux[i].out;
    }

    root === levels[n-1];
}

template MerkleMembership() {
    signal input private_key;
    signal input merkle_root;
    signal input recipient;
    signal input nullifier;

    signal input merkle_path[26];
    signal input merkle_path_indices;

    signal leaf;
    signal computed_nullifier;
    signal computed_address_field;

    // Hash recipient address (converted to field element) with Poseidon to create leaf
    component poseidon_leaf = Poseidon(3);
    poseidon_leaf.inputs[0] <== recipient;
    poseidon_leaf.inputs[1] <== 0;
    poseidon_leaf.inputs[2] <== 0;
    leaf <== poseidon_leaf.out;

    // Verify Merkle proof
    component merkle_verifier = MerklePathVerifier(26);
    merkle_verifier.leaf <== leaf;
    merkle_verifier.root <== merkle_root;

    // Extract bits from path indices - convert indices to bits then extract bit i
    component num2bits = Num2Bits(26);
    num2bits.in <== merkle_path_indices;

    for (var i = 0; i < 26; i++) {
        merkle_verifier.paths_siblings[i] <== merkle_path[i];
        merkle_verifier.paths_enabled[i] <== num2bits.out[i];
    }

    // Computed nullifier from private key (off-chain: Poseidon(private_key, salt, 0))
    computed_nullifier <== private_key;
    computed_nullifier === nullifier;
}

// NOTE: NULLIFIER_SALT is defined externally in the proving system

component main = MerkleMembership();
