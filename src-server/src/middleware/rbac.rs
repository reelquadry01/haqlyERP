// Author: Quadri Atharu
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tower::Layer;
use tower::Service;

use crate::middleware::auth::Claims;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Role {
    SuperAdmin,
    Admin,
    Accountant,
    Auditor,
    HRManager,
    Sales,
    Purchaser,
    InventoryMgr,
    Treasury,
    Viewer,
}

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::SuperAdmin => "SuperAdmin",
            Role::Admin => "Admin",
            Role::Accountant => "Accountant",
            Role::Auditor => "Auditor",
            Role::HRManager => "HRManager",
            Role::Sales => "Sales",
            Role::Purchaser => "Purchaser",
            Role::InventoryMgr => "InventoryMgr",
            Role::Treasury => "Treasury",
            Role::Viewer => "Viewer",
        }
    }

    pub fn from_str_role(s: &str) -> Option<Self> {
        match s {
            "SuperAdmin" => Some(Role::SuperAdmin),
            "Admin" => Some(Role::Admin),
            "Accountant" => Some(Role::Accountant),
            "Auditor" => Some(Role::Auditor),
            "HRManager" => Some(Role::HRManager),
            "Sales" => Some(Role::Sales),
            "Purchaser" => Some(Role::Purchaser),
            "InventoryMgr" => Some(Role::InventoryMgr),
            "Treasury" => Some(Role::Treasury),
            "Viewer" => Some(Role::Viewer),
            _ => None,
        }
    }

    pub fn permissions(&self) -> Vec<&'static str> {
        match self {
            Role::SuperAdmin => vec![
                "users:view", "users:create", "users:update", "users:delete",
                "org:view", "org:create", "org:update",
                "accounting:coa", "accounting:journal", "accounting:voucher",
                "accounting:reports", "accounting:tax",
                "sales:view", "sales:create",
                "purchases:view", "purchases:create",
                "inventory:view", "inventory:create",
                "fixed_assets:view", "fixed_assets:create",
                "loans:view", "loans:create",
                "finance:view", "finance:create",
                "payroll:view", "payroll:create", "payroll:run",
                "employees:view", "employees:create", "employees:update",
                "admin:roles", "admin:license", "admin:security",
                "einvoicing:manage",
                "reports:view", "bi:view",
            ],
            Role::Admin => vec![
                "users:view", "users:create", "users:update",
                "org:view", "org:create",
                "accounting:coa", "accounting:journal", "accounting:voucher",
                "accounting:reports", "accounting:tax",
                "sales:view", "sales:create",
                "purchases:view", "purchases:create",
                "inventory:view", "inventory:create",
                "fixed_assets:view", "fixed_assets:create",
                "loans:view", "loans:create",
                "finance:view", "finance:create",
                "payroll:view", "payroll:create", "payroll:run",
                "employees:view", "employees:create", "employees:update",
                "admin:roles", "einvoicing:manage",
                "reports:view", "bi:view",
            ],
            Role::Accountant => vec![
                "accounting:coa", "accounting:journal", "accounting:voucher",
                "accounting:reports", "accounting:tax",
                "finance:view", "finance:create",
                "sales:view", "purchases:view",
                "reports:view",
            ],
            Role::Auditor => vec![
                "accounting:coa", "accounting:journal", "accounting:voucher",
                "accounting:reports", "accounting:tax",
                "finance:view",
                "sales:view", "purchases:view",
                "inventory:view", "fixed_assets:view", "loans:view",
                "payroll:view", "employees:view",
                "reports:view", "bi:view",
            ],
            Role::HRManager => vec![
                "payroll:view", "payroll:create", "payroll:run",
                "employees:view", "employees:create", "employees:update",
            ],
            Role::Sales => vec![
                "sales:view", "sales:create",
            ],
            Role::Purchaser => vec![
                "purchases:view", "purchases:create",
            ],
            Role::InventoryMgr => vec![
                "inventory:view", "inventory:create",
            ],
            Role::Treasury => vec![
                "loans:view", "loans:create",
                "finance:view",
            ],
            Role::Viewer => vec![
                "org:view", "accounting:coa", "accounting:journal",
                "sales:view", "purchases:view", "inventory:view",
                "fixed_assets:view", "loans:view", "finance:view",
                "reports:view",
            ],
        }
    }
}

pub struct AuthenticatedUser {
    pub user_id: uuid::Uuid,
    pub email: String,
    pub role: Role,
}

impl<S> axum::extract::FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = Response;

    fn from_request_parts<'life0, 'life1, 'async_trait>(
        parts: &'life0 mut axum::http::request::Parts,
        _state: &'life1 S,
    ) -> ::core::pin::Pin<Box<dyn ::core::future::Future<Output = Result<Self, Self::Rejection>> + ::core::marker::Send + 'async_trait>>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        let result = parts
            .extensions
            .get::<Claims>()
            .map(|claims| AuthenticatedUser {
                user_id: claims.sub,
                email: claims.email.clone(),
                role: Role::from_str_role(&claims.role).unwrap_or(Role::Viewer),
            });

        Box::pin(async move {
            match result {
                Some(user) => Ok(user),
                None => Err((
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({
                        "success": false,
                        "error": { "code": 401, "message": "Authentication required" }
                    })),
                )
                    .into_response()),
            }
        })
    }
}

fn build_route_permission_map() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("/api/v1/journals", "accounting:journal");
    m.insert("/api/v1/accounting", "accounting:coa");
    m.insert("/api/v1/payment-vouchers", "accounting:voucher");
    m.insert("/api/v1/tax", "accounting:tax");
    m.insert("/api/v1/reports", "reports:view");
    m.insert("/api/v1/sales", "sales:view");
    m.insert("/api/v1/purchases", "purchases:view");
    m.insert("/api/v1/inventory", "inventory:view");
    m.insert("/api/v1/fixed-assets", "fixed_assets:view");
    m.insert("/api/v1/depreciation", "fixed_assets:view");
    m.insert("/api/v1/loans", "loans:view");
    m.insert("/api/v1/payroll", "payroll:view");
    m.insert("/api/v1/employees", "employees:view");
    m.insert("/api/v1/bi", "bi:view");
    m.insert("/api/v1/admin", "admin:roles");
    m.insert("/api/v1/einvoicing", "einvoicing:manage");
    m.insert("/api/v1/imports", "accounting:journal");
    m.insert("/api/v1/crm", "sales:view");
    m.insert("/api/v1/notifications", "org:view");
    m.insert("/api/v1/users", "users:view");
    m.insert("/api/v1/org", "org:view");
    m
}

fn resolve_required_permission(path: &str, map: &HashMap<&str, &'static str>) -> Option<&'static str> {
    let stripped = path.split('?').next().unwrap_or(path);
    for (prefix, perm) in map.iter() {
        if stripped.starts_with(prefix) {
            return Some(perm);
        }
    }
    None
}

fn is_write_method(method: &str) -> bool {
    matches!(method, "POST" | "PUT" | "PATCH" | "DELETE")
}

fn write_permission(read_perm: &str) -> Option<&'static str> {
    match read_perm {
        "accounting:journal" => Some("accounting:journal"),
        "accounting:coa" => Some("accounting:coa"),
        "accounting:voucher" => Some("accounting:voucher"),
        "accounting:tax" => Some("accounting:tax"),
        "sales:view" => Some("sales:create"),
        "purchases:view" => Some("purchases:create"),
        "inventory:view" => Some("inventory:create"),
        "fixed_assets:view" => Some("fixed_assets:create"),
        "loans:view" => Some("loans:create"),
        "finance:view" => Some("finance:create"),
        "payroll:view" => Some("payroll:create"),
        "employees:view" => Some("employees:create"),
        "users:view" => Some("users:create"),
        "org:view" => Some("org:create"),
        "admin:roles" => Some("admin:roles"),
        "einvoicing:manage" => Some("einvoicing:manage"),
        _ => None,
    }
}

#[derive(Clone)]
pub struct RbacLayer {
    route_map: Arc<HashMap<&'static str, &'static str>>,
}

impl RbacLayer {
    pub fn new() -> Self {
        Self {
            route_map: Arc::new(build_route_permission_map()),
        }
    }
}

impl Default for RbacLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> Layer<S> for RbacLayer
where
    S: Service<Request<Body>, Response = Response, Error = Infallible> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Service = RbacService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RbacService {
            inner,
            route_map: self.route_map.clone(),
        }
    }
}

#[derive(Clone)]
pub struct RbacService<S> {
    inner: S,
    route_map: Arc<HashMap<&'static str, &'static str>>,
}

impl<S> Service<Request<Body>> for RbacService<S>
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
        let mut inner = self.inner.clone();
        let route_map = self.route_map.clone();
        let path = req.uri().path().to_string();
        let method = req.method().clone();

        let claims = req.extensions().get::<Claims>().cloned();

        Box::pin(async move {
            let Some(claims) = claims else {
                return inner.call(req).await;
            };

            let user_role = Role::from_str_role(&claims.role).unwrap_or(Role::Viewer);

            if user_role == Role::SuperAdmin {
                return inner.call(req).await;
            }

            let Some(base_perm) = resolve_required_permission(&path, &route_map) else {
                return inner.call(req).await;
            };

            let required_perm = if is_write_method(method.as_str()) {
                write_permission(base_perm).unwrap_or(base_perm)
            } else {
                base_perm
            };

            if user_role.permissions().contains(&required_perm) {
                inner.call(req).await
            } else {
                let body = serde_json::json!({
                    "success": false,
                    "error": {
                        "code": 403,
                        "message": format!("Permission '{}' required", required_perm)
                    }
                });
                Ok((
                    StatusCode::FORBIDDEN,
                    [(axum::http::header::CONTENT_TYPE, "application/json")],
                    serde_json::to_string(&body).unwrap_or_default(),
                )
                    .into_response())
            }
        })
    }
}

pub async fn require_role(
    required_roles: Vec<Role>,
    req: Request<Body>,
    next: axum::middleware::Next,
) -> Result<Response, Response> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned();

    match claims {
        Some(claims) => {
            let user_role = Role::from_str_role(&claims.role).unwrap_or(Role::Viewer);
            if required_roles.contains(&user_role) || user_role == Role::SuperAdmin {
                Ok(next.run(req).await)
            } else {
                Err((
                    StatusCode::FORBIDDEN,
                    Json(serde_json::json!({
                        "success": false,
                        "error": { "code": 403, "message": "Insufficient role permissions" }
                    })),
                )
                    .into_response())
            }
        }
        None => Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "success": false,
                "error": { "code": 401, "message": "Authentication required" }
            })),
        )
            .into_response()),
    }
}

pub fn require_permission(permission: &str) -> impl Fn(Request<Body>, axum::middleware::Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, Response>> + Send>> + Clone + 'static {
    let perm = permission.to_string();
    move |req: Request<Body>, next: axum::middleware::Next| {
        let perm = perm.clone();
        Box::pin(async move {
            let claims = req
                .extensions()
                .get::<Claims>()
                .cloned();

            match claims {
                Some(claims) => {
                    let user_role = Role::from_str_role(&claims.role).unwrap_or(Role::Viewer);
                    if user_role == Role::SuperAdmin
                        || user_role.permissions().contains(&perm.as_str())
                    {
                        Ok(next.run(req).await)
                    } else {
                        Err((
                            StatusCode::FORBIDDEN,
                            Json(serde_json::json!({
                                "success": false,
                                "error": { "code": 403, "message": format!("Permission '{}' required", perm) }
                            })),
                        )
                            .into_response())
                    }
                }
                None => Err((
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({
                        "success": false,
                        "error": { "code": 401, "message": "Authentication required" }
                    })),
                )
                    .into_response()),
            }
        })
    }
}
