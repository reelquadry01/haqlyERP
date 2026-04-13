// Author: Quadri Atharu
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_updater::UpdaterExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpdateStatus {
    UpToDate {
        current_version: String,
    },
    UpdateAvailable {
        current_version: String,
        latest_version: String,
        release_notes: Option<String>,
    },
    UpdateInstalled {
        previous_version: String,
        new_version: String,
    },
    UpdateFailed {
        error: String,
    },
}

pub async fn check_and_update(app: AppHandle) -> Result<UpdateStatus, String> {
    let current_version = app
        .config()
        .version
        .clone()
        .unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string());

    let updater = app
        .updater()
        .map_err(|e| format!("Updater not available: {e}"))?;

    let update = match updater.check().await {
        Ok(Some(update)) => update,
        Ok(None) => {
            return Ok(UpdateStatus::UpToDate {
                current_version,
            });
        }
        Err(e) => {
            return Ok(UpdateStatus::UpdateFailed {
                error: format!("Update check failed: {e}"),
            });
        }
    };

    let latest_version = update.version.clone();
    let release_notes = update.body.clone();
    let previous_version = current_version.clone();

    tracing::info!(
        "Update available: {} -> {}",
        previous_version,
        latest_version
    );

    match update.download_and_install(
        |chunk_length, content_length| {
            tracing::debug!(
                "Downloaded {} bytes (total: {:?})",
                chunk_length,
                content_length
            );
        },
        || {
            tracing::info!("Download complete; preparing to install and restart");
        },
    ).await {
        Ok(()) => {
            tracing::info!("Update installed successfully");
            Ok(UpdateStatus::UpdateInstalled {
                previous_version,
                new_version: latest_version,
            })
        }
        Err(e) => {
            tracing::error!("Update installation failed: {e}");
            Ok(UpdateStatus::UpdateFailed {
                error: format!("Update installation failed: {e}"),
            })
        }
    }
}
