use actix_web::{web, HttpRequest, HttpResponse, Responder};
use tracing::{info, error, warn};
use std::time::SystemTime;

use crate::state::AppState;
use crate::types_plonk::*;

pub async fn health(
    state: web::Data<AppState>,
) -> impl Responder {
    let services = Services {
        database: "connected".to_string(),
        redis: "connected".to_string(),
        optimism_node: "connected".to_string(),
        relayer_wallet: RelayerWalletInfo {
            address: state.relayer_address(),
            balance: state.get_relayer_balance().await.to_string(),
            sufficient: state.has_sufficient_balance().await,
        },
    };

    HttpResponse::Ok().json(HealthResponse {
        status: if state.is_healthy().await { "healthy".to_string() } else { "unhealthy".to_string() },
        timestamp: format!("{:?}", SystemTime::now()),
        version: "1.0.0".to_string(),
        services,
    })
}

pub async fn submit_claim(
    req: HttpRequest,
    state: web::Data<AppState>,
    claim: web::Json<SubmitClaimRequest>,
) -> impl Responder {
    info!("Received {} claim submission from nullifier: {}", claim.proof.type_name(), claim.nullifier);

    // Rate limiting check
    if let Err(e) = state.check_rate_limit(&req, &claim.nullifier, RateLimitType::SubmitClaim).await {
        warn!("Rate limit exceeded: {}", e);
        return HttpResponse::TooManyRequests().json(ErrorResponse {
            success: false,
            error: "Rate limit exceeded. Try again later.".to_string(),
            code: Some("RATE_LIMITED".to_string()),
            retry_after: Some(60),
        });
    }

    // Check if already claimed
    if state.is_nullifier_used(&claim.nullifier).await {
        warn!("Nullifier already claimed: {}", claim.nullifier);
        return HttpResponse::BadRequest().json(ErrorResponse {
            success: false,
            error: "This nullifier has already been used. Each qualified account can only claim once.".to_string(),
            code: Some("ALREADY_CLAIMED".to_string()),
            retry_after: None,
        });
    }

    // Validate proof structure
    if !claim.proof.is_valid_structure() {
        warn!("Invalid {} proof structure", claim.proof.type_name());
        let error_code = if claim.proof.type_name() == "Plonk" {
            "PLONK_FORMAT_ERROR"
        } else {
            "INVALID_PROOF"
        };
        let error_message = if claim.proof.type_name() == "Plonk" {
            "PLONK proof format is invalid. Expected 8 field elements.".to_string()
        } else {
            format!("The provided {} proof is invalid. Please regenerate proof with correct inputs.", claim.proof.type_name())
        };
        return HttpResponse::BadRequest().json(ErrorResponse {
            success: false,
            error: error_message,
            code: Some(error_code.to_string()),
            retry_after: None,
        });
    }

    info!("Validated {} proof successfully", claim.proof.type_name());

    // PLONK-specific warning
    if claim.proof.type_name() == "Plonk" {
        info!("PLONK proof detected - verification gas estimate: ~1.3M");
    }

    // Check relayer balance
    if !state.has_sufficient_balance().await {
        error!("Insufficient relayer balance");
        return HttpResponse::ServiceUnavailable().json(ErrorResponse {
            success: false,
            error: "Relayer temporarily unavailable due to insufficient funds. Please try another relayer or submit directly to the contract.".to_string(),
            code: Some("INSUFFICIENT_FUNDS".to_string()),
            retry_after: None,
        });
    }

    // Submit transaction
    match state.submit_claim(&claim).await {
        Ok(tx_hash) => {
            info!("Claim submitted successfully ({}): {}", claim.proof.type_name(), tx_hash);
            HttpResponse::Ok().json(SubmitClaimResponse {
                success: true,
                tx_hash: Some(tx_hash),
                status: Some("pending".to_string()),
                estimated_confirmation: Some(format!("{:?}", SystemTime::now())),
                error: None,
                code: None,
            })
        }
        Err(e) => {
            error!("Failed to submit claim: {} - Error: {}", claim.nullifier, e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                success: false,
                error: "Failed to submit claim. Please try again later.".to_string(),
                code: Some("INTERNAL_ERROR".to_string()),
                retry_after: Some(60),
            })
        }
    }
}

pub async fn check_status(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    let nullifier = path.into_inner();

    info!("Checking status for nullifier: {}", nullifier);

    match state.get_claim_status(&nullifier).await {
        Some(status) => HttpResponse::Ok().json(status),
        None => HttpResponse::Ok().json(CheckStatusResponse {
            nullifier,
            claimed: false,
            tx_hash: None,
            recipient: None,
            timestamp: None,
            block_number: None,
        }),
    }
}

pub async fn get_merkle_root(
    state: web::Data<AppState>,
) -> impl Responder {
    HttpResponse::Ok().json(MerkleRootResponse {
        merkle_root: state.config.merkle_tree.merkle_root.clone(),
        block_number: 0,
        timestamp: format!("{:?}", SystemTime::now()),
    })
}

pub async fn get_contract_info(
    state: web::Data<AppState>,
) -> impl Responder {
    HttpResponse::Ok().json(ContractInfoResponse {
        network: "optimism".to_string(),
        chain_id: state.config.network.chain_id,
        contracts: ContractsInfo {
            airdrop: ContractDetails {
                address: state.config.network.contracts.airdrop_address.clone(),
                deployed_at: None,
                block_number: None,
            },
            token: TokenDetails {
                address: state.config.network.contracts.token_address.clone(),
                symbol: "ZKP".to_string(),
                decimals: 18,
            },
            relayer_registry: state.config.network.contracts.relayer_registry_address.clone()
                .map(|addr| ContractDetails {
                    address: addr,
                    deployed_at: None,
                    block_number: None,
                }),
        },
        claim_amount: "1000000000000000000000".to_string(),
        claim_deadline: format!("{:?}", SystemTime::now()),
    })
}

pub async fn donate(
    claim: web::Json<DonateRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    info!("Received donation from {}", claim.donor);
    
    let donation_address = state.relayer_address();
    
    HttpResponse::Ok().json(DonateResponse {
        donation_address,
        amount_received: claim.amount.clone(),
        tx_hash: None,
        thank_you: "Thank you for supporting privacy!".to_string(),
    })
}

pub async fn get_stats(
    state: web::Data<AppState>,
) -> impl Responder {
    let stats = state.get_stats().await;
    HttpResponse::Ok().json(stats)
}

pub async fn get_merkle_path(
    state: web::Data<AppState>,
    path: web::Path<String>,
    req: HttpRequest,
) -> impl Responder {
    let address = path.into_inner();

    info!("Getting Merkle path for address: {}", address);

    // Rate limiting
    if state.check_rate_limit(&req, &address, RateLimitType::GetMerklePath).await.is_err() {
        return HttpResponse::TooManyRequests().json(ErrorResponse {
            success: false,
            error: "Rate limit exceeded. Try again later.".to_string(),
            code: Some("RATE_LIMITED".to_string()),
            retry_after: Some(60),
        });
    }

    match state.get_merkle_path(&address).await {
        Some(path_data) => HttpResponse::Ok().json(path_data),
        None => HttpResponse::NotFound().json(ErrorResponse {
            success: false,
            error: "Address not found in Merkle tree.".to_string(),
            code: Some("ADDRESS_NOT_FOUND".to_string()),
            retry_after: None,
        }),
    }
}
