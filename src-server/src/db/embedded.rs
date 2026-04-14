// Author: Quadri Atharu
use rusqlite::Connection;
use std::sync::Arc;
use parking_lot::Mutex;

pub struct SqlitePool {
    conn: Arc<Mutex<Connection>>,
}

impl SqlitePool {
    pub fn new(db_path: &str) -> Result<Self, String> {
        let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON; PRAGMA busy_timeout=5000;"
        ).map_err(|e| e.to_string())?;
        Ok(Self { conn: Arc::new(Mutex::new(conn)) })
    }

    pub fn get(&self) -> Arc<Mutex<Connection>> {
        self.conn.clone()
    }

    pub fn run_migrations(&self) -> Result<(), String> {
        let migration_sql = include_str!("migrations_sqlite/000_all.sql");
        let conn = self.conn.lock();
        conn.execute_batch(migration_sql).map_err(|e| format!("SQLite migration failed: {e}"))?;
        tracing::info!("SQLite migrations applied successfully");
        Ok(())
    }
}
