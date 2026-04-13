// Author: Quadri Atharu
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};

use crate::sidecar::python_engine;

#[derive(Clone)]
pub struct SidecarManager {
    backend_running: Arc<AtomicBool>,
    ai_running: Arc<AtomicBool>,
    backend_handle: Arc<Mutex<Option<std::process::Child>>>,
    ai_handle: Arc<Mutex<Option<std::process::Child>>>,
}

impl SidecarManager {
    pub fn new() -> Self {
        Self {
            backend_running: Arc::new(AtomicBool::new(false)),
            ai_running: Arc::new(AtomicBool::new(false)),
            backend_handle: Arc::new(Mutex::new(None)),
            ai_handle: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn start_all(&self) -> Result<()> {
        tracing::info!("Starting all sidecar services");

        if let Err(e) = self.start_backend().await {
            tracing::error!("Failed to start backend: {e}");
        }

        if let Err(e) = self.start_ai_engine().await {
            tracing::error!("Failed to start AI engine: {e}");
        }

        Ok(())
    }

    pub async fn stop_all(&self) -> Result<()> {
        tracing::info!("Stopping all sidecar services");

        self.stop_backend()?;
        self.stop_ai_engine()?;

        Ok(())
    }

    pub async fn health_check(&self, service: &str) -> bool {
        let url = match service {
            "backend" => "http://localhost:8100/health",
            "ai" => "http://localhost:8200/health",
            _ => return false,
        };

        reqwest::Client::new()
            .get(url)
            .timeout(std::time::Duration::from_secs(3))
            .send()
            .await
            .is_ok()
    }

    pub async fn start_backend(&self) -> Result<()> {
        if self.backend_running.load(Ordering::Relaxed) && self.health_check("backend").await {
            tracing::info!("Backend is already running");
            return Ok(());
        }

        let binary_path = locate_backend_binary()?;

        tracing::info!("Starting backend: {}", binary_path.display());

        let child = std::process::Command::new(&binary_path)
            .env("PORT", "8100")
            .env("RUST_LOG", "info")
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .with_context(|| format!("Failed to spawn backend: {}", binary_path.display()))?;

        *self.backend_handle.lock().unwrap() = Some(child);
        self.backend_running.store(true, Ordering::Relaxed);

        let mut retries = 0u32;
        let max_retries = 30;
        while retries < max_retries {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            if self.health_check("backend").await {
                tracing::info!("Backend is healthy after {} retries", retries);
                return Ok(());
            }
            retries += 1;
        }

        tracing::warn!("Backend started but health check did not pass within timeout");
        Ok(())
    }

    pub async fn start_ai_engine(&self) -> Result<()> {
        if self.ai_running.load(Ordering::Relaxed) && self.health_check("ai").await {
            tracing::info!("AI engine is already running");
            return Ok(());
        }

        let python_path = python_engine::locate_python()?;
        python_engine::ensure_dependencies(&python_path).await?;

        tracing::info!("Starting AI engine with Python: {}", python_path.display());

        let child = python_engine::start_engine(&python_path)?;

        *self.ai_handle.lock().unwrap() = Some(child);
        self.ai_running.store(true, Ordering::Relaxed);

        let mut retries = 0u32;
        let max_retries = 30;
        while retries < max_retries {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            if python_engine::is_healthy().await {
                tracing::info!("AI engine is healthy after {} retries", retries);
                return Ok(());
            }
            retries += 1;
        }

        tracing::warn!("AI engine started but health check did not pass within timeout");
        Ok(())
    }

    fn stop_backend(&self) -> Result<()> {
        if let Ok(mut guard) = self.backend_handle.lock() {
            if let Some(ref mut child) = guard.take() {
                let _ = child.kill();
                let _ = child.wait();
                tracing::info!("Backend process terminated");
            }
        }
        self.backend_running.store(false, Ordering::Relaxed);
        Ok(())
    }

    fn stop_ai_engine(&self) -> Result<()> {
        if let Ok(mut guard) = self.ai_handle.lock() {
            if let Some(ref mut child) = guard.take() {
                let _ = child.kill();
                let _ = child.wait();
                tracing::info!("AI engine process terminated");
            }
        }
        self.ai_running.store(false, Ordering::Relaxed);
        Ok(())
    }
}

fn locate_backend_binary() -> Result<std::path::PathBuf> {
    if let Ok(exe_dir) = std::env::current_exe() {
        if let Some(parent) = exe_dir.parent() {
            let candidates = [
                parent.join("haqly-erp-server"),
                parent.join("haqly-erp-server.exe"),
                parent.join("sidecar").join("haqly-erp-server"),
                parent.join("sidecar").join("haqly-erp-server.exe"),
                parent.join("bin").join("haqly-erp-server"),
                parent.join("bin").join("haqly-erp-server.exe"),
            ];
            for candidate in &candidates {
                if candidate.exists() {
                    tracing::info!("Found backend binary: {}", candidate.display());
                    return Ok(candidate.clone());
                }
            }
        }
    }

    let path_check = std::process::Command::new("haqly-erp-server")
        .arg("--version")
        .output();

    if path_check.is_ok() {
        tracing::info!("Backend binary found in PATH");
        return Ok(std::path::PathBuf::from("haqly-erp-server"));
    }

    anyhow::bail!(
        "haqly-erp-server binary not found. Checked executable directory and PATH."
    )
}
