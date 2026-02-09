mod config;
mod db;
mod handlers;
mod metrics;
mod redis;
mod state;
mod types_plonk;

use actix_web::http::header::HeaderName;
use actix_web::{middleware, web, App, HttpServer, HttpResponse};
use std::sync::Arc;
use tokio::signal;
use tracing::info;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");

    info!("Starting ZKP Airdrop Relayer...");

    let config = config::Config::from_env()?;
    config.validate()?;
    let db_pool = db::create_pool(&config.database_url).await?;
    let redis_client = redis::connect(&config.redis_url).await?;

    let bind_address = format!("{}:{}", config.host, config.port);

    let allowed_origins = Arc::new(config.cors.allowed_origins.clone());

    let app_state = state::AppState::new(config.clone(), db_pool, redis_client).await?;

    info!("Listening on {}", bind_address);
    info!("CORS allowed origins: {:?}", config.cors.allowed_origins);

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .wrap(middleware::NormalizePath::trim())
            .app_data(
                web::JsonConfig::default()
                    .limit(1024 * 1024)
                    .error_handler(|err, _| {
                        actix_web::error::InternalError::from_response(
                            err,
                            HttpResponse::PayloadTooLarge().json(serde_json::json!({
                                "success": false,
                                "error": "Request payload too large. Maximum size is 1MB.",
                                "code": "PAYLOAD_TOO_LARGE"
                            })),
                        )
                        .into()
                    })
            )
            .wrap({
                let allowed_origins = Arc::clone(&allowed_origins);
                actix_cors::Cors::default()
                    .allowed_origin_fn(move |origin, _req_head| {
                        if let Ok(origin_str) = origin.to_str() {
                            allowed_origins
                                .iter()
                                .any(|allowed| allowed == "*" || origin_str == *allowed)
                        } else {
                            false
                        }
                    })
                    .allowed_methods(
                        config
                            .cors
                            .allowed_methods
                            .iter()
                            .map(|s| s.as_str())
                            .collect::<Vec<_>>(),
                    )
                    .allowed_headers(
                        config
                            .cors
                            .allowed_headers
                            .iter()
                            .filter_map(|h| match HeaderName::from_bytes(h.as_bytes()) {
                                Ok(header_name) => Some(header_name),
                                Err(_) => {
                                    tracing::warn!("Invalid header name: {}", h);
                                    None
                                }
                            })
                            .collect::<Vec<_>>(),
                    )
                    .max_age(config.cors.max_age)
                    .supports_credentials()
                    .expose_any_header()
            })
            .service(
                web::scope("/api/v1")
                    .route("/health", web::get().to(handlers::health))
                    .route("/submit-claim", web::post().to(handlers::submit_claim))
                    .route(
                        "/check-status/{nullifier}",
                        web::get().to(handlers::check_status),
                    )
                    .route("/merkle-root", web::get().to(handlers::get_merkle_root))
                    .route("/contract-info", web::get().to(handlers::get_contract_info))
                    .route("/donate", web::post().to(handlers::donate))
                    .route("/stats", web::get().to(handlers::get_stats))
                    .route(
                        "/merkle-path/{address}",
                        web::get().to(handlers::get_merkle_path),
                    ),
            )
            .route("/metrics", web::get().to(metrics::metrics))
    })
    .bind(&bind_address)?
    .run();

    let handle = server.handle();

    tokio::spawn(async move {
        let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate()).unwrap();
        let mut sigint = signal::unix::signal(signal::unix::SignalKind::interrupt()).unwrap();

        tokio::select! {
            _ = sigterm.recv() => {
                info!("Received SIGTERM, shutting down gracefully...");
            }
            _ = sigint.recv() => {
                info!("Received SIGINT, shutting down gracefully...");
            }
        }

        info!("Stopping server gracefully...");
        let _ = handle.stop(true).await;
    });

    server.await?;

    Ok(())
}
