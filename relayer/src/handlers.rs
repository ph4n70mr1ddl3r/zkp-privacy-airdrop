use actix_web::{web, HttpRequest, HttpResponse, Responder};
use ethers::types::Address;
use num_bigint::BigUint;
use num_traits::Num;
use once_cell::sync::Lazy;
use regex::Regex;
use std::str::FromStr;
use std::sync::OnceLock;
use tracing::{error, info, warn};

/// BN254 scalar field modulus
const BN254_FIELD_MODULUS: &str =
    "21888242871839275222246405745257275088548364400416034343698204186575808495617";

static FIELD_MODULUS: OnceLock<BigUint> = OnceLock::new();

fn get_field_modulus() -> &'static BigUint {
    FIELD_MODULUS.get_or_init(|| {
        BigUint::from_str_radix(BN254_FIELD_MODULUS, 10).expect("Invalid field modulus constant")
    })
}

/// Compiled regex patterns for sensitive data filtering (pre-compiled for performance)
static SENSITIVE_PATTERNS: Lazy<Vec<(Regex, &'static str)>> = Lazy::new(|| {
    vec![
        (Regex::new("(?i)private_key").unwrap(), "private key"),
        (Regex::new("(?i)private-key").unwrap(), "private key"),
        (Regex::new("(?i)privatekey").unwrap(), "private key"),
        (Regex::new("(?i)priv_key").unwrap(), "private key"),
        (Regex::new("(?i)privkey").unwrap(), "private key"),
        (Regex::new("(?i)0x[0-9a-f]{32,}").unwrap(), "hex credential"),
        (Regex::new("(?i)seed").unwrap(), "seed"),
        (Regex::new("(?i)mnemonic").unwrap(), "mnemonic"),
        (Regex::new("(?i)credentials").unwrap(), "credentials"),
        (Regex::new("(?i)api_key").unwrap(), "API key"),
        (Regex::new("(?i)apikey").unwrap(), "API key"),
        (Regex::new("(?i)access_token").unwrap(), "access token"),
        (Regex::new("(?i)refresh_token").unwrap(), "refresh token"),
        (Regex::new("(?i)access_key").unwrap(), "access key"),
        (Regex::new("(?i)secret").unwrap(), "secret value"),
        (Regex::new("(?i)secret_key").unwrap(), "secret key"),
        (Regex::new("(?i)session").unwrap(), "session token"),
        (Regex::new("(?i)signature").unwrap(), "signature"),
        (Regex::new("(?i)auth_token").unwrap(), "auth token"),
        (Regex::new("(?i)password").unwrap(), "password"),
        (Regex::new("(?i)passwd").unwrap(), "password"),
        (Regex::new("(?i)pwd").unwrap(), "password"),
        (
            Regex::new("(?i)authorization").unwrap(),
            "authorization header",
        ),
        (Regex::new("(?i)bearer").unwrap(), "bearer token"),
        (
            Regex::new("(?i)0x[0-9a-f]{64,}").unwrap(),
            "private key or hash",
        ),
        (
            Regex::new(r"(?i)pk\s*=\s*[0-9a-f]+").unwrap(),
            "private key param",
        ),
        (Regex::new(r"(?i)key\s*=\s*[0-9a-f]+").unwrap(), "key param"),
        (
            Regex::new(r"(?i)token\s*=\s*[a-z0-9\-._~+/]+=*").unwrap(),
            "token param",
        ),
    ]
});

use crate::state::AppState;
use crate::types_plonk::*;

/// Validates claim input parameters and returns an error response if invalid
fn validate_claim_input(
    claim: &SubmitClaimRequest,
    expected_merkle_root: &str,
) -> Result<(), HttpResponse> {
    if !is_valid_nullifier(&claim.nullifier) {
        warn!("Invalid nullifier format: {}", claim.nullifier);
        return Err(HttpResponse::BadRequest().json(ErrorResponse {
            success: false,
            error: "Invalid nullifier format. Expected 66-character hex string starting with 0x."
                .to_string(),
            code: Some("INVALID_NULLIFIER".to_string()),
            retry_after: None,
        }));
    }

    if !is_valid_address(&claim.recipient) {
        warn!("Invalid recipient address: {}", claim.recipient);
        return Err(HttpResponse::BadRequest().json(ErrorResponse {
            success: false,
            error: "Invalid Ethereum address format for recipient.".to_string(),
            code: Some("INVALID_ADDRESS".to_string()),
            retry_after: None,
        }));
    }

    if !is_valid_merkle_root(&claim.merkle_root) {
        warn!("Invalid merkle_root format: {}", claim.merkle_root);
        return Err(HttpResponse::BadRequest().json(ErrorResponse {
            success: false,
            error: "Invalid merkle_root format. Expected 66-character hex string starting with 0x."
                .to_string(),
            code: Some("INVALID_MERKLE_ROOT".to_string()),
            retry_after: None,
        }));
    }

    if claim.merkle_root != expected_merkle_root {
        warn!(
            "Merkle root mismatch: provided={}, expected={}",
            claim.merkle_root, expected_merkle_root
        );
        return Err(HttpResponse::BadRequest().json(ErrorResponse {
            success: false,
            error: "Merkle root does not match the current airdrop root. Please ensure you are using the latest merkle tree.".to_string(),
            code: Some("MERKLE_ROOT_MISMATCH".to_string()),
            retry_after: None,
        }));
    }

    Ok(())
}

fn sanitize_error_message(error: &str) -> String {
    let lower = error.to_lowercase();

    for (re, description) in SENSITIVE_PATTERNS.iter() {
        if re.is_match(&lower) {
            tracing::warn!(
                "Filtered sensitive error message containing {}: error='{}'",
                description,
                error
            );
            return "Internal error occurred. Check logs for details.".to_string();
        }
    }

    error.to_string()
}

fn is_valid_hex_string(input: &str, expected_len: usize) -> bool {
    input.len() == expected_len
        && (input.starts_with("0x") || input.starts_with("0X"))
        && hex::decode(&input[2..]).is_ok()
}

#[allow(dead_code)]
fn is_valid_field_element(hex_str: &str) -> bool {
    if !hex_str.starts_with("0x") && !hex_str.starts_with("0X") {
        return false;
    }

    let hex_value = match hex::decode(&hex_str[2..]) {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };

    if hex_value.len() != 32 {
        return false;
    }

    let value = BigUint::from_bytes_be(&hex_value);
    let field_modulus = get_field_modulus();

    value < *field_modulus
}

pub fn is_valid_address(address: &str) -> bool {
    if !is_valid_hex_string(address, 42) {
        return false;
    }
    if hex::decode(&address[2..]).is_err() {
        return false;
    }
    Address::from_str(address).is_ok()
}

pub fn is_valid_nullifier(nullifier: &str) -> bool {
    is_valid_hex_bytes(nullifier, 66, false)
}

pub fn is_valid_merkle_root(merkle_root: &str) -> bool {
    is_valid_hex_bytes(merkle_root, 66, true)
}

fn is_valid_hex_bytes(input: &str, expected_len: usize, reject_zero: bool) -> bool {
    if !is_valid_hex_string(input, expected_len) {
        return false;
    }

    let hex = &input[2..];
    if let Ok(bytes) = hex::decode(hex) {
        if bytes.len() == 32 {
            if !reject_zero {
                return true;
            }
            let zero_value = [0u8; 32];
            return bytes != zero_value;
        }
    }

    false
}

pub async fn health(state: web::Data<AppState>) -> impl Responder {
    let relayer_address = match state.relayer_address() {
        Ok(addr) => addr,
        Err(e) => {
            error!("Failed to get relayer address: {}", e);
            return HttpResponse::InternalServerError().json(HealthResponse {
                status: "unhealthy".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                services: Services {
                    database: "unknown".to_string(),
                    redis: "unknown".to_string(),
                    optimism_node: "unknown".to_string(),
                    relayer_wallet: RelayerWalletInfo {
                        address: "0x0000000000000000000000000000000000000000".to_string(),
                        balance: "0".to_string(),
                        sufficient: false,
                    },
                    merkle_tree: "unknown".to_string(),
                },
            });
        }
    };

    let services = Services {
        database: state.get_db_status().await.to_string(),
        redis: state.get_redis_status().await.to_string(),
        optimism_node: state.get_node_status().await.to_string(),
        relayer_wallet: RelayerWalletInfo {
            address: relayer_address,
            balance: state.get_relayer_balance().await.to_string(),
            sufficient: state.has_sufficient_balance().await,
        },
        merkle_tree: if state.check_merkle_tree().await {
            "valid".to_string()
        } else {
            "invalid".to_string()
        },
    };

    let status = if state.is_healthy().await {
        "healthy".to_string()
    } else {
        "unhealthy".to_string()
    };

    let http_status = if status == "healthy" {
        actix_web::http::StatusCode::OK
    } else {
        actix_web::http::StatusCode::SERVICE_UNAVAILABLE
    };

    HttpResponse::build(http_status).json(HealthResponse {
        status,
        timestamp: chrono::Utc::now().to_rfc3339(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        services,
    })
}

pub async fn submit_claim(
    _req: HttpRequest,
    state: web::Data<AppState>,
    claim: web::Json<SubmitClaimRequest>,
) -> impl Responder {
    if let Err(response) = validate_claim_input(&claim, &state.config.merkle_tree.merkle_root) {
        return response;
    }

    info!(
        "Received {} claim submission from nullifier: {}",
        claim.proof.type_name(),
        claim.nullifier
    );

    // Rate limiting check
    if let Err(e) = state
        .check_rate_limit(&claim.nullifier, RateLimitType::SubmitClaim)
        .await
    {
        warn!("Rate limit exceeded: {}", e);
        return HttpResponse::TooManyRequests().json(ErrorResponse {
            success: false,
            error: "Rate limit exceeded. Try again later.".to_string(),
            code: Some("RATE_LIMITED".to_string()),
            retry_after: Some(60),
        });
    }

    if !claim.proof.is_valid_structure() {
        warn!(
            "Invalid {} proof structure from nullifier: {}",
            claim.proof.type_name(),
            claim.nullifier
        );
        let error_code = if claim.proof.type_name() == "Plonk" {
            "PLONK_FORMAT_ERROR"
        } else {
            "INVALID_PROOF"
        };
        let error_message = if claim.proof.type_name() == "Plonk" {
            "Plonk proof format is invalid. Expected at least 8 field elements.".to_string()
        } else {
            format!(
                "The provided {} proof is invalid. Please regenerate proof with correct inputs.",
                claim.proof.type_name()
            )
        };
        return HttpResponse::BadRequest().json(ErrorResponse {
            success: false,
            error: error_message,
            code: Some(error_code.to_string()),
            retry_after: None,
        });
    }

    info!("Validated {} proof successfully", claim.proof.type_name());

    // Plonk-specific warning
    if claim.proof.type_name() == "Plonk" {
        info!("Plonk proof detected - verification gas estimate: ~1.3M");
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
            info!(
                "Claim submitted successfully ({}): {}",
                claim.proof.type_name(),
                tx_hash
            );
            HttpResponse::Ok().json(SubmitClaimResponse {
                success: true,
                tx_hash: Some(tx_hash),
                status: Some("pending".to_string()),
                estimated_confirmation: Some(chrono::Utc::now().to_rfc3339()),
                error: None,
                code: None,
            })
        }
        Err(e) => {
            if e.contains("already been used") {
                warn!("Nullifier already claimed: {}", claim.nullifier);
                return HttpResponse::BadRequest().json(ErrorResponse {
                    success: false,
                    error: e,
                    code: Some("ALREADY_CLAIMED".to_string()),
                    retry_after: None,
                });
            }
            error!("Failed to submit claim: {} - Error: {}", claim.nullifier, e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                success: false,
                error: sanitize_error_message(&format!("Failed to submit claim: {}", e)),
                code: Some("INTERNAL_ERROR".to_string()),
                retry_after: Some(60),
            })
        }
    }
}

pub async fn check_status(
    _req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    let nullifier = path.into_inner();

    if !is_valid_nullifier(&nullifier) {
        warn!("Invalid nullifier format: {}", nullifier);
        return HttpResponse::BadRequest().json(ErrorResponse {
            success: false,
            error: "Invalid nullifier format. Expected 66-character hex string starting with 0x."
                .to_string(),
            code: Some("INVALID_NULLIFIER".to_string()),
            retry_after: None,
        });
    }

    info!("Checking status for nullifier: {}", nullifier);

    if let Err(e) = state
        .check_rate_limit(&nullifier, RateLimitType::CheckStatus)
        .await
    {
        warn!("Rate limit exceeded for check_status: {}", e);
        return HttpResponse::TooManyRequests().json(ErrorResponse {
            success: false,
            error: "Rate limit exceeded. Try again later.".to_string(),
            code: Some("RATE_LIMITED".to_string()),
            retry_after: Some(60),
        });
    }

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

pub async fn get_merkle_root(state: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().json(MerkleRootResponse {
        merkle_root: state.config.merkle_tree.merkle_root.clone(),
        block_number: 0,
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

pub async fn get_contract_info(state: web::Data<AppState>) -> impl Responder {
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
            relayer_registry: state
                .config
                .network
                .contracts
                .relayer_registry_address
                .clone()
                .map(|addr| ContractDetails {
                    address: addr,
                    deployed_at: None,
                    block_number: None,
                }),
        },
        claim_amount: "1000000000000000000000".to_string(),
        claim_deadline: chrono::Utc::now().to_rfc3339(),
    })
}

pub async fn donate(
    claim: web::Json<DonateRequest>,
    _state: web::Data<AppState>,
) -> impl Responder {
    info!(
        "Donation request from {} amount: {}",
        claim.donor, claim.amount
    );

    HttpResponse::BadRequest().json(ErrorResponse {
        success: false,
        error: "Direct donations via API are not supported. Please send ETH/OPT directly to the relayer wallet address displayed on the health check endpoint.".to_string(),
        code: Some("DONATIONS_NOT_SUPPORTED".to_string()),
        retry_after: None,
    })
}

pub async fn get_stats(state: web::Data<AppState>) -> impl Responder {
    let response_stats = state.get_stats().await;
    HttpResponse::Ok().json(response_stats)
}

pub async fn get_merkle_path(
    _req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    let address = path.into_inner();

    if !is_valid_address(&address) {
        warn!("Invalid address format: {}", address);
        return HttpResponse::BadRequest().json(ErrorResponse {
            success: false,
            error: "Invalid Ethereum address format.".to_string(),
            code: Some("INVALID_ADDRESS".to_string()),
            retry_after: None,
        });
    }

    info!("Getting Merkle path for address: {}", address);

    // Rate limiting
    if state
        .check_rate_limit(&address, RateLimitType::GetMerklePath)
        .await
        .is_err()
    {
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
