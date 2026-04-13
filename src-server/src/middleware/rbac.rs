// Author: Quadri Atharu
use axum::body::Body;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Role {
    SuperAdmin,
    Admin,
    Accountant,
    Sales,
    Purchaser,
    InventoryMgr,
    HR,
    Treasury,
    Viewer,
}

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::SuperAdmin => "SuperAdmin",
            Role::Admin => "Admin",
            Role::Accountant => "Accountant",
            Role::Sales => "Sales",
            Role::Purchaser => "Purchaser",
            Role::InventoryMgr => "InventoryMgr",
            Role::HR => "HR",
            Role::Treasury => "Treasury",
            Role::Viewer => "Viewer",
        }
    }

    pub fn from_str_role(s: &str) -> Option<Self> {
        match s {
            "SuperAdmin" => Some(Role::SuperAdmin),
            "Admin" => Some(Role::Admin),
            "Accountant" => Some(Role::Accountant),
            "Sales" => Some(Role::Sales),
            "Purchaser" => Some(Role::Purchaser),
            "InventoryMgr" => Some(Role::InventoryMgr),
            "HR" => Some(Role::HR),
            "Treasury" => Some(Role::Treasury),
            "Viewer" => Some(Role::Viewer),
            _ => None,
        }
    }

    pub fn permissions(&self) -> Vec<&'static str> {
        match self {
            Role::SuperAdmin => vec![
                "users:view", "users:create", "users:update",
                "org:view", "org:create",
                "accounting:coa", "accounting:journal", "accounting:voucher",
                "sales:view", "sales:create",
                "purchases:view", "purchases:create",
                "inventory:view", "inventory:create",
                "fixed_assets:view", "fixed_assets:create",
                "loans:view", "loans:create",
                "finance:view", "finance:create",
                "admin:roles", "einvoicing:manage",
            ],
            Role::Admin => vec![
                "users:view", "users:create", "users:update",
                "org:view", "org:create",
                "accounting:coa", "accounting:journal", "accounting:voucher",
                "sales:view", "sales:create",
                "purchases:view", "purchases:create",
                "inventory:view", "inventory:create",
                "fixed_assets:view", "fixed_assets:create",
                "loans:view", "loans:create",
                "finance:view", "finance:create",
                "admin:roles", "einvoicing:manage",
            ],
            Role::Accountant => vec![
                "accounting:coa", "accounting:journal", "accounting:voucher",
                "finance:view", "finance:create",
                "sales:view", "purchases:view",
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
            Role::HR => vec![
                "users:view", "users:create",
            ],
            Role::Treasury => vec![
                "loans:view", "loans:create",
                "finance:view",
            ],
            Role::Viewer => vec![
                "org:view", "accounting:coa", "accounting:journal",
                "sales:view", "purchases:view", "inventory:view",
                "fixed_assets:view", "loans:view", "finance:view",
            ],
        }
    }
}

pub struct AuthenticatedUser {
    pub user_id: uuid::Uuid,
    pub email: String,
    pub role: Role,
}

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = Response;

    fn from_request_parts(parts: &mut Parts, _state: &S) -> futures::future::BoxFuture<'_, Result<Self, Self::Rejection>> {
        let result = parts
            .extensions
            .get::<crate::middleware::auth::Claims>()
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
                    serde_json::json!({
                        "success": false,
                        "error": { "code": 401, "message": "Authentication required" }
                    }),
                )
                    .into_response()),
            }
        })
    }
}

pub async fn require_role(
    required_roles: Vec<Role>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, Response> {
    let claims = req
        .extensions()
        .get::<crate::middleware::auth::Claims>()
        .cloned();

    match claims {
        Some(claims) => {
            let user_role = Role::from_str_role(&claims.role).unwrap_or(Role::Viewer);
            if required_roles.contains(&user_role) || user_role == Role::SuperAdmin {
                Ok(next.run(req).await)
            } else {
                Err((
                    StatusCode::FORBIDDEN,
                    serde_json::json!({
                        "success": false,
                        "error": { "code": 403, "message": "Insufficient role permissions" }
                    }),
                )
                    .into_response())
            }
        }
        None => Err((
            StatusCode::UNAUTHORIZED,
            serde_json::json!({
                "success": false,
                "error": { "code": 401, "message": "Authentication required" }
            }),
        )
            .into_response()),
    }
}

pub fn require_permission(permission: &str) -> impl Fn(Request<Body>, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, Response>> + Send>> + Clone + 'static {
    let perm = permission.to_string();
    move |req: Request<Body>, next: Next| {
        let perm = perm.clone();
        Box::pin(async move {
            let claims = req
                .extensions()
                .get::<crate::middleware::auth::Claims>()
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
                            serde_json::json!({
                                "success": false,
                                "error": { "code": 403, "message": format!("Permission '{}' required", perm) }
                            }),
                        )
                            .into_response())
                    }
                }
                None => Err((
                    StatusCode::UNAUTHORIZED,
                    serde_json::json!({
                        "success": false,
                        "error": { "code": 401, "message": "Authentication required" }
                    }),
                )
                    .into_response()),
            }
        })
    }
}
