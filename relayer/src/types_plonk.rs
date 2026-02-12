use ethers::contract::abigen;

abigen!(
    IPLONKVerifier,
    r#"[
        function claim(uint256[8] calldata proof, bytes32 nullifier, address recipient) external returns (uint256)
        function isClaimed(bytes32 nullifier) external view returns (bool)
        function getInstanceCount() external view returns (uint256)
        function getProofElementCount() external view returns (uint256)
    ]"#,
);

pub use zkp_airdrop_utils::types::{
    CheckStatusResponse, ContractDetails, ContractInfoResponse, ContractsInfo, DonateRequest,
    ErrorResponse, HealthResponse, MerklePathResponse, MerkleRootResponse, Proof, RateLimitType,
    RelayerWalletInfo, ResponseTime, Services, StatsResponse, SubmitClaimRequest,
    SubmitClaimResponse, TokenDetails,
};
