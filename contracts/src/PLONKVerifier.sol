// SPDX-License-Identifier: GPL-3.0
/*
    Copyright 2021 0KIMS association.

    This file is generated with [snarkJS](https://github.com/iden3/snarkjs).

    snarkJS is a free software: you can redistribute it and/or modify it
    under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    snarkJS is distributed in the hope that it will be useful, but WITHOUT
    ANY WARRANTY; without even the implied warranty of MERCHANTABILITY
    or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public
    License for more details.

    You should have received a copy of the GNU General Public License
    along with snarkJS. If not, see <https://www.gnu.org/licenses/>.
*/


pragma solidity ^0.8.20;

contract PlonkVerifier {
    uint256 private constant W1 = 204029317488435389851510012645300498748715729336946348365670693966133783803;
    uint256 private constant Q = 21888242871839275222246405745257275088548364400416034343698204186575808495617;
    uint256 private constant QF = 21888242871839275222246405745257275088696311157297823662689037894645226208583;

    uint256 private constant G1X = 1;
    uint256 private constant G1Y = 2;
    uint256 private constant G2X1 = 10857046999023057135944570762232829481370756359578518086990519993285655852781;
    uint256 private constant G2X2 = 11559732032986387107991004021392285783925812861821192530917403151452391805634;
    uint256 private constant G2Y1 = 8495653923123431417604973247489272438418190587263600148770280649306958101930;
    uint256 private constant G2Y2 = 4082367875863433681332203403145435568316851327593401208105741076214120093531;

    uint32 private constant N = 32768;
    uint16 private constant N_PUBLIC = 3;
    uint16 private constant N_LAGRANGE = 3;

    uint256 private constant Q_MX = 2128786833672115291554563479601545063679681794051709240680814084621364539258;
    uint256 private constant Q_MY = 18982018189841681121225949193326500308916022888469996159976349944343440790344;
    uint256 private constant Q_LX = 1813756335343574096582962734926616038589495181679029540393496284351037687537;
    uint256 private constant Q_LY = 10171595212640606897416899263041713497136507460364321968233453229377585300859;
    uint256 private constant Q_RX = 7696679014182450175670283854178052847566216746962518734365491510050753974333;
    uint256 private constant Q_RY = 10093526044843329894555392940658044005311915523079596730138020161621507902676;
    uint256 private constant Q_OX = 11675874586065437555604060414363336214661765383353943712150123599013330298037;
    uint256 private constant Q_OY = 17249763395400275753532447489227743213564796196807831568110745396351614037821;
    uint256 private constant Q_CX = 12517875309754021932694403526491410758468314795692675873381039853929889015415;
    uint256 private constant Q_CY = 3859612647990378783553969090048632560207111820524146066636737668859148866963;
    uint256 private constant S1X = 8846778305305596948559301216331196654040689983867376011920717808925945338624;
    uint256 private constant S1Y = 19614641841672176996355775426266661060199906829319932266135077342455287493304;
    uint256 private constant S2X = 19621646559221276274983745838630547751044631528458831783128564655932629702301;
    uint256 private constant S2Y = 13947740039028277036054749179991507890432898865556012624197169194345412733496;
    uint256 private constant S3X = 16823670097669721981541772266204422267648377777626897203946300545813836456286;
    uint256 private constant S3Y = 20252101391878141720667760238312419116797327184727912871675882697803380926092;
    uint256 private constant K1 = 2;
    uint256 private constant K2 = 3;
    uint256 private constant X2X1 = 17847856396279484570091410360519945834888968045220508245403569997337481013916;
    uint256 private constant X2X2 = 6881726463071667325243888170110899888692120744531154987380102201069604982960;
    uint256 private constant X2Y1 = 7705972092362505012576294971550916522159291908647707169350197112553260216296;
    uint256 private constant X2Y2 = 2015291012436459172475116701717362582926430112802156794901040619858153698654;

    uint16 private constant P_A = 4 + 0;
    uint16 private constant P_B = 4 + 64;
    uint16 private constant P_C = 4 + 128;
    uint16 private constant P_Z = 4 + 192;
    uint16 private constant P_T1 = 4 + 256;
    uint16 private constant P_T2 = 4 + 320;
    uint16 private constant P_T3 = 4 + 384;
    uint16 private constant P_WXI = 4 + 448;
    uint16 private constant P_WXIW = 4 + 512;
    uint16 private constant P_EVAL_A = 4 + 576;
    uint16 private constant P_EVAL_B = 4 + 608;
    uint16 private constant P_EVAL_C = 4 + 640;
    uint16 private constant P_EVAL_S1 = 4 + 672;
    uint16 private constant P_EVAL_S2 = 4 + 704;
    uint16 private constant P_EVAL_ZW = 4 + 736;

    uint16 private constant P_ALPHA = 0;
    uint16 private constant P_BETA = 32;
    uint16 private constant P_GAMMA = 64;
    uint16 private constant P_XI = 96;
    uint16 private constant P_XIN = 128;
    uint16 private constant P_BETA_XI = 160;
    uint16 private constant P_V1 = 192;
    uint16 private constant P_V2 = 224;
    uint16 private constant P_V3 = 256;
    uint16 private constant P_V4 = 288;
    uint16 private constant P_V5 = 320;
    uint16 private constant P_U = 352;

    uint16 private constant P_PI = 384;
    uint16 private constant P_EVAL_R0 = 416;
    uint16 private constant P_D = 448;
    uint16 private constant P_F = 512;
    uint16 private constant P_E = 576;
    uint16 private constant P_TMP = 640;
    uint16 private constant P_ALPHA2 = 704;
    uint16 private constant P_ZH = 736;
    uint16 private constant P_ZH_INV = 768;

    uint16 private constant P_EVAL_L1 = 800;
    uint16 private constant P_EVAL_L2 = 832;
    uint16 private constant P_EVAL_L3 = 864;

    uint16 private constant LAST_MEM = 896;

    // solhint-disable no-inline-assembly
    // This verifier is auto-generated by snarkJS and uses assembly for performance
    /* eslint-disable */
    function verifyProof(uint256[24] calldata, uint256[3] calldata _pubSignals) public view returns (bool) {
        assembly {
            /////////
            // Computes the inverse using the extended euclidean algorithm
            /////////
            function inverse(a, q) -> inv {
                let t := 0     
                let newt := 1
                let r := q     
                let newr := a
                let quotient
                let aux
                
                for { } newr { } {
                    quotient := sdiv(r, newr)
                    aux := sub(t, mul(quotient, newt))
                    t:= newt
                    newt:= aux
                    
                    aux := sub(r,mul(quotient, newr))
                    r := newr
                    newr := aux
                }
                
                if gt(r, 1) { revert(0,0) }
                if slt(t, 0) { t:= add(t, q) }

                inv := t
            }
            
            ///////
            // Computes the inverse of an array of values
            // See https://vitalik.ca/general/2018/07/21/starks_part_3.html in section where explain fields operations
            //////
            function inverseArray(pVals, n) {

                let pAux := mload(0x40)     // Point to the next free position
                let pIn := pVals
                let lastPIn := add(pVals, mul(n, 32))  // Read n elements
                let acc := mload(pIn)       // Read the first element
                pIn := add(pIn, 32)         // Point to the second element
                let inv


                for { } lt(pIn, lastPIn) {
                    pAux := add(pAux, 32)
                    pIn := add(pIn, 32)
                }
                {
                    mstore(pAux, acc)
                    acc := mulmod(acc, mload(pIn), Q)
                }
                acc := inverse(acc, Q)

                // At this point pAux pint to the next free position we subtract 1 to point to the last used
                pAux := sub(pAux, 32)
                // pIn points to the n+1 element, we subtract to point to n
                pIn := sub(pIn, 32)
                lastPIn := pVals  // We don't process the first element
                for { } gt(pIn, lastPIn) {
                    pAux := sub(pAux, 32)
                    pIn := sub(pIn, 32)
                }
                {
                    inv := mulmod(acc, mload(pAux), Q)
                    acc := mulmod(acc, mload(pIn), Q)
                    mstore(pIn, inv)
                }
                // pIn points to first element, we just set it.
                mstore(pIn, acc)
            }
            
            function checkField(v) {
                if iszero(lt(v, Q)) {
                    mstore(0, 0)
                    return(0,0x20)
                }
            }
            
            function checkPointBelongsToBN128Curve(p) {
                let x := calldataload(p)
                let y := calldataload(add(p, 32))

                // Check that point is on curve
                // y^2 = x^3 + 3
                let x3_3 := addmod(mulmod(x, mulmod(x, x, QF), QF), 3, QF)
                let y2 := mulmod(y, y, QF)

                if iszero(eq(x3_3, y2)) {
                    mstore(0, 0)
                    return(0, 0x20)
                }
            }  

            function checkProofData() {
                checkPointBelongsToBN128Curve(P_A)
                checkPointBelongsToBN128Curve(P_B)
                checkPointBelongsToBN128Curve(P_C)
                checkPointBelongsToBN128Curve(P_Z)
                checkPointBelongsToBN128Curve(P_T1)
                checkPointBelongsToBN128Curve(P_T2)
                checkPointBelongsToBN128Curve(P_T3)
                checkPointBelongsToBN128Curve(P_WXI)
                checkPointBelongsToBN128Curve(P_WXIW)

                checkField(calldataload(P_A))
                checkField(calldataload(add(P_A, 32)))
                checkField(calldataload(P_B))
                checkField(calldataload(add(P_B, 32)))
                checkField(calldataload(P_C))
                checkField(calldataload(add(P_C, 32)))
                checkField(calldataload(P_Z))
                checkField(calldataload(add(P_Z, 32)))
                checkField(calldataload(P_T1))
                checkField(calldataload(add(P_T1, 32)))
                checkField(calldataload(P_T2))
                checkField(calldataload(add(P_T2, 32)))
                checkField(calldataload(P_T3))
                checkField(calldataload(add(P_T3, 32)))
                checkField(calldataload(P_WXI))
                checkField(calldataload(add(P_WXI, 32)))
                checkField(calldataload(P_WXIW))
                checkField(calldataload(add(P_WXIW, 32)))

                checkField(calldataload(P_EVAL_A))
                checkField(calldataload(P_EVAL_B))
                checkField(calldataload(P_EVAL_C))
                checkField(calldataload(P_EVAL_S1))
                checkField(calldataload(P_EVAL_S2))
                checkField(calldataload(P_EVAL_ZW))
            }
            
            function calculateChallenges(pMem, pPublic) {
                let beta
                let aux

                let mIn := mload(0x40)

                mstore(mIn, Q_MX)
                mstore(add(mIn, 32), Q_MY)
                mstore(add(mIn, 64), Q_LX)
                mstore(add(mIn, 96), Q_LY)
                mstore(add(mIn, 128), Q_RX)
                mstore(add(mIn, 160), Q_RY)
                mstore(add(mIn, 192), Q_OX)
                mstore(add(mIn, 224), Q_OY)
                mstore(add(mIn, 256), Q_CX)
                mstore(add(mIn, 288), Q_CY)
                mstore(add(mIn, 320), S1X)
                mstore(add(mIn, 352), S1Y)
                mstore(add(mIn, 384), S2X)
                mstore(add(mIn, 416), S2Y)
                mstore(add(mIn, 448), S3X)
                mstore(add(mIn, 480), S3Y)

                mstore(add(mIn, 512), calldataload(add(pPublic, 0)))
                mstore(add(mIn, 544), calldataload(add(pPublic, 32)))
                mstore(add(mIn, 576), calldataload(add(pPublic, 64)))
                mstore(add(mIn, 608), calldataload(P_A))
                mstore(add(mIn, 640), calldataload(add(P_A, 32)))
                mstore(add(mIn, 672), calldataload(P_B))
                mstore(add(mIn, 704), calldataload(add(P_B, 32)))
                mstore(add(mIn, 736), calldataload(P_C))
                mstore(add(mIn, 768), calldataload(add(P_C, 32)))

                beta := mod(keccak256(mIn, 800), Q)
                mstore(add(pMem, P_BETA), beta)

                mstore(add(pMem, P_GAMMA), mod(keccak256(add(pMem, P_BETA), 32), Q))

                mstore(mIn, mload(add(pMem, P_BETA)))
                mstore(add(mIn, 32), mload(add(pMem, P_GAMMA)))
                mstore(add(mIn, 64), calldataload(P_Z))
                mstore(add(mIn, 96), calldataload(add(P_Z, 32)))

                aux := mod(keccak256(mIn, 128), Q)
                mstore(add(pMem, P_ALPHA), aux)
                mstore(add(pMem, P_ALPHA2), mulmod(aux, aux, Q))

                mstore(mIn, aux)
                mstore(add(mIn, 32), calldataload(P_T1))
                mstore(add(mIn, 64), calldataload(add(P_T1, 32)))
                mstore(add(mIn, 96), calldataload(P_T2))
                mstore(add(mIn, 128), calldataload(add(P_T2, 32)))
                mstore(add(mIn, 160), calldataload(P_T3))
                mstore(add(mIn, 192), calldataload(add(P_T3, 32)))

                aux := mod(keccak256(mIn, 224), Q)
                mstore(add(pMem, P_XI), aux)

                mstore(mIn, aux)
                mstore(add(mIn, 32), calldataload(P_EVAL_A))
                mstore(add(mIn, 64), calldataload(P_EVAL_B))
                mstore(add(mIn, 96), calldataload(P_EVAL_C))
                mstore(add(mIn, 128), calldataload(P_EVAL_S1))
                mstore(add(mIn, 160), calldataload(P_EVAL_S2))
                mstore(add(mIn, 192), calldataload(P_EVAL_ZW))

                let v1 := mod(keccak256(mIn, 224), Q)
                mstore(add(pMem, P_V1), v1)

                mstore(add(pMem, P_BETA_XI), mulmod(beta, aux, Q))

                aux:= mulmod(aux, aux, Q)
                aux:= mulmod(aux, aux, Q)
                aux:= mulmod(aux, aux, Q)
                aux:= mulmod(aux, aux, Q)
                aux:= mulmod(aux, aux, Q)
                aux:= mulmod(aux, aux, Q)
                aux:= mulmod(aux, aux, Q)
                aux:= mulmod(aux, aux, Q)
                aux:= mulmod(aux, aux, Q)
                aux:= mulmod(aux, aux, Q)
                aux:= mulmod(aux, aux, Q)
                aux:= mulmod(aux, aux, Q)
                aux:= mulmod(aux, aux, Q)
                aux:= mulmod(aux, aux, Q)

                mstore(add(pMem, P_XIN), aux)

                aux:= mod(add(sub(aux, 1), Q), Q)
                mstore(add(pMem, P_ZH), aux)
                mstore(add(pMem, P_ZH_INV), aux)

                aux := mulmod(v1, v1, Q)
                mstore(add(pMem, P_V2), aux)
                aux := mulmod(aux, v1, Q)
                mstore(add(pMem, P_V3), aux)
                aux := mulmod(aux, v1, Q)
                mstore(add(pMem, P_V4), aux)
                aux := mulmod(aux, v1, Q)
                mstore(add(pMem, P_V5), aux)

                mstore(mIn, calldataload(P_WXI))
                mstore(add(mIn, 32), calldataload(add(P_WXI, 32)))
                mstore(add(mIn, 64), calldataload(P_WXIW))
                mstore(add(mIn, 96), calldataload(add(P_WXIW, 32)))

                mstore(add(pMem, P_U), mod(keccak256(mIn, 128), Q))
            }
            
            function calculateLagrange(pMem) {
                let w := 1

                mstore(
                    add(pMem, P_EVAL_L1),
                    mulmod(
                        N,
                        mod(
                            add(
                                sub(
                                    mload(add(pMem, P_XI)),
                                    w
                                ),
                                Q
                            ),
                            Q
                        ),
                        Q
                    )
                )

                w := mulmod(w, W1, Q)

                mstore(
                    add(pMem, P_EVAL_L2),
                    mulmod(
                        N,
                        mod(
                            add(
                                sub(
                                    mload(add(pMem, P_XI)),
                                    w
                                ),
                                Q
                            ),
                            Q
                        ),
                        Q
                    )
                )

                w := mulmod(w, W1, Q)

                mstore(
                    add(pMem, P_EVAL_L3),
                    mulmod(
                        N,
                        mod(
                            add(
                                sub(
                                    mload(add(pMem, P_XI)),
                                    w
                                ),
                                Q
                            ),
                            Q
                        ),
                        Q
                    )
                )

                inverseArray(add(pMem, P_ZH_INV), 4)

                let zh := mload(add(pMem, P_ZH))
                w := 1

                mstore(
                    add(pMem, P_EVAL_L1),
                    mulmod(
                        mload(add(pMem, P_EVAL_L1)),
                        zh,
                        Q
                    )
                )

                w := mulmod(w, W1, Q)

                mstore(
                    add(pMem, P_EVAL_L2),
                    mulmod(
                        w,
                        mulmod(
                            mload(add(pMem, P_EVAL_L2)),
                            zh,
                            Q
                        ),
                        Q
                    )
                )

                w := mulmod(w, W1, Q)

                mstore(
                    add(pMem, P_EVAL_L3),
                    mulmod(
                        w,
                        mulmod(
                            mload(add(pMem, P_EVAL_L3)),
                            zh,
                            Q
                        ),
                        Q
                    )
                )
            }
            
            function calculatePI(pMem, pPub) {
                let pl := 0

                pl := mod(
                    add(
                        sub(
                            pl,
                            mulmod(
                                mload(add(pMem, P_EVAL_L1)),
                                calldataload(add(pPub, 0)),
                                Q
                            )
                        ),
                        Q
                    ),
                    Q
                )

                pl := mod(
                    add(
                        sub(
                            pl,
                            mulmod(
                                mload(add(pMem, P_EVAL_L2)),
                                calldataload(add(pPub, 32)),
                                Q
                            )
                        ),
                        Q
                    ),
                    Q
                )

                pl := mod(
                    add(
                        sub(
                            pl,
                            mulmod(
                                mload(add(pMem, P_EVAL_L3)),
                                calldataload(add(pPub, 64)),
                                Q
                            )
                        ),
                        Q
                    ),
                    Q
                )

                mstore(add(pMem, P_PI), pl)
            }

            function calculateR0(pMem) {
                let e1 := mload(add(pMem, P_PI))

                let e2 := mulmod(mload(add(pMem, P_EVAL_L1)), mload(add(pMem, P_ALPHA2)), Q)

                let e3a := addmod(
                    calldataload(P_EVAL_A),
                    mulmod(mload(add(pMem, P_BETA)), calldataload(P_EVAL_S1), Q),
                    Q)
                e3a := addmod(e3a, mload(add(pMem, P_GAMMA)), Q)

                let e3b := addmod(
                    calldataload(P_EVAL_B),
                    mulmod(mload(add(pMem, P_BETA)), calldataload(P_EVAL_S2), Q),
                    Q)
                e3b := addmod(e3b, mload(add(pMem, P_GAMMA)), Q)

                let e3c := addmod(
                    calldataload(P_EVAL_C),
                    mload(add(pMem, P_GAMMA)),
                    Q)

                let e3 := mulmod(mulmod(e3a, e3b, Q), e3c, Q)
                e3 := mulmod(e3, calldataload(P_EVAL_ZW), Q)
                e3 := mulmod(e3, mload(add(pMem, P_ALPHA)), Q)

                let r0 := addmod(e1, mod(sub(Q, e2), Q), Q)
                r0 := addmod(r0, mod(sub(Q, e3), Q), Q)

                mstore(add(pMem, P_EVAL_R0) , r0)
            }
            
            function g1_set(pR, pP) {
                mstore(pR, mload(pP))
                mstore(add(pR, 32), mload(add(pP,32)))
            }   

            function g1_setC(pR, x, y) {
                mstore(pR, x)
                mstore(add(pR, 32), y)
            }

            function g1_calldataSet(pR, pP) {
                mstore(pR,          calldataload(pP))
                mstore(add(pR, 32), calldataload(add(pP, 32)))
            }

            function g1_acc(pR, pP) {
                let mIn := mload(0x40)
                mstore(mIn, mload(pR))
                mstore(add(mIn,32), mload(add(pR, 32)))
                mstore(add(mIn,64), mload(pP))
                mstore(add(mIn,96), mload(add(pP, 32)))

                let success := staticcall(sub(gas(), 2000), 6, mIn, 128, pR, 64)
                
                if iszero(success) {
                    mstore(0, 0)
                    return(0,0x20)
                }
            }

            function g1_mulAcc(pR, pP, s) {
                let success
                let mIn := mload(0x40)
                mstore(mIn, mload(pP))
                mstore(add(mIn,32), mload(add(pP, 32)))
                mstore(add(mIn,64), s)

                success := staticcall(sub(gas(), 2000), 7, mIn, 96, mIn, 64)
                
                if iszero(success) {
                    mstore(0, 0)
                    return(0,0x20)
                }
                
                mstore(add(mIn,64), mload(pR))
                mstore(add(mIn,96), mload(add(pR, 32)))

                success := staticcall(sub(gas(), 2000), 6, mIn, 128, pR, 64)
                
                if iszero(success) {
                    mstore(0, 0)
                    return(0,0x20)
                }
                
            }

            function g1_mulAccC(pR, x, y, s) {
                let success
                let mIn := mload(0x40)
                mstore(mIn, x)
                mstore(add(mIn,32), y)
                mstore(add(mIn,64), s)

                success := staticcall(sub(gas(), 2000), 7, mIn, 96, mIn, 64)
                
                if iszero(success) {
                    mstore(0, 0)
                    return(0,0x20)
                }
                
                mstore(add(mIn,64), mload(pR))
                mstore(add(mIn,96), mload(add(pR, 32)))

                success := staticcall(sub(gas(), 2000), 6, mIn, 128, pR, 64)
                
                if iszero(success) {
                    mstore(0, 0)
                    return(0,0x20)
                }
            }

            function g1_mulSetC(pR, x, y, s) {
                let success
                let mIn := mload(0x40)
                mstore(mIn, x)
                mstore(add(mIn,32), y)
                mstore(add(mIn,64), s)

                success := staticcall(sub(gas(), 2000), 7, mIn, 96, pR, 64)
                
                if iszero(success) {
                    mstore(0, 0)
                    return(0,0x20)
                }
            }

            function g1_mulSet(pR, pP, s) {
                g1_mulSetC(pR, mload(pP), mload(add(pP, 32)), s)
            }

            function calculateD(pMem) {
                let _pD:= add(pMem, P_D)
                let gamma := mload(add(pMem, P_GAMMA))
                let mIn := mload(0x40)
                mstore(0x40, add(mIn, 256))

                g1_setC(_pD, Q_CX, Q_CY)
                g1_mulAccC(_pD, Q_MX, Q_MY, mulmod(calldataload(P_EVAL_A), calldataload(P_EVAL_B), Q))
                g1_mulAccC(_pD, Q_LX, Q_LY, calldataload(P_EVAL_A))
                g1_mulAccC(_pD, Q_RX, Q_RY, calldataload(P_EVAL_B))
                g1_mulAccC(_pD, Q_OX, Q_OY, calldataload(P_EVAL_C))

                let betaxi := mload(add(pMem, P_BETA_XI))
                let val1 := addmod(
                    addmod(calldataload(P_EVAL_A), betaxi, Q),
                    gamma, Q)

                let val2 := addmod(
                    addmod(
                        calldataload(P_EVAL_B),
                        mulmod(betaxi, K1, Q),
                        Q), gamma, Q)

                let val3 := addmod(
                    addmod(
                        calldataload(P_EVAL_C),
                        mulmod(betaxi, K2, Q),
                        Q), gamma, Q)

                let d2a := mulmod(
                    mulmod(mulmod(val1, val2, Q), val3, Q),
                    mload(add(pMem, P_ALPHA)),
                    Q
                )

                let d2b := mulmod(
                    mload(add(pMem, P_EVAL_L1)),
                    mload(add(pMem, P_ALPHA2)),
                    Q
                )

                g1_calldataSet(add(mIn, 192), P_Z)
                g1_mulSet(
                    mIn,
                    add(mIn, 192),
                    addmod(addmod(d2a, d2b, Q), mload(add(pMem, P_U)), Q))

                val1 := addmod(
                    addmod(
                        calldataload(P_EVAL_A),
                        mulmod(mload(add(pMem, P_BETA)), calldataload(P_EVAL_S1), Q),
                        Q), gamma, Q)

                val2 := addmod(
                    addmod(
                        calldataload(P_EVAL_B),
                        mulmod(mload(add(pMem, P_BETA)), calldataload(P_EVAL_S2), Q),
                        Q), gamma, Q)

                val3 := mulmod(
                    mulmod(mload(add(pMem, P_ALPHA)), mload(add(pMem, P_BETA)), Q),
                    calldataload(P_EVAL_ZW), Q)

                g1_mulSetC(
                    add(mIn, 64),
                    S3X,
                    S3Y,
                    mulmod(mulmod(val1, val2, Q), val3, Q))

                g1_calldataSet(add(mIn, 128), P_T1)

                g1_mulAccC(add(mIn, 128), calldataload(P_T2), calldataload(add(P_T2, 32)), mload(add(pMem, P_XIN)))
                let xin2 := mulmod(mload(add(pMem, P_XIN)), mload(add(pMem, P_XIN)), Q)
                g1_mulAccC(add(mIn, 128), calldataload(P_T3), calldataload(add(P_T3, 32)) , xin2)

                g1_mulSetC(add(mIn, 128), mload(add(mIn, 128)), mload(add(mIn, 160)), mload(add(pMem, P_ZH)))

                mstore(add(add(mIn, 64), 32), mod(sub(QF, mload(add(add(mIn, 64), 32))), QF))
                mstore(add(mIn, 160), mod(sub(QF, mload(add(mIn, 160))), QF))
                g1_acc(_pD, mIn)
                g1_acc(_pD, add(mIn, 64))
                g1_acc(_pD, add(mIn, 128))
            }

            function calculateF(pMem) {
                let p := add(pMem, P_F)

                g1_set(p, add(pMem, P_D))
                g1_mulAccC(p, calldataload(P_A), calldataload(add(P_A, 32)), mload(add(pMem, P_V1)))
                g1_mulAccC(p, calldataload(P_B), calldataload(add(P_B, 32)), mload(add(pMem, P_V2)))
                g1_mulAccC(p, calldataload(P_C), calldataload(add(P_C, 32)), mload(add(pMem, P_V3)))
                g1_mulAccC(p, S1X, S1Y, mload(add(pMem, P_V4)))
                g1_mulAccC(p, S2X, S2Y, mload(add(pMem, P_V5)))
            }

            function calculateE(pMem) {
                let s := mod(sub(Q, mload(add(pMem, P_EVAL_R0))), Q)

                s := addmod(s, mulmod(calldataload(P_EVAL_A),  mload(add(pMem, P_V1)), Q), Q)
                s := addmod(s, mulmod(calldataload(P_EVAL_B),  mload(add(pMem, P_V2)), Q), Q)
                s := addmod(s, mulmod(calldataload(P_EVAL_C),  mload(add(pMem, P_V3)), Q), Q)
                s := addmod(s, mulmod(calldataload(P_EVAL_S1), mload(add(pMem, P_V4)), Q), Q)
                s := addmod(s, mulmod(calldataload(P_EVAL_S2), mload(add(pMem, P_V5)), Q), Q)
                s := addmod(s, mulmod(calldataload(P_EVAL_ZW), mload(add(pMem, P_U)),  Q), Q)

                g1_mulSetC(add(pMem, P_E), G1X, G1Y, s)
            }

            function checkPairing(pMem) -> isOk {
                let mIn := mload(0x40)
                mstore(0x40, add(mIn, 576))

                let _pWxi := add(mIn, 384)
                let _pWxiw := add(mIn, 448)
                let _aux := add(mIn, 512)

                g1_calldataSet(_pWxi, P_WXI)
                g1_calldataSet(_pWxiw, P_WXIW)

                g1_mulSet(mIn, _pWxiw, mload(add(pMem, P_U)))
                g1_acc(mIn, _pWxi)
                mstore(add(mIn, 32), mod(sub(QF, mload(add(mIn, 32))), QF))

                mstore(add(mIn,64), X2X2)
                mstore(add(mIn,96), X2X1)
                mstore(add(mIn,128), X2Y2)
                mstore(add(mIn,160), X2Y1)

                g1_mulSet(add(mIn, 192), _pWxi, mload(add(pMem, P_XI)))

                let s := mulmod(mload(add(pMem, P_U)), mload(add(pMem, P_XI)), Q)
                s := mulmod(s, W1, Q)
                g1_mulSet(_aux, _pWxiw, s)
                g1_acc(add(mIn, 192), _aux)
                g1_acc(add(mIn, 192), add(pMem, P_F))
                mstore(add(pMem, add(P_E, 32)), mod(sub(QF, mload(add(pMem, add(P_E, 32)))), QF))
                g1_acc(add(mIn, 192), add(pMem, P_E))

                mstore(add(mIn,256), G2X2)
                mstore(add(mIn,288), G2X1)
                mstore(add(mIn,320), G2Y2)
                mstore(add(mIn,352), G2Y1)

                let success := staticcall(sub(gas(), 2000), 8, mIn, 384, mIn, 0x20)

                isOk := and(success, mload(mIn))
            }

            let pMem := mload(0x40)
            mstore(0x40, add(pMem, LAST_MEM))

            checkProofData()
            calculateChallenges(pMem, _pubSignals)
            calculateLagrange(pMem)
            calculatePI(pMem, _pubSignals)
            calculateR0(pMem)
            calculateD(pMem)
            calculateF(pMem)
            calculateE(pMem)
            let isValid := checkPairing(pMem)

            mstore(0x40, sub(pMem, LAST_MEM))
            mstore(0, isValid)
            return(0,0x20)
        }

    }
}
