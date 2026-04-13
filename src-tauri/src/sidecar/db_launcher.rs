// Author: Quadri Atharu
use anyhow::{Context, Result};

pub async fn start_database() -> Result<()> {
    let pg_ready = check_postgres_running();

    if pg_ready {
        tracing::info!("PostgreSQL is already running");
        return Ok(());
    }

    tracing::info!("PostgreSQL is not running; attempting to start via docker-compose");

    let output = std::process::Command::new("docker-compose")
        .args(["up", "-d", "postgres"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .context("docker-compose not found; install Docker Compose or start PostgreSQL manually")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("docker-compose up postgres failed: {stderr}");
    }

    let mut retries = 0u32;
    let max_retries = 30;
    while retries < max_retries {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        if check_postgres_running() {
            tracing::info!("PostgreSQL started successfully after {}s", retries + 1);
            return Ok(());
        }
        retries += 1;
    }

    anyhow::bail!("PostgreSQL did not become ready within {max_retries} seconds")
}

pub async fn run_migrations(database_url: &str) -> Result<()> {
    tracing::info!("Running database migrations against: {database_url}");

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(2)
        .connect(database_url)
        .await
        .context("Failed to connect to database for migrations")?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("Database migrations failed")?;

    tracing::info!("Database migrations completed successfully");
    Ok(())
}

pub async fn verify_connectivity(database_url: &str) -> bool {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(database_url)
        .await;

    match pool {
        Ok(pool) => {
            let result = sqlx::query("SELECT 1")
                .execute(&pool)
                .await;
            result.is_ok()
        }
        Err(e) => {
            tracing::warn!("Database connectivity check failed: {e}");
            false
        }
    }
}

fn check_postgres_running() -> bool {
    let output = std::process::Command::new("pg_isready")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output();

    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let accepting = stdout.contains("accepting connections");
            if accepting {
                return true;
            }
            if cfg!(target_os = "windows") {
                return check_postgres_running_windows();
            }
            false
        }
        Err(_) => {
            if cfg!(target_os = "windows") {
                check_postgres_running_windows()
            } else {
                false
            }
        }
    }
}

fn check_postgres_running_windows() -> bool {
    let output = std::process::Command::new("powershell")
        .args([
            "-Command",
            "Get-Service -Name 'postgresql*' | Where-Object { $_.Status -eq 'Running' }",
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output();

    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            !stdout.trim().is_empty()
        }
        Err(_) => false,
    }
}
