// Author: Quadri Atharu
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;

use crate::db::embedded::SqlitePool;

pub enum DbPool {
    Postgres(PgPool),
    Sqlite(SqlitePool),
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

    pub fn sqlite_pool(&self) -> Option<&SqlitePool> {
        match self {
            DbPool::Postgres(_) => None,
            DbPool::Sqlite(s) => Some(s),
        }
    }
}

pub async fn create_pool(database_url: &str) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(30))
        .connect(database_url)
        .await?;

    sqlx::query("SELECT 1").execute(&pool).await?;

    tracing::info!("Database connection pool established and verified");
    Ok(pool)
}

pub async fn create_pool_with_fallback(
    database_url: &str,
    app_data_dir: &str,
) -> anyhow::Result<DbPool> {
    match try_postgres_pool(database_url).await {
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
            let sqlite_pool = SqlitePool::new(&db_path)?;
            sqlite_pool.run_migrations()?;
            tracing::info!("Database mode: SQLite (single-user fallback)");
            Ok(DbPool::Sqlite(sqlite_pool))
        }
    }
}

async fn try_postgres_pool(database_url: &str) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(10))
        .connect(database_url)
        .await?;

    sqlx::query("SELECT 1").execute(&pool).await?;

    Ok(pool)
}
