// Author: Quadri Atharu
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Insight {
    pub category: String,
    pub description: String,
    pub severity: String,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub company_id: String,
    pub analysis_type: String,
    pub insights: Vec<Insight>,
    pub recommendations: Vec<String>,
    pub generated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxBreakdownItem {
    pub name: String,
    pub rate: f64,
    pub taxable_amount: f64,
    pub tax_amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxResult {
    pub company_id: String,
    pub tax_type: String,
    pub total_tax: f64,
    pub breakdown: Vec<TaxBreakdownItem>,
    pub currency: String,
    pub period: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportResult {
    pub company_id: String,
    pub report_type: String,
    pub title: String,
    pub content: serde_json::Value,
    pub generated_at: String,
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiEngineStatus {
    pub available: bool,
    pub model: String,
    pub version: String,
    pub uptime_seconds: u64,
}

const AI_URL: &str = "http://localhost:8200";
const TIMEOUT_SECS: u64 = 30;

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(TIMEOUT_SECS))
        .build()
        .expect("failed to build reqwest client")
}

#[tauri::command]
pub async fn analyze_financials(
    company_id: String,
    analysis_type: String,
) -> Result<AnalysisResult, String> {
    let url = format!("{AI_URL}/analyze");
    let body = serde_json::json!({
        "company_id": company_id,
        "analysis_type": analysis_type,
    });
    let resp = client()
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("AI engine unreachable: {e}"))?;
    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Analysis request failed ({}): {body}", resp.status()));
    }
    resp.json()
        .await
        .map_err(|e| format!("Failed to parse analysis result: {e}"))
}

#[tauri::command]
pub async fn compute_tax(company_id: String, tax_type: String) -> Result<TaxResult, String> {
    let url = format!("{AI_URL}/tax/compute");
    let body = serde_json::json!({
        "company_id": company_id,
        "tax_type": tax_type,
    });
    let resp = client()
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("AI engine unreachable: {e}"))?;
    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!(
            "Tax computation request failed ({}): {body}",
            resp.status()
        ));
    }
    resp.json()
        .await
        .map_err(|e| format!("Failed to parse tax result: {e}"))
}

#[tauri::command]
pub async fn generate_report(
    company_id: String,
    report_type: String,
) -> Result<ReportResult, String> {
    let url = format!("{AI_URL}/reports/generate");
    let body = serde_json::json!({
        "company_id": company_id,
        "report_type": report_type,
    });
    let resp = client()
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("AI engine unreachable: {e}"))?;
    if !resp.status().is_success() {
        let b = resp.text().await.unwrap_or_default();
        return Err(format!("Report generation failed ({}): {b}", resp.status()));
    }
    resp.json()
        .await
        .map_err(|e| format!("Failed to parse report result: {e}"))
}

#[tauri::command]
pub async fn get_ai_status() -> Result<AiEngineStatus, String> {
    let url = format!("{AI_URL}/health");
    let resp = client()
        .get(&url)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await;
    match resp {
        Ok(response) => {
            if response.status().is_success() {
                response
                    .json()
                    .await
                    .map_err(|e| format!("Failed to parse AI status: {e}"))
            } else {
                Ok(AiEngineStatus {
                    available: false,
                    model: String::new(),
                    version: String::new(),
                    uptime_seconds: 0,
                })
            }
        }
        Err(_) => Ok(AiEngineStatus {
            available: false,
            model: String::new(),
            version: String::new(),
            uptime_seconds: 0,
        }),
    }
}
