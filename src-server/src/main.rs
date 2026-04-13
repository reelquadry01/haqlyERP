use axum::Router;
use haqly_erp_server::{
    config::settings::Settings,
    db::pool::create_pool,
    middleware::{auth::AuthLayer, audit::AuditLayer, error::ErrorLayer},
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
    let pool = create_pool(&settings.database_url).await?;

    tracing::info!("Running database migrations...");
    sqlx::migrate!("./src/db/migrations")
        .run(&pool)
        .await
        .map_err(|e| {
            tracing::error!("Migration failed: {}", e);
            e
        })?;
    tracing::info!("Migrations applied successfully.");

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

    let app = app_routes(pool.clone(), settings.clone())
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .layer(AuthLayer::new(settings.jwt_secret.clone(), settings.jwt_expiration))
        .layer(AuditLayer::new(pool.clone()))
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
