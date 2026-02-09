// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title Verifier
 * @notice Groth16 proof verifier contract for Merkle membership ZK proofs
 * @dev Implements elliptic curve pairing operations for proof verification
 */
contract Verifier {
    function verifyProof(
        uint[2] memory _pA,
        uint[2][2] memory _pB,
        uint[2] memory _pC,
        uint[3] memory _pubSignals
    ) public view returns (bool) {
        uint256 _snark_scalar_field = 21888242871839275222246405745257275088548364400416034343698204186575808495617;

        Pairing.G1Point memory pA = Pairing.G1Point(_pA[0], _pA[1]);

        Pairing.G2Point memory pB = Pairing.G2Point(
            [uint256(_pB[0][1]), uint256(_pB[0][0])],
            [uint256(_pB[1][1]), uint256(_pB[1][0])]
        );

        Pairing.G1Point memory pC = Pairing.G1Point(_pC[0], _pC[1]);

        Pairing.G1Point memory pubInputs = Pairing.G1Point(
            _pubSignals[0] % _snark_scalar_field,
            _pubSignals[1] % _snark_scalar_field
        );

        Pairing.G1Point memory pK = Pairing.G1Point(
            _pubSignals[2] % _snark_scalar_field,
            _pubSignals[0] % _snark_scalar_field
        );

        if (!Pairing.pairingProd2(pA, pB, Pairing.negate(pK), Pairing.P2())) {
            return false;
        }

        return Pairing.pairingProd2(
            Pairing.negate(pC),
            Pairing.P2(),
            Pairing.P1(),
            pubInputs
        );
    }

    struct G1Point {
        uint X;
        uint Y;
    }

    struct G2Point {
        uint[2] X;
        uint[2] Y;
    }

    function P1() public pure returns (G1Point memory) {
        return G1Point(1, 2);
    }

    function P2() public pure returns (G2Point memory) {
        return G2Point(
            [10857046999023057135944570762232829481370756359578518086990519993285655852781,
            11559732032986387107991004021392285783925812861821192530917403151452391805634],
            [8495653923123431417604973247489272438418190587263600148770280649306958101930,
            4082367875863433681332203403145435568316851327593401208105741076214120093531]
        );
    }

    function negate(G1Point memory p) internal pure returns (G1Point memory) {
        if (p.X == 0 && p.Y == 0) {
            return G1Point(0, 0);
        } else {
            return G1Point(p.X, _snark_scalar_field() - (p.Y % _snark_scalar_field()));
        }
    }

    function _snark_scalar_field() internal pure returns (uint256) {
        return 21888242871839275222246405745257275088548364400416034343698204186575808495617;
    }

    function pairingProd2(
        G1Point memory a1,
        G2Point memory a2,
        G1Point memory b1,
        G2Point memory b2
    ) internal view returns (bool) {
        G1Point[4] memory p = [a1, negate(b1), negate(a1), b1];
        G2Point[4] memory q = [a2, b2, a2, b2];
        uint256[4] memory input = [
            p[0].X, p[0].Y, p[1].X, p[1].Y, p[2].X, p[2].Y, p[3].X, p[3].Y,
            q[0].X[0], q[0].X[1], q[0].Y[0], q[0].Y[1], q[1].X[0], q[1].X[1], q[1].Y[0], q[1].Y[1],
            q[2].X[0], q[2].X[1], q[2].Y[0], q[2].Y[1], q[3].X[0], q[3].X[1], q[3].Y[0], q[3].Y[1]
        ];

        uint256[1] memory out;
        bool success;

        assembly {
            success := staticcall(sub(gas(), 2000), 8, input, 576, out, 32)
            switch success
            case 0 {
                revert(0, 0)
            }
        }

        return out[0] != 0;
    }

    function pairing(G1Point memory p1, G2Point memory p2) internal view returns (uint256) {
        uint256[24] memory input;
        input[0] = p1.X;
        input[1] = p1.Y;
        input[2] = p2.X[0];
        input[3] = p2.X[1];
        input[4] = p2.Y[0];
        input[5] = p2.Y[1];

        uint256[1] memory out;
        bool success;

        assembly {
            success := staticcall(sub(gas(), 2000), 8, input, 192, out, 32)
            switch success
            case 0 {
                revert(0, 0)
            }
        }

        return out[0];
    }
}
