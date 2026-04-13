// Author: Quadri Atharu
use std::time::Duration;

use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::ocr::classifier::DocumentType;

const OLLAMA_URL: &str = "http://localhost:11434/api/generate";
const OLLAMA_MODEL: &str = "llama3";
const OLLAMA_TIMEOUT_SECS: u64 = 120;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineItem {
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub quantity: f64,
    #[serde(default)]
    pub unit_price: f64,
    #[serde(default)]
    pub tax_rate: f64,
    #[serde(default)]
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionResult {
    #[serde(default)]
    pub vendor_name: String,
    #[serde(default)]
    pub vendor_tin: String,
    #[serde(default)]
    pub invoice_number: String,
    #[serde(default)]
    pub invoice_date: String,
    #[serde(default)]
    pub due_date: String,
    #[serde(default)]
    pub total_amount: f64,
    #[serde(default)]
    pub tax_amount: f64,
    #[serde(default)]
    pub currency: String,
    #[serde(default)]
    pub line_items: Vec<LineItem>,
    #[serde(default)]
    pub suggested_debit_account: String,
    #[serde(default)]
    pub suggested_credit_account: String,
    #[serde(default)]
    pub suggested_tax_account: String,
    #[serde(default)]
    pub confidence_score: f64,
}

impl Default for ExtractionResult {
    fn default() -> Self {
        Self {
            vendor_name: String::new(),
            vendor_tin: String::new(),
            invoice_number: String::new(),
            invoice_date: String::new(),
            due_date: String::new(),
            total_amount: 0.0,
            tax_amount: 0.0,
            currency: "NGN".to_string(),
            line_items: vec![],
            suggested_debit_account: String::new(),
            suggested_credit_account: String::new(),
            suggested_tax_account: String::new(),
            confidence_score: 0.0,
        }
    }
}

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    response: String,
}

pub async fn reason(text: &str, document_type: &DocumentType) -> Result<ExtractionResult> {
    let prompt = build_prompt(text, document_type);

    let body = serde_json::json!({
        "model": OLLAMA_MODEL,
        "prompt": prompt,
        "stream": false,
        "format": "json"
    });

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(OLLAMA_TIMEOUT_SECS))
        .build()
        .context("Failed to build HTTP client for Ollama")?;

    let resp = client
        .post(OLLAMA_URL)
        .json(&body)
        .send()
        .await
        .context("Ollama is unreachable; ensure it is running on localhost:11434")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body_text = resp.text().await.unwrap_or_default();
        anyhow::bail!("Ollama returned status {status}: {body_text}");
    }

    let ollama_resp: OllamaResponse = resp
        .json()
        .await
        .context("Failed to parse Ollama response envelope")?;

    let extraction = parse_extraction_from_llm_output(&ollama_resp.response)
        .context("Failed to parse LLM output as ExtractionResult")?;

    Ok(extraction)
}

pub fn fallback_extraction(text: &str, document_type: &DocumentType) -> ExtractionResult {
    tracing::warn!("Using regex fallback for extraction (Ollama unavailable)");

    let mut result = ExtractionResult::default();

    result.vendor_name = extract_vendor_name(text);
    result.vendor_tin = extract_tin(text);
    result.invoice_number = extract_invoice_number(text);
    result.invoice_date = extract_date(text);
    result.due_date = extract_due_date(text);
    result.total_amount = extract_total_amount(text);
    result.tax_amount = extract_tax_amount(text);
    result.currency = "NGN".to_string();
    result.line_items = vec![];
    result.suggested_debit_account = suggest_debit_account(document_type);
    result.suggested_credit_account = suggest_credit_account(document_type);
    result.suggested_tax_account = suggest_tax_account(document_type);
    result.confidence_score = 0.3;

    result
}

fn build_prompt(text: &str, document_type: &DocumentType) -> String {
    format!(
r#"You are an expert Nigerian accountant AI. Analyze the following document text and extract structured information.
Document type detected: {document_type}

Return ONLY a valid JSON object (no markdown, no explanation) with exactly these fields:
{{
  "vendor_name": "string - name of the vendor/supplier",
  "vendor_tin": "string - Nigerian Tax Identification Number (8+ digits)",
  "invoice_number": "string - invoice or document reference number",
  "invoice_date": "string - document date in YYYY-MM-DD format",
  "due_date": "string - payment due date in YYYY-MM-DD format, empty if not found",
  "total_amount": "number - total amount as a number without commas or currency symbols",
  "tax_amount": "number - VAT/tax amount as a number",
  "currency": "string - currency code, default NGN",
  "line_items": [{{"description": "string", "quantity": number, "unit_price": number, "tax_rate": number, "amount": number}}],
  "suggested_debit_account": "string - Nigerian COA debit account code and name",
  "suggested_credit_account": "string - Nigerian COA credit account code and name",
  "suggested_tax_account": "string - Nigerian COA tax account code and name",
  "confidence_score": "number from 0.0 to 1.0 - your confidence in the extraction"
}}

Nigerian Chart of Accounts conventions:
- Purchases of goods: Debit 3000 (Purchases)
- Services/Expenses: Debit 4000-4999 (various expense accounts)
- Accounts Payable: Credit 5000 (Trade Payables)
- Cash/Bank: Credit 1000-1999 (asset accounts)
- VAT Payable: Credit 5010
- WHT Payable: Credit 5020
- Use NGN as default currency for Nigerian documents
- For invoices: debit Purchases/Expense, credit Trade Payables
- For receipts: debit Cash/Bank, credit Revenue

Document text:
{text}"#
    )
}

fn parse_extraction_from_llm_output(raw: &str) -> Result<ExtractionResult> {
    let json_str = extract_json_from_text(raw);

    let extraction: ExtractionResult = serde_json::from_str(&json_str)
        .with_context(|| {
            format!(
                "LLM output is not valid ExtractionResult JSON. Raw (first 500 chars): {}",
                &json_str[..json_str.len().min(500)]
            )
        })?;

    Ok(extraction)
}

fn extract_json_from_text(text: &str) -> String {
    let trimmed = text.trim();

    if trimmed.starts_with('{') {
        if let Some(end) = find_matching_brace(trimmed) {
            return trimmed[..=end].to_string();
        }
    }

    if let Some(start) = trimmed.find('{') {
        let sub = &trimmed[start..];
        if let Some(end) = find_matching_brace(sub) {
            return sub[..=end].to_string();
        }
    }

    let re = Regex::new(r"```json\s*(.*?)\s*```").ok();
    if let Some(re) = re {
        if let Some(caps) = re.captures(trimmed) {
            if let Some(m) = caps.get(1) {
                return m.as_str().to_string();
            }
        }
    }

    let code_re = Regex::new(r"```\s*(.*?)\s*```").ok();
    if let Some(code_re) = code_re {
        if let Some(caps) = code_re.captures(trimmed) {
            if let Some(m) = caps.get(1) {
                let inner = m.as_str().trim();
                if inner.starts_with('{') {
                    return inner.to_string();
                }
            }
        }
    }

    trimmed.to_string()
}

fn find_matching_brace(s: &str) -> Option<usize> {
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escape_next = false;

    for (i, ch) in s.char_indices() {
        if escape_next {
            escape_next = false;
            continue;
        }
        if ch == '\\' && in_string {
            escape_next = true;
            continue;
        }
        if ch == '"' {
            in_string = !in_string;
            continue;
        }
        if !in_string {
            if ch == '{' {
                depth += 1;
            } else if ch == '}' {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
        }
    }
    None
}

fn extract_vendor_name(text: &str) -> String {
    let re = Regex::new(r"(?i)(?:from|vendor|supplier|seller)\s*[:\-]?\s*([^\n\r]{2,80})")
        .unwrap();
    if let Some(caps) = re.captures(text) {
        if let Some(m) = caps.get(1) {
            return m.as_str().trim().to_string();
        }
    }
    String::new()
}

fn extract_tin(text: &str) -> String {
    let re = Regex::new(r"(?i)(?:tin|tax\s*id(?:entification)?\s*(?:number)?)\s*[:\-]?\s*(\d[\d\-]{7,})")
        .unwrap();
    if let Some(caps) = re.captures(text) {
        if let Some(m) = caps.get(1) {
            return m.as_str().trim().to_string();
        }
    }
    String::new()
}

fn extract_invoice_number(text: &str) -> String {
    let re = Regex::new(
        r"(?i)(?:invoice|inv|receipt)\s*(?:no|number|#|ref)\s*[:\-]?\s*([A-Za-z0-9\-/]{2,30})",
    )
    .unwrap();
    if let Some(caps) = re.captures(text) {
        if let Some(m) = caps.get(1) {
            return m.as_str().trim().to_string();
        }
    }
    String::new()
}

fn extract_date(text: &str) -> String {
    let patterns = [
        r"(?i)(?:invoice|document|bill)\s*date\s*[:\-]?\s*(\d{1,2}[/\-]\d{1,2}[/\-]\d{2,4})",
        r"(?i)date\s*[:\-]?\s*(\d{1,2}[/\-]\d{1,2}[/\-]\d{2,4})",
        r"(\d{4}[/\-]\d{1,2}[/\-]\d{1,2})",
        r"(\d{1,2}[/\-]\d{1,2}[/\-]\d{4})",
    ];
    for pattern in &patterns {
        let re = Regex::new(pattern).unwrap();
        if let Some(caps) = re.captures(text) {
            if let Some(m) = caps.get(1) {
                return m.as_str().trim().to_string();
            }
        }
    }
    String::new()
}

fn extract_due_date(text: &str) -> String {
    let patterns = [
        r"(?i)(?:due|payment|pay)\s*date\s*[:\-]?\s*(\d{1,2}[/\-]\d{1,2}[/\-]\d{2,4})",
        r"(?i)(?:due|payment|pay)\s*[:\-]?\s*(\d{1,2}[/\-]\d{1,2}[/\-]\d{2,4})",
    ];
    for pattern in &patterns {
        let re = Regex::new(pattern).unwrap();
        if let Some(caps) = re.captures(text) {
            if let Some(m) = caps.get(1) {
                return m.as_str().trim().to_string();
            }
        }
    }
    String::new()
}

fn extract_total_amount(text: &str) -> f64 {
    let patterns = [
        r"(?i)total\s*(?:amount|due|payable)?\s*[:\-]?\s*[₦$₤]?\s*([\d,]+\.?\d*)",
        r"(?i)grand\s*total\s*[:\-]?\s*[₦$₤]?\s*([\d,]+\.?\d*)",
        r"[₦]\s*([\d,]+\.?\d*)",
    ];
    for pattern in &patterns {
        let re = Regex::new(pattern).unwrap();
        if let Some(caps) = re.captures(text) {
            if let Some(m) = caps.get(1) {
                let cleaned = m.as_str().replace(",", "").replace(" ", "");
                if let Ok(val) = cleaned.parse::<f64>() {
                    return val;
                }
            }
        }
    }
    0.0
}

fn extract_tax_amount(text: &str) -> f64 {
    let patterns = [
        r"(?i)(?:vat|value\s*added\s*tax)\s*(?:amount)?\s*[:\-]?\s*[₦$₤]?\s*([\d,]+\.?\d*)",
        r"(?i)tax\s*(?:amount)?\s*[:\-]?\s*[₦$₤]?\s*([\d,]+\.?\d*)",
        r"(?i)wht\s*(?:amount)?\s*[:\-]?\s*[₦$₤]?\s*([\d,]+\.?\d*)",
    ];
    for pattern in &patterns {
        let re = Regex::new(pattern).unwrap();
        if let Some(caps) = re.captures(text) {
            if let Some(m) = caps.get(1) {
                let cleaned = m.as_str().replace(",", "").replace(" ", "");
                if let Ok(val) = cleaned.parse::<f64>() {
                    return val;
                }
            }
        }
    }
    0.0
}

fn suggest_debit_account(document_type: &DocumentType) -> String {
    match document_type {
        DocumentType::Invoice | DocumentType::PurchaseOrder => "3000 - Purchases".to_string(),
        DocumentType::Receipt => "1000 - Cash/Bank".to_string(),
        DocumentType::BankStatement => "1000 - Cash/Bank".to_string(),
        DocumentType::CreditNote => "5000 - Trade Payables".to_string(),
        DocumentType::DebitNote => "3000 - Purchases".to_string(),
        DocumentType::DeliveryNote => "3000 - Purchases".to_string(),
        DocumentType::Unknown => "4000 - General Expenses".to_string(),
    }
}

fn suggest_credit_account(document_type: &DocumentType) -> String {
    match document_type {
        DocumentType::Invoice | DocumentType::PurchaseOrder => "5000 - Trade Payables".to_string(),
        DocumentType::Receipt => "4000 - Revenue".to_string(),
        DocumentType::BankStatement => String::new(),
        DocumentType::CreditNote => "3000 - Purchases".to_string(),
        DocumentType::DebitNote => "5000 - Trade Payables".to_string(),
        DocumentType::DeliveryNote => "5000 - Trade Payables".to_string(),
        DocumentType::Unknown => "5000 - Trade Payables".to_string(),
    }
}

fn suggest_tax_account(document_type: &DocumentType) -> String {
    match document_type {
        DocumentType::Invoice | DocumentType::PurchaseOrder | DocumentType::DebitNote => {
            "5010 - VAT Payable".to_string()
        }
        DocumentType::Receipt | DocumentType::CreditNote => "5010 - VAT Payable".to_string(),
        DocumentType::BankStatement | DocumentType::DeliveryNote | DocumentType::Unknown => {
            String::new()
        }
    }
}
