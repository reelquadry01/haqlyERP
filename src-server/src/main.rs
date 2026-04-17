use axum::Router;
use haqly_erp_server::{
    config::settings::Settings,
    db::pool::{create_pool_with_fallback, DbPool, PoolConfig},
    middleware::{auth::AuthLayer, audit::AuditLayer, error::ErrorLayer, rbac::RbacLayer, rate_limit::RateLimitLayer},
    routes::app_routes,
};
use argon2::PasswordHasher;
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
            seed_admin_user(pool).await;
        }
        DbPool::Sqlite(_) => {
            tracing::info!("SQLite migrations already applied during pool initialization.");
        }
    }

    let cors = CorsLayer::new()
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

async fn seed_admin_user(pool: &sqlx::PgPool) {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE email = 'admin@haqly.com')")
        .fetch_one(pool)
        .await
        .unwrap_or(false);

    if exists {
        tracing::info!("Admin user already exists, skipping seed.");
        return;
    }

    let company_id: uuid::Uuid = match sqlx::query_scalar::<_, uuid::Uuid>(
        "SELECT id FROM companies LIMIT 1"
    )
    .fetch_optional(pool)
    .await
    .unwrap_or(None)
    {
        Some(id) => id,
        None => {
            let id = uuid::Uuid::now_v7();
            sqlx::query(
                "INSERT INTO companies (id, name, registration_number, industry, is_active, created_at, updated_at) VALUES ($1, 'HAQLY ERP Demo Company', 'RC-000001', 'technology', true, NOW(), NOW())"
            )
            .bind(id)
            .execute(pool)
            .await
            .ok();
            id
        }
    };

    let salt = argon2::password_hash::SaltString::generate(&mut rand::rngs::OsRng);
    let password_hash = argon2::Argon2::default()
        .hash_password(b"Admin@2026", &salt)
        .expect("Failed to hash admin password")
        .to_string();

    let user_id = uuid::Uuid::now_v7();
    sqlx::query(
        r#"INSERT INTO users (id, company_id, email, password_hash, full_name, is_active, mfa_enabled, created_at, updated_at)
           VALUES ($1, $2, 'admin@haqly.com', $3, 'System Administrator', true, false, NOW(), NOW())"#
    )
    .bind(user_id)
    .bind(company_id)
    .bind(&password_hash)
    .execute(pool)
    .await
    .ok();

    let admin_role_id: Option<uuid::Uuid> = sqlx::query_scalar(
        "SELECT id FROM roles WHERE name = 'SuperAdmin' LIMIT 1"
    )
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    if let Some(role_id) = admin_role_id {
        sqlx::query(
            "INSERT INTO user_roles (user_id, role_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"
        )
        .bind(user_id)
        .bind(role_id)
        .execute(pool)
        .await
        .ok();
    }

    tracing::info!("✅ Seed admin user created: admin@haqly.com / Admin@2026");
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
