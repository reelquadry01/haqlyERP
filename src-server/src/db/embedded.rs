// Author: Quadri Atharu
// SQLite fallback for desktop deployment (no PostgreSQL required)
// Uses sqlx::SqlitePool to avoid linking conflict with rusqlite

use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use std::path::Path;

pub struct EmbeddedDb {
    pool: SqlitePool,
}

impl EmbeddedDb {
    pub async fn new(db_path: &str) -> Result<Self, String> {
        let db_url = if Path::new(db_path).exists() {
            format!("sqlite:{}?mode=rw", db_path)
        } else {
            format!("sqlite:{}?mode=rwc", db_path)
        };

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await
            .map_err(|e| format!("SQLite connection failed: {e}"))?;

        sqlx::query("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON; PRAGMA busy_timeout=5000;")
            .execute(&pool)
            .await
            .map_err(|e| format!("PRAGMA failed: {e}"))?;

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub async fn run_migrations(&self) -> Result<(), String> {
        let migration_sql = include_str!("migrations_sqlite/000_all.sql");
        for stmt in migration_sql.split(';') {
            let stmt = stmt.trim();
            if stmt.is_empty() || stmt.starts_with("--") {
                continue;
            }
            sqlx::query(stmt)
                .execute(&self.pool)
                .await
                .map_err(|e| format!("SQLite migration statement failed: {e}\nSQL: {stmt}"))?;
        }
        tracing::info!("SQLite migrations applied successfully");
        Ok(())
    }
}
