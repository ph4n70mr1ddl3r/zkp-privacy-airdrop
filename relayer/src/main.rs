mod config;
mod handlers;
mod metrics;
mod state;
mod db;
mod redis;

use actix_web::{App, HttpServer, middleware, web};
use tracing::{info, error};
use tracing_subscriber;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");

    info!("Starting ZKP Airdrop Relayer...");

    let config = config::Config::from_env()?;
    let db_pool = db::create_pool(&config.database_url).await?;
    let redis_client = redis::connect(&config.redis_url).await?;

    let app_state = state::AppState::new(
        config.clone(),
        db_pool,
        redis_client,
    ).await?;

    let bind_address = format!("{}:{}", config.host, config.port);

    info!("Listening on {}", bind_address);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .wrap(middleware::NormalizePath::trim())
            .wrap(
                actix_cors::Cors::permissive()
                    .allowed_origin_fn(|origin, _req_head| {
                        true // Allow all origins for now
                    })
                    .allowed_methods(vec!["GET", "POST", "OPTIONS"])
                    .allowed_headers(vec![
                        actix_web::http::header::AUTHORIZATION,
                        actix_web::http::header::ACCEPT,
                        actix_web::http::header::CONTENT_TYPE,
                    ])
            )
            .service(
                web::scope("/api/v1")
                    .route("/health", web::get().to(handlers::health))
                    .route("/submit-claim", web::post().to(handlers::submit_claim))
                    .route("/check-status/{nullifier}", web::get().to(handlers::check_status))
                    .route("/merkle-root", web::get().to(handlers::get_merkle_root))
                    .route("/contract-info", web::get().to(handlers::get_contract_info))
                    .route("/donate", web::post().to(handlers::donate))
                    .route("/stats", web::get().to(handlers::get_stats))
                    .route("/merkle-path/{address}", web::get().to(handlers::get_merkle_path))
            )
            .route("/metrics", web::get().to(metrics::metrics))
    })
    .bind(&bind_address)?
    .run()
    .await?;

    Ok(())
}
