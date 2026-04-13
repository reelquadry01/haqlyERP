// Author: Quadri Atharu
use serde_json::{json, Value};
use thiserror::Error;
use tracing::{debug, error, warn};

#[derive(Debug, Error)]
pub enum FirsClientError {
    #[error("FIRS client not configured: {details}")]
    NotConfigured { details: String },

    #[error("FIRS request failed: {endpoint} returned {status}")]
    RequestFailed {
        endpoint: String,
        status: u16,
        response: Value,
    },

    #[error("FIRS authentication failed: {message}")]
    AuthenticationFailed { message: String },

    #[error("FIRS network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("FIRS response parse error: {0}")]
    ParseError(String),
}

pub struct FirsClient {
    base_url: String,
    api_key: String,
    api_secret: String,
    http: reqwest::Client,
}

impl FirsClient {
    pub fn new(base_url: &str, api_key: &str, api_secret: &str) -> Self {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("failed to build reqwest client");
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
            api_secret: api_secret.to_string(),
            http,
        }
    }

    pub fn from_credentials(cred: &super::EInvoiceCredentialRow) -> Self {
        Self::new(&cred.base_url, &cred.api_key, &cred.api_secret)
    }

    pub fn get_configuration_status(&self) -> ConfigurationStatus {
        ConfigurationStatus {
            configured: !self.base_url.is_empty() && !self.api_key.is_empty() && !self.api_secret.is_empty(),
            base_url_configured: !self.base_url.is_empty(),
            api_key_configured: !self.api_key.is_empty(),
            api_secret_configured: !self.api_secret.is_empty(),
            auth_headers: vec!["x-api-key".to_string(), "x-api-secret".to_string()],
            base_url: if self.base_url.is_empty() {
                None
            } else {
                Some(self.base_url.clone())
            },
        }
    }

    fn assert_configured(&self) -> Result<(), FirsClientError> {
        let status = self.get_configuration_status();
        if !status.configured {
            return Err(FirsClientError::NotConfigured {
                details: format!(
                    "baseUrl={}, apiKey={}, apiSecret={}",
                    status.base_url_configured,
                    status.api_key_configured,
                    status.api_secret_configured
                ),
            });
        }
        Ok(())
    }

    fn build_headers(&self, has_body: bool) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        if let Ok(val) = reqwest::header::HeaderValue::from_str(&self.api_key) {
            headers.insert("x-api-key", val);
        }
        if let Ok(val) = reqwest::header::HeaderValue::from_str(&self.api_secret) {
            headers.insert("x-api-secret", val);
        }
        if has_body {
            headers.insert(
                reqwest::header::CONTENT_TYPE,
                reqwest::header::HeaderValue::from_static("application/json"),
            );
        }
        headers
    }

    fn parse_response_body(raw: &str) -> Result<Value, FirsClientError> {
        if raw.is_empty() {
            return Ok(Value::Null);
        }
        serde_json::from_str(raw).map_err(|e| FirsClientError::ParseError(e.to_string()))
    }

    async fn request(
        &self,
        method: reqwest::Method,
        path: &str,
        payload: Option<&Value>,
    ) -> Result<Value, FirsClientError> {
        self.assert_configured()?;

        let url = format!("{}{}", self.base_url, path);
        debug!(method = %method, url = %url, "FIRS request");

        let has_body = payload.is_some();
        let headers = self.build_headers(has_body);

        let mut builder = self.http.request(method, &url).headers(headers);

        if let Some(body) = payload {
            builder = builder.json(body);
        }

        let response = builder.send().await?;

        let status = response.status().as_u16();
        let raw = response.text().await.unwrap_or_default();
        let parsed = Self::parse_response_body(&raw)?;

        if status >= 400 {
            error!(status, endpoint = %path, "FIRS request failed");
            return Err(FirsClientError::RequestFailed {
                endpoint: path.to_string(),
                status,
                response: parsed,
            });
        }

        debug!(status, "FIRS request succeeded");
        Ok(parsed)
    }

    pub async fn taxpayer_authenticate(&self, payload: &Value) -> Result<Value, FirsClientError> {
        self.request(reqwest::Method::POST, "/api/v1/utilities/authenticate", Some(payload))
            .await
    }

    pub async fn validate_irn(&self, payload: &Value) -> Result<Value, FirsClientError> {
        self.request(reqwest::Method::POST, "/api/v1/invoice/irn/validate", Some(payload))
            .await
    }

    pub async fn validate_invoice(&self, payload: &Value) -> Result<Value, FirsClientError> {
        self.request(reqwest::Method::POST, "/api/v1/invoice/validate", Some(payload))
            .await
    }

    pub async fn sign_invoice(&self, payload: &Value) -> Result<Value, FirsClientError> {
        self.request(reqwest::Method::POST, "/api/v1/invoice/sign", Some(payload))
            .await
    }

    pub async fn confirm_invoice(&self, irn: &str) -> Result<Value, FirsClientError> {
        let path = format!("/api/v1/invoice/confirm/{}", irn);
        self.request(reqwest::Method::GET, &path, None).await
    }

    pub async fn download_invoice(&self, irn: &str) -> Result<Value, FirsClientError> {
        let path = format!("/api/v1/invoice/download/{}", irn);
        self.request(reqwest::Method::GET, &path, None).await
    }

    pub async fn update_invoice(&self, irn: &str, payload: &Value) -> Result<Value, FirsClientError> {
        let path = format!("/api/v1/invoice/update/{}", irn);
        self.request(reqwest::Method::PATCH, &path, Some(payload))
            .await
    }

    pub async fn report_post_payment(&self, payload: &Value) -> Result<Value, FirsClientError> {
        self.request(reqwest::Method::POST, "/api/v1/vat/postpayment", Some(payload))
            .await
    }

    pub async fn exchange_self_health_check(&self) -> Result<Value, FirsClientError> {
        self.request(reqwest::Method::GET, "/api/v1/invoice/transmit/self-health-check", None)
            .await
    }

    pub async fn exchange_lookup_irn(&self, irn: &str) -> Result<Value, FirsClientError> {
        let path = format!("/api/v1/invoice/transmit/lookup/{}", irn);
        self.request(reqwest::Method::GET, &path, None).await
    }

    pub async fn exchange_lookup_tin(&self, tin: &str) -> Result<Value, FirsClientError> {
        let path = format!("/api/v1/invoice/transmit/lookup/tin/{}", tin);
        self.request(reqwest::Method::GET, &path, None).await
    }

    pub async fn exchange_transmit(&self, irn: &str, payload: &Value) -> Result<Value, FirsClientError> {
        let path = format!("/api/v1/invoice/transmit/{}", irn);
        self.request(reqwest::Method::POST, &path, Some(payload))
            .await
    }

    pub async fn exchange_confirm_receipt(&self, irn: &str, payload: &Value) -> Result<Value, FirsClientError> {
        let path = format!("/api/v1/invoice/transmit/{}", irn);
        self.request(reqwest::Method::PATCH, &path, Some(payload))
            .await
    }

    pub async fn exchange_pull_invoice(&self) -> Result<Value, FirsClientError> {
        self.request(reqwest::Method::GET, "/api/v1/invoice/transmit/pull", None)
            .await
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "serde")]
pub struct ConfigurationStatus {
    pub configured: bool,
    pub base_url_configured: bool,
    pub api_key_configured: bool,
    pub api_secret_configured: bool,
    pub auth_headers: Vec<String>,
    pub base_url: Option<String>,
}
