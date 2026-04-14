// Author: Quadri Atharu

use axum::body::Body;
use axum::http::{HeaderName, HeaderValue, Request, StatusCode};
use axum::response::{IntoResponse, Response};
use governor::clock::DefaultClock;
use governor::state::keyed::DefaultKeyedStateStore;
use governor::{Quota, RateLimiter};
use std::convert::Infallible;
use std::future::Future;
use std::net::IpAddr;
use std::num::NonZeroU32;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;
use tower::Layer;
use tower::Service;
use tower_governor::governor::GovernorConfigBuilder;

const AUTH_LIMIT: u32 = 5;
const API_LIMIT: u32 = 60;
const EINVOICING_LIMIT: u32 = 20;
const WINDOW_SECS: u64 = 60;

type SharedLimiter = Arc<RateLimiter<IpAddr, DefaultKeyedStateStore, DefaultClock>>;

#[derive(Clone, Copy, PartialEq, Eq)]
enum RateLimitTier {
    Auth,
    Api,
    EInvoicing,
}

impl RateLimitTier {
    fn limit(&self) -> u32 {
        match self {
            Self::Auth => AUTH_LIMIT,
            Self::Api => API_LIMIT,
            Self::EInvoicing => EINVOICING_LIMIT,
        }
    }

    fn quota(&self) -> Quota {
        match self {
            Self::Auth => Quota::with_period(
                Duration::from_secs(WINDOW_SECS) / AUTH_LIMIT,
                NonZeroU32::new(AUTH_LIMIT).unwrap(),
            ),
            Self::Api => Quota::with_period(
                Duration::from_secs(WINDOW_SECS) / API_LIMIT,
                NonZeroU32::new(API_LIMIT).unwrap(),
            ),
            Self::EInvoicing => Quota::with_period(
                Duration::from_secs(WINDOW_SECS) / EINVOICING_LIMIT,
                NonZeroU32::new(EINVOICING_LIMIT).unwrap(),
            ),
        }
    }
}

fn classify_path(path: &str) -> RateLimitTier {
    if path.contains("/auth/login")
        || path.contains("/auth/register")
        || path.contains("/auth/mfa")
        || path.contains("/auth/forgot-password")
        || path.contains("/auth/reset-password")
    {
        RateLimitTier::Auth
    } else if path.contains("/einvoicing/") {
        RateLimitTier::EInvoicing
    } else {
        RateLimitTier::Api
    }
}

fn extract_client_ip(req: &Request<Body>) -> IpAddr {
    req.headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .and_then(|v| v.trim().parse::<IpAddr>().ok())
        .or_else(|| {
            req.headers()
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<IpAddr>().ok())
        })
        .unwrap_or(IpAddr::from([127, 0, 0, 1]))
}

fn make_header_value(val: u32) -> HeaderValue {
    HeaderValue::from_str(&val.to_string()).unwrap_or_else(|_| HeaderValue::from_static("0"))
}

fn make_header_value_i64(val: i64) -> HeaderValue {
    HeaderValue::from_str(&val.to_string()).unwrap_or_else(|_| HeaderValue::from_static("0"))
}

struct RateLimiters {
    auth: SharedLimiter,
    api: SharedLimiter,
    einvoicing: SharedLimiter,
}

#[derive(Clone)]
pub struct RateLimitLayer {
    limiters: Arc<RateLimiters>,
}

impl RateLimitLayer {
    pub fn new() -> Self {
        let _auth_builder = GovernorConfigBuilder::default()
            .per_second(1)
            .burst_size(AUTH_LIMIT);

        let _einvoicing_builder = GovernorConfigBuilder::default()
            .per_second(1)
            .burst_size(EINVOICING_LIMIT);

        let _api_builder = GovernorConfigBuilder::default()
            .per_second(1)
            .burst_size(API_LIMIT);

        Self {
            limiters: Arc::new(RateLimiters {
                auth: Arc::new(RateLimiter::keyed(RateLimitTier::Auth.quota())),
                api: Arc::new(RateLimiter::keyed(RateLimitTier::Api.quota())),
                einvoicing: Arc::new(
                    RateLimiter::keyed(RateLimitTier::EInvoicing.quota()),
                ),
            }),
        }
    }
}

impl<S> Layer<S> for RateLimitLayer {
    type Service = RateLimitService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RateLimitService {
            inner,
            limiters: self.limiters.clone(),
        }
    }
}

#[derive(Clone)]
pub struct RateLimitService<S> {
    inner: S,
    limiters: Arc<RateLimiters>,
}

impl<S> Service<Request<Body>> for RateLimitService<S>
where
    S: Service<Request<Body>, Response = Response, Error = Infallible> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let inner = self.inner.clone();
        let limiters = self.limiters.clone();

        Box::pin(async move {
            let path = req.uri().path().to_string();
            let ip = extract_client_ip(&req);
            let tier = classify_path(&path);

            let limiter: &SharedLimiter = match tier {
                RateLimitTier::Auth => &limiters.auth,
                RateLimitTier::Api => &limiters.api,
                RateLimitTier::EInvoicing => &limiters.einvoicing,
            };

            let limit = tier.limit();
            let reset = chrono::Utc::now().timestamp() + WINDOW_SECS as i64;

            match limiter.check(&ip) {
                Ok(_) => {
                    let mut response = inner.call(req).await?;
                    let headers = response.headers_mut();

                    headers.insert(
                        HeaderName::from_static("x-ratelimit-limit"),
                        make_header_value(limit),
                    );
                    headers.insert(
                        HeaderName::from_static("x-ratelimit-remaining"),
                        make_header_value(limit.saturating_sub(1)),
                    );
                    headers.insert(
                        HeaderName::from_static("x-ratelimit-reset"),
                        make_header_value_i64(reset),
                    );

                    Ok(response)
                }
                Err(not_until) => {
                    let wait_time = not_until.wait_time_from(std::time::Instant::now());
                    let retry_after = wait_time.as_secs().max(1);

                    let body = serde_json::json!({
                        "success": false,
                        "error": {
                            "code": 429,
                            "message": "Too many requests, please try again later"
                        }
                    });

                    Ok((
                        StatusCode::TOO_MANY_REQUESTS,
                        [
                            (
                                HeaderName::from_static("retry-after"),
                                make_header_value(retry_after as u32),
                            ),
                            (
                                HeaderName::from_static("x-ratelimit-limit"),
                                make_header_value(limit),
                            ),
                            (
                                HeaderName::from_static("x-ratelimit-remaining"),
                                HeaderValue::from_static("0"),
                            ),
                            (
                                HeaderName::from_static("x-ratelimit-reset"),
                                make_header_value_i64(reset),
                            ),
                        ],
                        serde_json::to_string(&body).unwrap_or_default(),
                    )
                        .into_response())
                }
            }
        })
    }
}
