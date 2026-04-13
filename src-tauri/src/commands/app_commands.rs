use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tauri_plugin_updater::UpdaterExt;

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os: String,
    pub os_version: String,
    pub arch: String,
    pub app_version: String,
    pub cpu_count: usize,
    pub total_memory_gb: f64,
    pub hostname: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub available: bool,
    pub current_version: String,
    pub latest_version: Option<String>,
    pub release_notes: Option<String>,
    pub download_url: Option<String>,
    pub file_size: Option<u64>,
}

#[tauri::command]
pub async fn get_app_version(app: AppHandle) -> Result<String, String> {
    let version = app
        .config()
        .version
        .clone()
        .unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string());
    Ok(version)
}

#[tauri::command]
pub async fn open_external_link(url: String) -> Result<(), String> {
    let parsed = url::Url::parse(&url).map_err(|e| format!("Invalid URL: {e}"))?;
    match parsed.scheme() {
        "http" | "https" => {}
        _ => return Err("Only http and https URLs are allowed".to_string()),
    }
    open::that(url.as_str()).map_err(|e| format!("Failed to open URL: {e}"))
}

#[tauri::command]
pub async fn toggle_fullscreen(app: AppHandle) -> Result<bool, String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "Main window not found".to_string())?;
    let is_fullscreen = window
        .is_fullscreen()
        .map_err(|e| format!("Failed to check fullscreen state: {e}"))?;
    window
        .set_fullscreen(!is_fullscreen)
        .map_err(|e| format!("Failed to toggle fullscreen: {e}"))?;
    Ok(!is_fullscreen)
}

#[tauri::command]
pub async fn get_system_info(app: AppHandle) -> Result<SystemInfo, String> {
    let os = std::env::consts::OS.to_string();
    let os_version = sys_info::os_release().unwrap_or_else(|_| "unknown".to_string());
    let arch = std::env::consts::ARCH.to_string();
    let app_version = app
        .config()
        .version
        .clone()
        .unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string());
    let cpu_count = num_cpus::get();
    let total_memory_gb = sys_info::mem_info()
        .map(|m| (m.total as f64) / (1024.0 * 1024.0))
        .unwrap_or(0.0);
    let hostname = sys_info::hostname().unwrap_or_else(|_| "unknown".to_string());

    Ok(SystemInfo {
        os,
        os_version,
        arch,
        app_version,
        cpu_count,
        total_memory_gb,
        hostname,
    })
}

#[tauri::command]
pub async fn check_for_updates(app: AppHandle) -> Result<UpdateInfo, String> {
    let current_version = app
        .config()
        .version
        .clone()
        .unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string());

    match app.updater() {
        Ok(updater) => match updater.check().await {
            Ok(Some(update)) => Ok(UpdateInfo {
                available: true,
                current_version: current_version.clone(),
                latest_version: Some(update.version.clone()),
                release_notes: update.body.clone(),
                download_url: None,
                file_size: None,
            }),
            Ok(None) => Ok(UpdateInfo {
                available: false,
                current_version,
                latest_version: None,
                release_notes: None,
                download_url: None,
                file_size: None,
            }),
            Err(e) => Err(format!("Update check failed: {e}")),
        },
        Err(e) => Err(format!("Updater not available: {e}")),
    }
}
