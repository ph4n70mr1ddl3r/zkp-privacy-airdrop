pragma circom 2.0.0;

include "../node_modules/circomlib/circuits/poseidon.circom";

template TestPoseidon() {
    signal input in[3];
    signal output out;

    component poseidon = Poseidon(3);
    poseidon.inputs[0] <== in[0];
    poseidon.inputs[1] <== in[1];
    poseidon.inputs[2] <== in[2];

    out <== poseidon.out;
}

component main = TestPoseidon();
