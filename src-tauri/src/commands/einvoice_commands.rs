// Author: Quadri Atharu
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub invoice_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningResult {
    pub irn: String,
    pub signed_hash: String,
    pub timestamp: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmationResult {
    pub irn: String,
    pub confirmed: bool,
    pub confirmation_date: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadResult {
    pub irn: String,
    pub content: String,
    pub content_type: String,
    pub downloaded_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EInvoiceStatus {
    pub company_id: String,
    pub active: bool,
    pub total_invoices: u64,
    pub signed_count: u64,
    pub confirmed_count: u64,
    pub last_activity: Option<String>,
}

const BACKEND_URL: &str = "http://localhost:8100/api/v1/einvoicing";
const TIMEOUT_SECS: u64 = 15;

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(TIMEOUT_SECS))
        .build()
        .expect("failed to build reqwest client")
}

#[tauri::command]
pub async fn validate_invoice_nrs(invoice_id: String) -> Result<ValidationResult, String> {
    let url = format!("{BACKEND_URL}/validate/{invoice_id}");
    let resp = client()
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Backend unreachable: {e}"))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Validation request failed ({status}): {body}"));
    }
    resp.json()
        .await
        .map_err(|e| format!("Failed to parse validation result: {e}"))
}

#[tauri::command]
pub async fn sign_invoice_nrs(invoice_id: String) -> Result<SigningResult, String> {
    let url = format!("{BACKEND_URL}/sign/{invoice_id}");
    let resp = client()
        .post(&url)
        .send()
        .await
        .map_err(|e| format!("Backend unreachable: {e}"))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Signing request failed ({status}): {body}"));
    }
    resp.json()
        .await
        .map_err(|e| format!("Failed to parse signing result: {e}"))
}

#[tauri::command]
pub async fn confirm_invoice_nrs(irn: String) -> Result<ConfirmationResult, String> {
    let url = format!("{BACKEND_URL}/confirm/{irn}");
    let resp = client()
        .post(&url)
        .send()
        .await
        .map_err(|e| format!("Backend unreachable: {e}"))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!(
            "Confirmation request failed ({status}): {body}"
        ));
    }
    resp.json()
        .await
        .map_err(|e| format!("Failed to parse confirmation result: {e}"))
}

#[tauri::command]
pub async fn download_invoice_nrs(irn: String) -> Result<DownloadResult, String> {
    let url = format!("{BACKEND_URL}/download/{irn}");
    let resp = client()
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Backend unreachable: {e}"))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!(
            "Download request failed ({status}): {body}"
        ));
    }
    resp.json()
        .await
        .map_err(|e| format!("Failed to parse download result: {e}"))
}

#[tauri::command]
pub async fn get_einvoice_status(company_id: String) -> Result<EInvoiceStatus, String> {
    let url = format!("{BACKEND_URL}/status/{company_id}");
    let resp = client()
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Backend unreachable: {e}"))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Status request failed ({status}): {body}"));
    }
    resp.json()
        .await
        .map_err(|e| format!("Failed to parse e-invoice status: {e}"))
}
