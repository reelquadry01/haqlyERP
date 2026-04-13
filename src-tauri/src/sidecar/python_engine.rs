// Author: Quadri Atharu
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

const AI_SERVICE_PORT: u16 = 8200;
const REQUIREMENTS_FILE: &str = "src-ai/requirements.txt";
const AI_MODULE: &str = "src_ai.main:app";

pub fn locate_python() -> Result<PathBuf> {
    let candidates = ["python3", "python", "py"];

    for candidate in &candidates {
        let result = std::process::Command::new(candidate)
            .arg("--version")
            .output();

        if let Ok(output) = result {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout);
                tracing::info!("Found Python: {candidate} => {version}");
                return Ok(PathBuf::from(candidate));
            }
        }
    }

    if cfg!(target_os = "windows") {
        let result = std::process::Command::new("py")
            .args(["--list"])
            .output();

        if let Ok(output) = result {
            if output.status.success() {
                let listing = String::from_utf8_lossy(&output.stdout);
                tracing::info!("py --list output:\n{listing}");
                for line in listing.lines() {
                    if line.contains("3.") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if !parts.is_empty() {
                            let exe_path = parts[0];
                            tracing::info!("Using Python from py launcher: {exe_path}");
                            return Ok(PathBuf::from(exe_path));
                        }
                    }
                }
            }
        }
    }

    let home = std::env::var(if cfg!(target_os = "windows") {
        "USERPROFILE"
    } else {
        "HOME"
    })
    .unwrap_or_default();

    let common_paths = if cfg!(target_os = "windows") {
        vec![
            PathBuf::from(r"C:\Python312\python.exe"),
            PathBuf::from(r"C:\Python311\python.exe"),
            PathBuf::from(r"C:\Python310\python.exe"),
            PathBuf::from(r"C:\Python39\python.exe"),
            PathBuf::from(format!(
                r"{home}\AppData\Local\Programs\Python\Python312\python.exe"
            )),
            PathBuf::from(format!(
                r"{home}\AppData\Local\Programs\Python\Python311\python.exe"
            )),
            PathBuf::from(format!(
                r"{home}\AppData\Local\Programs\Python\Python310\python.exe"
            )),
        ]
    } else {
        vec![
            PathBuf::from("/usr/bin/python3"),
            PathBuf::from("/usr/local/bin/python3"),
            PathBuf::from("/opt/homebrew/bin/python3"),
            PathBuf::from(format!("{home}/.local/bin/python3")),
        ]
    };

    for path in &common_paths {
        if path.exists() {
            tracing::info!("Found Python at: {}", path.display());
            return Ok(path.clone());
        }
    }

    anyhow::bail!(
        "Python not found. Install Python 3.9+ and ensure it is on PATH."
    )
}

pub async fn ensure_dependencies(python_path: &Path) -> Result<()> {
    let pip_module = if cfg!(target_os = "windows") && python_path.to_str() == Some("py") {
        "-m"
    } else {
        "-m"
    };

    let install_check = std::process::Command::new(python_path)
        .args([pip_module, "pip", "show", "fastapi"])
        .output()
        .context("Failed to check pip packages")?;

    if install_check.status.success() {
        tracing::info!("AI engine dependencies appear to be installed");
        return Ok(());
    }

    let requirements_path = find_requirements_file();
    let requirements_arg = match &requirements_path {
        Some(p) => p.to_string_lossy().to_string(),
        None => {
            tracing::warn!(
                "requirements.txt not found; installing core dependencies directly"
            );
            let core_deps = "fastapi uvicorn sqlalchemy asyncpg redis httpx pandas scikit-learn joblib";
            let output = std::process::Command::new(python_path)
                .args([pip_module, "pip", "install", "--quiet"])
                .args(core_deps.split_whitespace())
                .output()
                .context("Failed to install core AI dependencies")?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("pip install failed: {stderr}");
            }
            tracing::info!("Core AI dependencies installed successfully");
            return Ok(());
        }
    };

    tracing::info!("Installing AI dependencies from: {requirements_arg}");

    let output = std::process::Command::new(python_path)
        .args([pip_module, "pip", "install", "--quiet", "-r"])
        .arg(&requirements_arg)
        .output()
        .context("Failed to run pip install -r requirements.txt")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("pip install -r requirements.txt failed: {stderr}");
    }

    tracing::info!("AI dependencies installed successfully");
    Ok(())
}

pub fn start_engine(python_path: &Path) -> Result<std::process::Child> {
    let child = std::process::Command::new(python_path)
        .args(["-m", "uvicorn", AI_MODULE])
        .args(["--host", "0.0.0.0"])
        .args(["--port", &AI_SERVICE_PORT.to_string()])
        .args(["--workers", "1"])
        .env("PYTHONUNBUFFERED", "1")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .with_context(|| {
            format!(
                "Failed to start AI engine with Python: {}",
                python_path.display()
            )
        })?;

    tracing::info!("AI engine process spawned on port {AI_SERVICE_PORT}");
    Ok(child)
}

pub async fn is_healthy() -> bool {
    reqwest::Client::new()
        .get(format!("http://localhost:{AI_SERVICE_PORT}/health"))
        .timeout(std::time::Duration::from_secs(3))
        .send()
        .await
        .is_ok()
}

fn find_requirements_file() -> Option<PathBuf> {
    if let Ok(exe_dir) = std::env::current_exe() {
        if let Some(parent) = exe_dir.parent() {
            let candidates = [
                parent.join(REQUIREMENTS_FILE),
                parent.join("sidecar").join(REQUIREMENTS_FILE),
                parent.join("..").join(REQUIREMENTS_FILE),
                parent.join("..").join("..").join(REQUIREMENTS_FILE),
            ];
            for candidate in &candidates {
                if candidate.exists() {
                    return Some(candidate.clone());
                }
            }
        }
    }

    for ancestor in std::env::current_dir().ok()?.ancestors() {
        let path = ancestor.join(REQUIREMENTS_FILE);
        if path.exists() {
            return Some(path);
        }
    }

    None
}
