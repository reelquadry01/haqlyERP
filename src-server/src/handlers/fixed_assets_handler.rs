// Author: Quadri Atharu
// Crate: haqly-erp-server

use axum::{
    Json,
    routing::{get, post, patch},
    Router,
};
use axum::extract::{Path, Query, State};
use bigdecimal::BigDecimal;
use sqlx::PgPool;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::models::fixed_asset::DepreciationMethod;
use crate::services::fixed_assets_service::FixedAssetsService;

#[derive(Debug, Deserialize)]
pub struct ListCategoriesQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub company_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct ListAssetsQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub category_id: Option<Uuid>,
    pub status: Option<String>,
    pub company_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct DepreciationScheduleQuery {
    pub asset_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct AssetRegisterQuery {
    pub company_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct RunDepreciationRequest {
    pub company_id: Uuid,
    pub period_id: Uuid,
    pub fiscal_year_id: Uuid,
    pub run_by: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct DisposeAssetRequest {
    pub disposal_proceeds: BigDecimal,
    pub disposal_date: String,
    pub disposed_by: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct PostDepreciationRequest {
    pub schedule_ids: Vec<Uuid>,
    pub posted_by: Uuid,
}

pub fn router() -> Router<PgPool> {
    Router::new()
        .route("/categories", post(create_category).get(list_categories))
        .route("/assets", post(create_asset).get(list_assets))
        .route("/assets/:id", get(get_asset).patch(update_asset))
        .route("/assets/register", get(get_asset_register))
        .route("/assets/:id/depreciation-schedule", get(get_depreciation_schedule))
        .route("/assets/:id/dispose", post(dispose_asset))
        .route("/depreciation/run", post(run_depreciation))
        .route("/depreciation/post", post(post_depreciation))
}

async fn create_category(
    State(pool): State<PgPool>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let service = FixedAssetsService::new(pool);
    let company_id: Uuid = match body.get("company_id").and_then(|v| serde_json::from_value(v.clone()).ok()) {
        Some(id) => id,
        None => return Json(json!({"error": "company_id is required"})),
    };
    let name = body.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let depreciation_method: DepreciationMethod = match body.get("depreciation_method").and_then(|v| v.as_str()) {
        Some("declining_balance") => DepreciationMethod::DecliningBalance,
        Some("sum_of_years_digits") => DepreciationMethod::SumOfYearsDigits,
        _ => DepreciationMethod::StraightLine,
    };
    let useful_life_years: i32 = body.get("useful_life_years").and_then(|v| v.as_i64()).unwrap_or(5) as i32;
    let residual_value_percent: BigDecimal = body.get("residual_value_percent").and_then(|v| serde_json::from_value(v.clone()).ok()).unwrap_or(BigDecimal::from(0));
    let depreciation_rate: BigDecimal = body.get("depreciation_rate").and_then(|v| serde_json::from_value(v.clone()).ok()).unwrap_or(BigDecimal::from(20));
    let asset_account_id: Uuid = match body.get("asset_account_id").and_then(|v| serde_json::from_value(v.clone()).ok()) {
        Some(id) => id,
        None => return Json(json!({"error": "asset_account_id is required"})),
    };
    let accumulated_dep_account_id: Uuid = match body.get("accumulated_dep_account_id").and_then(|v| serde_json::from_value(v.clone()).ok()) {
        Some(id) => id,
        None => return Json(json!({"error": "accumulated_dep_account_id is required"})),
    };
    let depreciation_expense_account_id: Uuid = match body.get("depreciation_expense_account_id").and_then(|v| serde_json::from_value(v.clone()).ok()) {
        Some(id) => id,
        None => return Json(json!({"error": "depreciation_expense_account_id is required"})),
    };
    let disposal_account_id = body.get("disposal_account_id").and_then(|v| serde_json::from_value(v.clone()).ok());
    let capital_allowance_category = body.get("capital_allowance_category").and_then(|v| v.as_str()).map(|s| s.to_string());

    match service.create_category(
        company_id, name, depreciation_method, useful_life_years,
        residual_value_percent, depreciation_rate, asset_account_id,
        accumulated_dep_account_id, depreciation_expense_account_id,
        disposal_account_id, capital_allowance_category,
    ).await {
        Ok(category) => Json(json!(category)),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}

async fn list_categories(
    State(_pool): State<PgPool>,
    Query(_params): Query<ListCategoriesQuery>,
) -> Json<Value> {
    Json(json!({"message": "fixed-assets/categories GET - not implemented"}))
}

async fn create_asset(
    State(pool): State<PgPool>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let service = FixedAssetsService::new(pool);
    let company_id: Uuid = match body.get("company_id").and_then(|v| serde_json::from_value(v.clone()).ok()) {
        Some(id) => id,
        None => return Json(json!({"error": "company_id is required"})),
    };
    let branch_id = body.get("branch_id").and_then(|v| serde_json::from_value(v.clone()).ok());
    let category_id: Uuid = match body.get("category_id").and_then(|v| serde_json::from_value(v.clone()).ok()) {
        Some(id) => id,
        None => return Json(json!({"error": "category_id is required"})),
    };
    let asset_code = body.get("asset_code").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let name = body.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let description = body.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
    let acquisition_date_str = body.get("acquisition_date").and_then(|v| v.as_str()).unwrap_or("2024-01-01");
    let acquisition_date = match chrono::NaiveDate::parse_from_str(acquisition_date_str, "%Y-%m-%d") {
        Ok(d) => d,
        Err(e) => return Json(json!({"error": format!("Invalid acquisition_date: {}", e)})),
    };
    let acquisition_cost: BigDecimal = body.get("acquisition_cost").and_then(|v| serde_json::from_value(v.clone()).ok()).unwrap_or(BigDecimal::from(0));
    let residual_value: BigDecimal = body.get("residual_value").and_then(|v| serde_json::from_value(v.clone()).ok()).unwrap_or(BigDecimal::from(0));
    let useful_life_years: i32 = body.get("useful_life_years").and_then(|v| v.as_i64()).unwrap_or(5) as i32;
    let depreciation_method: DepreciationMethod = match body.get("depreciation_method").and_then(|v| v.as_str()) {
        Some("declining_balance") => DepreciationMethod::DecliningBalance,
        Some("sum_of_years_digits") => DepreciationMethod::SumOfYearsDigits,
        _ => DepreciationMethod::StraightLine,
    };
    let depreciation_rate: BigDecimal = body.get("depreciation_rate").and_then(|v| serde_json::from_value(v.clone()).ok()).unwrap_or(BigDecimal::from(20));
    let asset_account_id: Uuid = match body.get("asset_account_id").and_then(|v| serde_json::from_value(v.clone()).ok()) {
        Some(id) => id,
        None => return Json(json!({"error": "asset_account_id is required"})),
    };
    let accumulated_dep_account_id: Uuid = match body.get("accumulated_dep_account_id").and_then(|v| serde_json::from_value(v.clone()).ok()) {
        Some(id) => id,
        None => return Json(json!({"error": "accumulated_dep_account_id is required"})),
    };
    let depreciation_expense_account_id: Uuid = match body.get("depreciation_expense_account_id").and_then(|v| serde_json::from_value(v.clone()).ok()) {
        Some(id) => id,
        None => return Json(json!({"error": "depreciation_expense_account_id is required"})),
    };
    let disposal_account_id = body.get("disposal_account_id").and_then(|v| serde_json::from_value(v.clone()).ok());
    let location = body.get("location").and_then(|v| v.as_str()).map(|s| s.to_string());
    let custodian = body.get("custodian").and_then(|v| v.as_str()).map(|s| s.to_string());
    let serial_number = body.get("serial_number").and_then(|v| v.as_str()).map(|s| s.to_string());
    let created_by: Uuid = body.get("created_by").and_then(|v| serde_json::from_value(v.clone()).ok()).unwrap_or(Uuid::now_v7());

    match service.register_asset(
        company_id, branch_id, category_id, asset_code, name, description,
        acquisition_date, acquisition_cost, residual_value, useful_life_years,
        depreciation_method, depreciation_rate, asset_account_id,
        accumulated_dep_account_id, depreciation_expense_account_id,
        disposal_account_id, location, custodian, serial_number, created_by,
    ).await {
        Ok(asset) => Json(json!(asset)),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}

async fn list_assets(
    State(pool): State<PgPool>,
    Query(params): Query<ListAssetsQuery>,
) -> Json<Value> {
    let service = FixedAssetsService::new(pool);
    match service.list_assets(params.company_id).await {
        Ok(assets) => Json(json!(assets)),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}

async fn get_asset(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
) -> Json<Value> {
    Json(json!({"message": "fixed-assets/assets/:id GET - not implemented"}))
}

async fn update_asset(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "fixed-assets/assets/:id PATCH - not implemented"}))
}

async fn get_asset_register(
    State(pool): State<PgPool>,
    Query(params): Query<AssetRegisterQuery>,
) -> Json<Value> {
    let service = FixedAssetsService::new(pool);
    match service.get_asset_register(params.company_id).await {
        Ok(register) => Json(json!(register)),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}

async fn get_depreciation_schedule(
    State(pool): State<PgPool>,
    Path(asset_id): Path<Uuid>,
) -> Json<Value> {
    let service = FixedAssetsService::new(pool);
    match service.get_depreciation_schedule(asset_id).await {
        Ok(schedule) => Json(json!(schedule)),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}

async fn dispose_asset(
    State(pool): State<PgPool>,
    Path(asset_id): Path<Uuid>,
    Json(body): Json<DisposeAssetRequest>,
) -> Json<Value> {
    let service = FixedAssetsService::new(pool);
    let disposal_date = match chrono::NaiveDate::parse_from_str(&body.disposal_date, "%Y-%m-%d") {
        Ok(d) => d,
        Err(e) => return Json(json!({"error": format!("Invalid disposal_date: {}", e)})),
    };
    match service.dispose_asset(asset_id, body.disposal_proceeds, disposal_date, body.disposed_by).await {
        Ok(result) => Json(json!(result)),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}

async fn run_depreciation(
    State(pool): State<PgPool>,
    Json(body): Json<RunDepreciationRequest>,
) -> Json<Value> {
    let service = FixedAssetsService::new(pool);
    match service.run_depreciation(body.company_id, body.period_id, body.fiscal_year_id, body.run_by).await {
        Ok(schedules) => Json(json!(schedules)),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}

async fn post_depreciation(
    State(pool): State<PgPool>,
    Json(body): Json<PostDepreciationRequest>,
) -> Json<Value> {
    let service = FixedAssetsService::new(pool);
    match service.post_depreciation_to_gl(body.schedule_ids, body.posted_by).await {
        Ok(_) => Json(json!({"message": "Depreciation schedules posted to GL"})),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}
