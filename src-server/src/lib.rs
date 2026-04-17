pub mod config;
pub mod db;
pub mod models;
pub mod handlers;
pub mod services;
pub mod einvoicing;
pub mod middleware;
pub mod routes;
pub mod dtos;
pub mod security;

pub async fn start_server() -> anyhow::Result<()> {
    use argon2::PasswordHasher;
    use tower_http::cors::CorsLayer;
    use tower_http::trace::TraceLayer;
    use tower_http::compression::CompressionLayer;

    dotenvy::dotenv().ok();

    let settings = config::settings::Settings::load()?;
    let app_data_dir = std::env::var("APP_DATA_DIR")
        .unwrap_or_else(|_| ".".to_string());

    let pool_config = db::pool::PoolConfig::from_settings(
        settings.db_max_connections,
        settings.db_min_connections,
        settings.db_acquire_timeout_secs,
        settings.db_idle_timeout_secs,
        settings.db_max_lifetime_secs,
    );
    let db_pool = db::pool::create_pool_with_fallback(&settings.database_url, &app_data_dir, &pool_config).await?;

    match &db_pool {
        db::pool::DbPool::Postgres(pool) => {
            tracing::info!("Running database migrations (PostgreSQL)...");
            sqlx::migrate!("./src/db/migrations")
                .run(pool)
                .await
                .map_err(|e| {
                    tracing::error!("Migration failed: {}", e);
                    e
                })?;
            tracing::info!("Migrations applied successfully.");

            let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE email = 'admin@haqly.com')")
                .fetch_one(pool)
                .await
                .unwrap_or(false);

            if !exists {
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
                            "INSERT INTO companies (id, code, name, is_active, created_at, updated_at) VALUES ($1, 'HAQLY-DEMO', 'HAQLY ERP Demo Company', true, NOW(), NOW())"
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

                tracing::info!("Seed admin user created: admin@haqly.com / Admin@2026");
            }
        }
        db::pool::DbPool::Sqlite(_) => {
            tracing::info!("SQLite mode — skipping migrations and seed.");
        }
    }

    let pg_pool = match &db_pool {
        db::pool::DbPool::Postgres(p) => p.clone(),
        db::pool::DbPool::Sqlite(_) => {
            anyhow::bail!("SQLite fallback mode is not yet supported for the full Axum route stack.");
        }
    };

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

    let app = routes::app_routes(pg_pool.clone(), settings.clone())
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .layer(middleware::rbac::RbacLayer::new())
        .layer(middleware::auth::AuthLayer::new(settings.rsa_keypair.clone(), settings.jwt_expiration))
        .layer(middleware::audit::AuditLayer::new(pg_pool.clone()))
        .layer(middleware::rate_limit::RateLimitLayer::new())
        .layer(middleware::error::ErrorLayer::new())
        .layer(cors);

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 8100));
    tracing::info!("HAQLY ERP Axum server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
