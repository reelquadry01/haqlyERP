// Author: Quadri Atharu
// Crate: haqly-erp-server

use axum::{
    Json,
    routing::get,
    Router,
};
use axum::extract::{FromRef, State};
use sqlx::PgPool;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::config::settings::Settings;
use crate::config::rsa_keys::RsaKeypair;
use crate::handlers::{
    auth_routes, users_routes, org_routes, accounting_routes,
    journals_routes, payment_vouchers_routes, sales_routes, purchases_routes,
    inventory_routes, tax_routes, fixed_assets_routes, depreciation_routes,
    loans_routes, reports_routes, imports_routes, admin_routes,
    einvoicing_routes, ocr_routes, ai_routes, payroll_routes,
    crm_routes, bi_routes, notification_routes, document_routes,
};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub rsa_keypair: Arc<RsaKeypair>,
    pub jwt_expiration: u64,
}

impl FromRef<AppState> for PgPool {
    fn from_ref(state: &AppState) -> Self {
        state.pool.clone()
    }
}

pub fn app_routes(pool: PgPool, settings: Settings) -> Router {
    let state = AppState {
        pool: pool.clone(),
        rsa_keypair: settings.rsa_keypair.clone(),
        jwt_expiration: settings.jwt_expiration,
    };

    let api_v1 = Router::new()
        .route("/health", get(health_check))
        .nest("/auth", auth_routes())
        .nest("/users", users_routes())
        .nest("/org", org_routes())
        .nest("/accounting", accounting_routes())
        .nest("/journals", journals_routes())
        .nest("/payment-vouchers", payment_vouchers_routes())
        .nest("/sales", sales_routes())
        .nest("/purchases", purchases_routes())
        .nest("/inventory", inventory_routes())
        .nest("/tax", tax_routes())
        .nest("/fixed-assets", fixed_assets_routes())
        .nest("/depreciation", depreciation_routes())
        .nest("/loans", loans_routes())
        .nest("/reports", reports_routes())
        .nest("/imports", imports_routes())
        .nest("/admin", admin_routes())
        .nest("/einvoicing", einvoicing_routes())
        .nest("/documents", ocr_routes())
        .nest("/ai", ai_routes())
        .nest("/payroll", payroll_routes())
        .nest("/crm", crm_routes())
        .nest("/bi", bi_routes())
        .nest("/notifications", notification_routes())
        .nest("/file-storage", document_routes())
        .with_state(state);

    Router::new().nest_service("/api/v1", api_v1)
}

async fn health_check(State(_pool): State<PgPool>) -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": "haqly-erp-server",
        "version": env!("CARGO_PKG_VERSION"),
        "author": "Quadri Atharu"
    }))
}
