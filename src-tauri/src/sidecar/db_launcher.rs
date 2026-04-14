// Author: Quadri Atharu
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

pub struct EmbeddedDbLauncher {
    data_dir: PathBuf,
    pg_bin_dir: PathBuf,
    port: u16,
}

impl EmbeddedDbLauncher {
    pub fn new(app_data_dir: PathBuf) -> Self {
        let data_dir = app_data_dir.join("pgdata");
        let pg_bin_dir = app_data_dir.join("pg-bin");
        Self {
            data_dir,
            pg_bin_dir,
            port: 15432,
        }
    }

    fn initdb_bin(&self) -> PathBuf {
        if cfg!(target_os = "windows") {
            self.pg_bin_dir.join("initdb.exe")
        } else {
            self.pg_bin_dir.join("initdb")
        }
    }

    fn pg_ctl_bin(&self) -> PathBuf {
        if cfg!(target_os = "windows") {
            self.pg_bin_dir.join("pg_ctl.exe")
        } else {
            self.pg_bin_dir.join("pg_ctl")
        }
    }

    fn psql_bin(&self) -> PathBuf {
        if cfg!(target_os = "windows") {
            self.pg_bin_dir.join("psql.exe")
        } else {
            self.pg_bin_dir.join("psql")
        }
    }

    fn createdb_bin(&self) -> PathBuf {
        if cfg!(target_os = "windows") {
            self.pg_bin_dir.join("createdb.exe")
        } else {
            self.pg_bin_dir.join("createdb")
        }
    }

    fn logfile(&self) -> PathBuf {
        self.data_dir.join("postgresql.log")
    }

    fn pid_file(&self) -> PathBuf {
        self.data_dir.join("postmaster.pid")
    }

    pub fn is_initialized(&self) -> bool {
        self.data_dir.join("PG_VERSION").exists()
    }

    pub fn is_running(&self) -> bool {
        let output = Command::new(self.pg_ctl_bin())
            .args([
                "status",
                "-D",
                &self.data_dir.to_string_lossy(),
            ])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output();

        match output {
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                stdout.contains("server is running")
            }
            Err(_) => false,
        }
    }

    pub fn initialize(&self) -> Result<(), String> {
        if self.is_initialized() {
            tracing::info!("Embedded PostgreSQL data directory already initialized");
            return Ok(());
        }

        tracing::info!("Initializing embedded PostgreSQL at: {}", self.data_dir.display());

        if !self.pg_bin_dir.exists() {
            return Err(format!(
                "PostgreSQL binaries not found at {}. Bundle pg-bin directory with the application.",
                self.pg_bin_dir.display()
            ));
        }

        let output = Command::new(self.initdb_bin())
            .args([
                "-D",
                &self.data_dir.to_string_lossy(),
                "-U",
                "haqly",
                "-E",
                "UTF8",
                "--auth=md5",
                "--pwprompt",
            ])
            .env("PGPASSWORD", "haqly")
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .map_err(|e| format!("Failed to run initdb: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("initdb failed: {stderr}"));
        }

        tracing::info!("Embedded PostgreSQL data directory initialized");
        Ok(())
    }

    pub fn initialize_unattended(&self) -> Result<(), String> {
        if self.is_initialized() {
            tracing::info!("Embedded PostgreSQL data directory already initialized");
            return Ok(());
        }

        tracing::info!("Initializing embedded PostgreSQL (unattended) at: {}", self.data_dir.display());

        if !self.pg_bin_dir.exists() {
            return Err(format!(
                "PostgreSQL binaries not found at {}. Bundle pg-bin directory with the application.",
                self.pg_bin_dir.display()
            ));
        }

        let output = Command::new(self.initdb_bin())
            .args([
                "-D",
                &self.data_dir.to_string_lossy(),
                "-U",
                "haqly",
                "-E",
                "UTF8",
                "--auth=trust",
            ])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .map_err(|e| format!("Failed to run initdb: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("initdb failed: {stderr}"));
        }

        let pg_hba_path = self.data_dir.join("pg_hba.conf");
        if pg_hba_path.exists() {
            let pg_hba_content = format!(
                "# HAQLY ERP - Local trust auth\nlocal   all   all   trust\nhost    all   all   127.0.0.1/32   md5\nhost    all   all   ::1/128   md5\n"
            );
            std::fs::write(&pg_hba_path, pg_hba_content)
                .map_err(|e| format!("Failed to write pg_hba.conf: {e}"))?;
        }

        tracing::info!("Embedded PostgreSQL initialized with trust auth (will set password after start)");
        Ok(())
    }

    pub fn start(&self) -> Result<u32, String> {
        if self.is_running() {
            tracing::info!("Embedded PostgreSQL is already running");
            return self.read_pid();
        }

        tracing::info!("Starting embedded PostgreSQL on port {}", self.port);

        let port_str = format!("-p{}", self.port);

        let output = Command::new(self.pg_ctl_bin())
            .args([
                "-D",
                &self.data_dir.to_string_lossy(),
                "-l",
                &self.logfile().to_string_lossy(),
                "-o",
                &port_str,
                "-w",
                "start",
            ])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .map_err(|e| format!("Failed to run pg_ctl: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("pg_ctl start failed: {stderr}"));
        }

        self.wait_for_ready()?;

        self.create_database_if_needed()?;

        self.set_password()?;

        let pid = self.read_pid()?;
        tracing::info!("Embedded PostgreSQL started with PID {pid} on port {}", self.port);
        Ok(pid)
    }

    fn wait_for_ready(&self) -> Result<(), String> {
        let max_retries = 30u32;
        for i in 0..max_retries {
            let output = Command::new(self.psql_bin())
                .args([
                    "-U",
                    "haqly",
                    "-h",
                    "localhost",
                    "-p",
                    &self.port.to_string(),
                    "-d",
                    "postgres",
                    "-c",
                    "SELECT 1",
                ])
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .output();

            if let Ok(o) = output {
                if o.status.success() {
                    tracing::info!("Embedded PostgreSQL ready after {} retries", i);
                    return Ok(());
                }
            }

            std::thread::sleep(Duration::from_secs(1));
        }

        Err("Embedded PostgreSQL did not become ready within 30 seconds".to_string())
    }

    fn create_database_if_needed(&self) -> Result<(), String> {
        let output = Command::new(self.createdb_bin())
            .args([
                "-U",
                "haqly",
                "-h",
                "localhost",
                "-p",
                &self.port.to_string(),
                "haqly_db",
            ])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .map_err(|e| format!("Failed to run createdb: {e}"))?;

        if output.status.success() {
            tracing::info!("Created haqly_db database");
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("already exists") {
                tracing::info!("Database haqly_db already exists");
            } else {
                tracing::warn!("createdb warning: {stderr}");
            }
        }

        Ok(())
    }

    fn set_password(&self) -> Result<(), String> {
        let output = Command::new(self.psql_bin())
            .args([
                "-U",
                "haqly",
                "-h",
                "localhost",
                "-p",
                &self.port.to_string(),
                "-d",
                "haqly_db",
                "-c",
                "ALTER USER haqly WITH PASSWORD 'haqly';",
            ])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .map_err(|e| format!("Failed to set password: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::warn!("Failed to set haqly user password: {stderr}");
        }

        let pg_hba_path = self.data_dir.join("pg_hba.conf");
        if pg_hba_path.exists() {
            let pg_hba_content = format!(
                "# HAQLY ERP - MD5 auth\nlocal   all   all   md5\nhost    all   all   127.0.0.1/32   md5\nhost    all   all   ::1/128   md5\n"
            );
            std::fs::write(&pg_hba_path, pg_hba_content)
                .map_err(|e| format!("Failed to update pg_hba.conf: {e}"))?;
        }

        let reload_output = Command::new(self.pg_ctl_bin())
            .args([
                "-D",
                &self.data_dir.to_string_lossy(),
                "reload",
            ])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output();

        if let Ok(o) = reload_output {
            if o.status.success() {
                tracing::info!("PostgreSQL reloaded with md5 authentication");
            }
        }

        Ok(())
    }

    fn read_pid(&self) -> Result<u32, String> {
        let pid_path = self.pid_file();
        if !pid_path.exists() {
            return Err("PID file not found".to_string());
        }

        let content = std::fs::read_to_string(&pid_path)
            .map_err(|e| format!("Failed to read PID file: {e}"))?;

        content
            .lines()
            .next()
            .and_then(|line| line.trim().parse::<u32>().ok())
            .ok_or_else(|| "Failed to parse PID from postmaster.pid".to_string())
    }

    pub fn stop(&self) -> Result<(), String> {
        if !self.is_running() {
            tracing::info!("Embedded PostgreSQL is not running");
            return Ok(());
        }

        tracing::info!("Stopping embedded PostgreSQL");

        let output = Command::new(self.pg_ctl_bin())
            .args([
                "-D",
                &self.data_dir.to_string_lossy(),
                "-m",
                "fast",
                "stop",
            ])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .map_err(|e| format!("Failed to run pg_ctl stop: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("no server running") {
                tracing::info!("Embedded PostgreSQL was not running");
                return Ok(());
            }
            return Err(format!("pg_ctl stop failed: {stderr}"));
        }

        tracing::info!("Embedded PostgreSQL stopped");
        Ok(())
    }

    pub fn get_connection_string(&self) -> String {
        format!(
            "postgresql://haqly:haqly@localhost:{}/haqly_db",
            self.port
        )
    }

    pub fn get_port(&self) -> u16 {
        self.port
    }

    pub async fn run_migrations(&self) -> Result<(), String> {
        let conn_str = self.get_connection_string();
        tracing::info!("Running database migrations against: {conn_str}");

        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(2)
            .connect(&conn_str)
            .await
            .map_err(|e| format!("Failed to connect for migrations: {e}"))?;

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(|e| format!("Database migrations failed: {e}"))?;

        tracing::info!("Database migrations completed successfully");
        Ok(())
    }
}

impl Drop for EmbeddedDbLauncher {
    fn drop(&mut self) {
        if self.is_running() {
            let _ = self.stop();
        }
    }
}
