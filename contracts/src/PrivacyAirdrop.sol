// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

interface IVerifier {
    function verifyProof(
        uint[2] calldata _pA,
        uint[2][2] calldata _pB,
        uint[2] calldata _pC,
        uint[3] calldata _pubSignals
    ) external view returns (bool);
}

abstract contract ReentrancyGuard {
    uint256 private constant _NOT_ENTERED = 1;
    uint256 private constant _ENTERED = 2;
    uint256 private _status;

    constructor() {
        _status = _NOT_ENTERED;
    }

    modifier nonReentrant() {
        require(_status != _ENTERED, "ReentrancyGuard: reentrant call");
        _status = _ENTERED;
        _;
        _status = _NOT_ENTERED;
    }
}

contract PrivacyAirdrop {
    bytes32 public immutable merkleRoot;
    mapping(bytes32 => bool) public nullifiers;
    address public immutable token;
    uint256 public immutable claimAmount;
    uint256 public immutable claimDeadline;
    IVerifier public immutable verifier;

    event Claimed(bytes32 indexed nullifier, address indexed recipient, uint256 timestamp);

    struct Proof {
        uint[2] a;
        uint[2][2] b;
        uint[2] c;
    }

    constructor(
        address _token,
        bytes32 _merkleRoot,
        uint256 _claimAmount,
        uint256 _claimDeadline,
        address _verifier
    ) {
        require(_token != address(0), "Invalid token address");
        require(_merkleRoot != bytes32(0), "Invalid merkle root");
        require(_claimAmount > 0, "Invalid claim amount");
        require(_claimDeadline > block.timestamp, "Invalid deadline");
        require(_verifier != address(0), "Invalid verifier address");
        token = _token;
        merkleRoot = _merkleRoot;
        claimAmount = _claimAmount;
        claimDeadline = _claimDeadline;
        verifier = IVerifier(_verifier);
    }

    function claim(
        Proof calldata proof,
        bytes32 nullifier,
        address recipient
    ) external nonReentrant {
        require(block.timestamp < claimDeadline, "Claim period ended");
        require(recipient != address(0), "Invalid recipient");
        require(!nullifiers[nullifier], "Already claimed");

        uint[3] memory publicSignals = [
            uint256(merkleRoot),
            uint256(uint160(recipient)),
            uint256(nullifier)
        ];

        require(verifier.verifyProof(proof.a, proof.b, proof.c, publicSignals), "Invalid proof");

        nullifiers[nullifier] = true;

        (bool success, ) = address(token).call(
            abi.encodeWithSelector(IERC20.transfer.selector, recipient, claimAmount)
        );
        require(success, "Token transfer failed");

        emit Claimed(nullifier, recipient, block.timestamp);
    }

    function isClaimed(bytes32 nullifier) external view returns (bool) {
        return nullifiers[nullifier];
    }

    function estimateClaimGas(
        Proof calldata proof,
        bytes32 nullifier,
        address recipient
    ) external view returns (uint256) {
        return 700_000; // Conservative estimate with buffer
    }
}
