use axum::Router;
use haqly_erp_server::{
    config::settings::Settings,
    db::pool::{create_pool_with_fallback, DbPool, PoolConfig},
    middleware::{auth::AuthLayer, audit::AuditLayer, error::ErrorLayer, rbac::RbacLayer, rate_limit::RateLimitLayer},
    routes::app_routes,
};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tower_http::compression::CompressionLayer;
use tokio::signal;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("haqly_erp_server=debug,tower_http=debug,sqlx=warn")),
        )
        .with_file(true)
        .with_line_number(true)
        .json()
        .init();

    tracing::info!("HAQLY ERP Server starting...");

    let settings = Settings::load()?;
    let app_data_dir = std::env::var("APP_DATA_DIR")
        .unwrap_or_else(|_| ".".to_string());

    let pool_config = PoolConfig::from_settings(
        settings.db_max_connections,
        settings.db_min_connections,
        settings.db_acquire_timeout_secs,
        settings.db_idle_timeout_secs,
        settings.db_max_lifetime_secs,
    );
    let db_pool = create_pool_with_fallback(&settings.database_url, &app_data_dir, &pool_config).await?;

    match &db_pool {
        DbPool::Postgres(pool) => {
            tracing::info!("Running database migrations (PostgreSQL)...");
            sqlx::migrate!("./src/db/migrations")
                .run(pool)
                .await
                .map_err(|e| {
                    tracing::error!("Migration failed: {}", e);
                    e
                })?;
            tracing::info!("Migrations applied successfully.");
        }
        DbPool::Sqlite(_) => {
            tracing::info!("SQLite migrations already applied during pool initialization.");
        }
    }

    let cors = CorsLayer::permissive()
        .allow_origin(
            settings
                .cors_origins
                .iter()
                .map(|o| o.parse::<axum::http::HeaderValue>().unwrap_or_else(|_| {
                    tracing::warn!("Invalid CORS origin: {}", o);
                    axum::http::HeaderValue::from_static("")
                }))
                .collect::<Vec<_>>(),
        )
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::PATCH,
            axum::http::Method::DELETE,
        ])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
            axum::http::header::ACCEPT,
            axum::http::header::HeaderName::from_static("x-request-id"),
            axum::http::header::HeaderName::from_static("x-company-id"),
        ])
        .allow_credentials(true);

    let pg_pool_for_routes = match &db_pool {
        DbPool::Postgres(p) => p.clone(),
        DbPool::Sqlite(_) => {
            anyhow::bail!(
                "SQLite fallback mode is not yet supported for the full Axum route stack. \
                 Please configure PostgreSQL or bundle embedded PostgreSQL binaries."
            );
        }
    };

    let app = app_routes(pg_pool_for_routes.clone(), settings.clone())
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .layer(RbacLayer::new())
        .layer(AuthLayer::new(settings.rsa_keypair.clone(), settings.jwt_expiration))
        .layer(AuditLayer::new(pg_pool_for_routes.clone()))
        .layer(RateLimitLayer::new())
        .layer(ErrorLayer::new())
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], settings.server_port));
    tracing::info!("HAQLY ERP Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("HAQLY ERP Server shut down gracefully.");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C, shutting down...");
        },
        _ = terminate => {
            tracing::info!("Received SIGTERM, shutting down...");
        },
    }
}
