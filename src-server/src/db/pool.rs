// Author: Quadri Atharu
use sqlx::postgres::PgPoolOptions;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{PgPool, SqlitePool};
use std::time::Duration;

use crate::db::embedded::EmbeddedDb;

pub struct PoolConfig {
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 10,
            min_connections: 2,
            acquire_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600),
            max_lifetime: Duration::from_secs(1800),
        }
    }
}

impl PoolConfig {
    pub fn from_settings(
        max: u32,
        min: u32,
        acquire_secs: u64,
        idle_secs: u64,
        lifetime_secs: u64,
    ) -> Self {
        Self {
            max_connections: max,
            min_connections: min,
            acquire_timeout: Duration::from_secs(acquire_secs),
            idle_timeout: Duration::from_secs(idle_secs),
            max_lifetime: Duration::from_secs(lifetime_secs),
        }
    }
}

pub enum DbPool {
    Postgres(PgPool),
    Sqlite(EmbeddedDb),
}

impl DbPool {
    pub fn is_postgres(&self) -> bool {
        matches!(self, DbPool::Postgres(_))
    }

    pub fn is_sqlite(&self) -> bool {
        matches!(self, DbPool::Sqlite(_))
    }

    pub fn pg_pool(&self) -> Option<&PgPool> {
        match self {
            DbPool::Postgres(p) => Some(p),
            DbPool::Sqlite(_) => None,
        }
    }

    pub fn sqlite_pool(&self) -> Option<&EmbeddedDb> {
        match self {
            DbPool::Postgres(_) => None,
            DbPool::Sqlite(s) => Some(s),
        }
    }
}

pub async fn create_pool(database_url: &str, config: &PoolConfig) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(config.acquire_timeout)
        .idle_timeout(config.idle_timeout)
        .max_lifetime(config.max_lifetime)
        .connect(database_url)
        .await?;

    sqlx::query("SELECT 1").execute(&pool).await?;

    tracing::info!("Database connection pool established and verified (max={}, min={})", config.max_connections, config.min_connections);
    Ok(pool)
}

pub async fn create_sqlite_pool(database_url: &str, config: &PoolConfig) -> anyhow::Result<SqlitePool> {
    let pool = SqlitePoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(config.acquire_timeout)
        .idle_timeout(config.idle_timeout)
        .max_lifetime(config.max_lifetime)
        .connect(database_url)
        .await?;

    sqlx::query("SELECT 1").execute(&pool).await?;

    tracing::info!("SQLite connection pool established and verified (max={}, min={})", config.max_connections, config.min_connections);
    Ok(pool)
}

pub async fn create_pool_with_fallback(
    database_url: &str,
    app_data_dir: &str,
    config: &PoolConfig,
) -> anyhow::Result<DbPool> {
    match try_postgres_pool(database_url, config).await {
        Ok(pool) => {
            tracing::info!("Database mode: PostgreSQL (full multi-user support)");
            Ok(DbPool::Postgres(pool))
        }
        Err(e) => {
            tracing::warn!(
                "PostgreSQL connection failed ({}): falling back to SQLite single-user mode",
                e
            );
            let db_path = format!("{}/haqly_erp.sqlite", app_data_dir);
            let embedded = EmbeddedDb::new(&db_path).await.map_err(|e| anyhow::anyhow!(e))?;
            embedded.run_migrations().await.map_err(|e| anyhow::anyhow!(e))?;
            tracing::info!("Database mode: SQLite (single-user fallback)");
            Ok(DbPool::Sqlite(embedded))
        }
    }
}

async fn try_postgres_pool(database_url: &str, config: &PoolConfig) -> anyhow::Result<PgPool> {
    let fallback_config = PoolConfig {
        max_connections: config.min_connections,
        min_connections: config.min_connections,
        acquire_timeout: Duration::from_secs(10),
        idle_timeout: config.idle_timeout,
        max_lifetime: config.max_lifetime,
    };
    let pool = PgPoolOptions::new()
        .max_connections(fallback_config.max_connections)
        .min_connections(fallback_config.min_connections)
        .acquire_timeout(fallback_config.acquire_timeout)
        .idle_timeout(fallback_config.idle_timeout)
        .max_lifetime(fallback_config.max_lifetime)
        .connect(database_url)
        .await?;

    sqlx::query("SELECT 1").execute(&pool).await?;

    Ok(pool)
}
