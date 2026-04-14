// Author: Quadri Atharu
use axum::body::Body;
use axum::http::{HeaderName, HeaderValue, Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;

static XSS_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        Regex::new(r"(?i)<\s*script").unwrap(),
        Regex::new(r"(?i)javascript\s*:").unwrap(),
        Regex::new(r"(?i)on\w+\s*=").unwrap(),
        Regex::new(r"(?i)<\s*iframe").unwrap(),
        Regex::new(r"(?i)<\s*object").unwrap(),
        Regex::new(r"(?i)<\s*embed").unwrap(),
        Regex::new(r"(?i)eval\s*\(").unwrap(),
        Regex::new(r"(?i)expression\s*\(").unwrap(),
        Regex::new(r"(?i)vbscript\s*:").unwrap(),
        Regex::new(r"(?i)<\s*link[^>]+href").unwrap(),
    ]
});

static SQL_INJECTION_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        Regex::new(r"(?i)(\b(union)\b.+?\b(select)\b)").unwrap(),
        Regex::new(r"(?i)(\b(drop)\b\s+\b(table|database)\b)").unwrap(),
        Regex::new(r"(?i)(\b(insert|update|delete)\b\s+\b(into|from)\b)").unwrap(),
        Regex::new(r"(?i)(;\s*--\s*$)").unwrap(),
        Regex::new(r"(?i)(\b(exec|execute)\b\s*\()").unwrap(),
        Regex::new(r"(?i)('\s*(or|and)\s+.*=)").unwrap(),
        Regex::new(r"(?i)(\b(waitfor)\b\s+\b(delay)\b)").unwrap(),
        Regex::new(r"(?i)(\b(benchmark)\b\s*\()").unwrap(),
        Regex::new(r"(?i)(\b(sleep)\b\s*\()").unwrap(),
    ]
});

const MAX_REQUEST_SIZE: usize = 10 * 1024 * 1024;

pub fn sanitize_input(input: &str) -> String {
    let mut sanitized = input.to_string();
    sanitized = sanitized.replace('&', "&amp;");
    sanitized = sanitized.replace('<', "&lt;");
    sanitized = sanitized.replace('>', "&gt;");
    sanitized = sanitized.replace('"', "&quot;");
    sanitized = sanitized.replace('\'', "&#x27;");
    sanitized
}

pub fn detect_xss(input: &str) -> bool {
    XSS_PATTERNS.iter().any(|pattern| pattern.is_match(input))
}

pub fn detect_sql_injection(input: &str) -> bool {
    SQL_INJECTION_PATTERNS.iter().any(|pattern| pattern.is_match(input))
}

pub fn scan_request_body(body: &str) -> Vec<String> {
    let mut threats = Vec::new();
    if detect_xss(body) {
        threats.push("XSS pattern detected in request body".to_string());
    }
    if detect_sql_injection(body) {
        threats.push("SQL injection pattern detected in request body".to_string());
    }
    threats
}

pub async fn security_headers(req: Request<Body>, next: Next) -> Result<Response, Response> {
    let mut response = next.run(req).await;
    let headers = response.headers_mut();

    headers.insert(
        HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("deny"),
    );
    headers.insert(
        HeaderName::from_static("x-xss-protection"),
        HeaderValue::from_static("1; mode=block"),
    );
    headers.insert(
        HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    headers.insert(
        HeaderName::from_static("content-security-policy"),
        HeaderValue::from_static(
            "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; \
             img-src 'self' data:; font-src 'self'; connect-src 'self' https://einvoice.firs.gov.ng; \
             frame-ancestors 'none'; base-uri 'self'; form-action 'self'"
        ),
    );
    headers.insert(
        HeaderName::from_static("strict-transport-security"),
        HeaderValue::from_static("max-age=31536000; includeSubDomains; preload"),
    );
    headers.insert(
        HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
    );

    Ok(response)
}

pub async fn request_size_limit(req: Request<Body>, next: Next) -> Result<Response, Response> {
    let content_length = req
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<usize>().ok());

    if let Some(length) = content_length {
        if length > MAX_REQUEST_SIZE {
            return Err((
                StatusCode::PAYLOAD_TOO_LARGE,
                serde_json::json!({
                    "success": false,
                    "error": { "code": 413, "message": "Request body exceeds maximum allowed size" }
                }),
            )
                .into_response());
        }
    }

    Ok(next.run(req).await)
}

pub async fn suspicious_pattern_detector(req: Request<Body>, next: Next) -> Result<Response, Response> {
    let query = req.uri().query().unwrap_or("");
    if detect_sql_injection(query) || detect_xss(query) {
        tracing::warn!(
            "Suspicious pattern detected in query string: {}",
            query
        );
        return Err((
            StatusCode::BAD_REQUEST,
            serde_json::json!({
                "success": false,
                "error": { "code": 400, "message": "Request contains disallowed patterns" }
            }),
        )
            .into_response());
    }

    Ok(next.run(req).await)
}

pub fn strip_dangerous_input(input: &str) -> String {
    let mut result = input.to_string();
    for pattern in XSS_PATTERNS.iter() {
        result = pattern.replace_all(&result, "").to_string();
    }
    for pattern in SQL_INJECTION_PATTERNS.iter() {
        result = pattern.replace_all(&result, "").to_string();
    }
    result.trim().to_string()
}
