// Author: Quadri Atharu
use anyhow::{anyhow, Result};
use bigdecimal::BigDecimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::dtos::tax_dto::{TaxComputationRequest, TaxComputationResult, TaxDashboard};
use crate::models::tax::{TaxConfig, TaxTransaction, TaxType, WhtRateCategory};

#[derive(Clone)]
pub struct TaxService {
    pub pool: PgPool,
}

impl TaxService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_tax_config(
        &self,
        company_id: Uuid,
        tax_type: TaxType,
        name: String,
        rate: BigDecimal,
        effective_from: chrono::NaiveDate,
        effective_to: Option<chrono::NaiveDate>,
        account_id: Option<Uuid>,
        wht_category: Option<WhtRateCategory>,
        description: Option<String>,
    ) -> Result<TaxConfig> {
        let id = Uuid::now_v7();
        sqlx::query(
            r#"INSERT INTO tax_configs (id, company_id, tax_type, name, rate, effective_from, effective_to, is_active, account_id, wht_category, description, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, true, $8, $9, $10, NOW(), NOW())"#,
        )
        .bind(id)
        .bind(company_id)
        .bind(&tax_type)
        .bind(&name)
        .bind(&rate)
        .bind(effective_from)
        .bind(effective_to)
        .bind(account_id)
        .bind(&wht_category)
        .bind(&description)
        .execute(&self.pool)
        .await?;

        sqlx::query_as::<_, TaxConfig>("SELECT * FROM tax_configs WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch tax config: {}", e))
    }

    pub async fn list_configs(&self, company_id: Uuid) -> Result<Vec<TaxConfig>> {
        let configs = sqlx::query_as::<_, TaxConfig>(
            "SELECT * FROM tax_configs WHERE company_id = $1 AND is_active = true ORDER BY tax_type, name",
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(configs)
    }

    pub fn compute_vat(&self, base_amount: &BigDecimal) -> TaxComputationResult {
        let rate = BigDecimal::from(7) + BigDecimal::from(5) / BigDecimal::from(10);
        let tax_amount = base_amount * &rate / BigDecimal::from(100);
        TaxComputationResult {
            tax_type: "VAT".to_string(),
            base_amount: base_amount.clone(),
            rate: rate,
            tax_amount,
            currency: "NGN".to_string(),
            notes: Some("Nigerian VAT at 7.5%".to_string()),
        }
    }

    pub fn compute_wht(&self, base_amount: &BigDecimal, category: &WhtRateCategory) -> TaxComputationResult {
        let rate = category.rate();
        let tax_amount = base_amount * &rate / BigDecimal::from(100);
        TaxComputationResult {
            tax_type: "WHT".to_string(),
            base_amount: base_amount.clone(),
            rate: rate.clone(),
            tax_amount,
            currency: "NGN".to_string(),
            notes: Some(format!("WHT for {} category at {}%", category, rate)),
        }
    }

    pub fn compute_cit(&self, taxable_profit: &BigDecimal, annual_revenue: &BigDecimal) -> TaxComputationResult {
        let (rate, note) = if *annual_revenue <= BigDecimal::from(50_000_000) {
            (BigDecimal::from(0), "Exempt: annual revenue <= NGN 50M (Small company - Tax Reform 2025)".to_string())
        } else if *annual_revenue <= BigDecimal::from(250_000_000) {
            (BigDecimal::from(15), "15% for medium companies (revenue NGN 50M-250M - Tax Reform 2025)".to_string())
        } else {
            (BigDecimal::from(25), "25% for large companies (revenue > NGN 250M - Tax Reform 2025)".to_string())
        };

        let tax_amount = taxable_profit * &rate / BigDecimal::from(100);
        TaxComputationResult {
            tax_type: "CIT".to_string(),
            base_amount: taxable_profit.clone(),
            rate: rate.clone(),
            tax_amount,
            currency: "NGN".to_string(),
            notes: Some(note),
        }
    }

    pub fn compute_education_tax(&self, assessable_profit: &BigDecimal) -> TaxComputationResult {
        let rate = BigDecimal::from(1);
        let tax_amount = assessable_profit * &rate / BigDecimal::from(100);
        TaxComputationResult {
            tax_type: "Education Tax".to_string(),
            base_amount: assessable_profit.clone(),
            rate,
            tax_amount,
            currency: "NGN".to_string(),
            notes: Some("1% Education Tax on assessable profit (Tax Reform 2025, NDDC merged)".to_string()),
        }
    }

    pub fn compute_cgt(&self, capital_gain: &BigDecimal) -> TaxComputationResult {
        let (rate, note) = if *capital_gain <= BigDecimal::from(50_000_000) {
            (BigDecimal::from(10), "10% CGT on gains up to NGN 50M (Tax Reform 2025)".to_string())
        } else if *capital_gain <= BigDecimal::from(250_000_000) {
            (BigDecimal::from(15), "15% CGT on gains NGN 50M-250M (Tax Reform 2025)".to_string())
        } else {
            (BigDecimal::from(20), "20% CGT on gains above NGN 250M (Tax Reform 2025)".to_string())
        };

        let tax_amount = capital_gain * &rate / BigDecimal::from(100);
        TaxComputationResult {
            tax_type: "CGT".to_string(),
            base_amount: capital_gain.clone(),
            rate,
            tax_amount,
            currency: "NGN".to_string(),
            notes: Some(note),
        }
    }

    pub async fn compute(&self, req: TaxComputationRequest) -> Result<TaxComputationResult> {
        match req.tax_type.to_lowercase().as_str() {
            "vat" => Ok(self.compute_vat(&req.base_amount)),
            "wht" => {
                let category = req
                    .category
                    .as_deref()
                    .and_then(|c| match c {
                        "contract_general" => Some(WhtRateCategory::ContractGeneral),
                        "contract_construction" => Some(WhtRateCategory::ContractConstruction),
                        "consultancy" => Some(WhtRateCategory::Consultancy),
                        "management" => Some(WhtRateCategory::Management),
                        "dividend" => Some(WhtRateCategory::Dividend),
                        "interest" => Some(WhtRateCategory::Interest),
                        "royalty" => Some(WhtRateCategory::Royalty),
                        "rent" => Some(WhtRateCategory::Rent),
                        "commission" => Some(WhtRateCategory::Commission),
                        _ => None,
                    })
                    .unwrap_or(WhtRateCategory::ContractGeneral);
                Ok(self.compute_wht(&req.base_amount, &category))
            }
            "cit" => {
                let annual_revenue = req.annual_revenue.unwrap_or(BigDecimal::from(100_000_001));
                Ok(self.compute_cit(&req.base_amount, &annual_revenue))
            }
            "edu_tax" | "education_tax" => Ok(self.compute_education_tax(&req.base_amount)),
            "cgt" => Ok(self.compute_cgt(&req.base_amount)),
            _ => Err(anyhow!("Unsupported tax type: {}", req.tax_type)),
        }
    }

    pub async fn generate_tax_schedule(
        &self,
        company_id: Uuid,
        tax_type: TaxType,
        from_date: chrono::NaiveDate,
        to_date: chrono::NaiveDate,
    ) -> Result<Vec<TaxTransaction>> {
        let transactions = sqlx::query_as::<_, TaxTransaction>(
            "SELECT * FROM tax_transactions WHERE company_id = $1 AND tax_type = $2 AND created_at >= $3 AND created_at <= $4 ORDER BY created_at",
        )
        .bind(company_id)
        .bind(&tax_type)
        .bind(from_date)
        .bind(to_date)
        .fetch_all(&self.pool)
        .await?;
        Ok(transactions)
    }

    pub async fn tax_dashboard(&self, company_id: Uuid) -> Result<TaxDashboard> {
        let vat_payable: BigDecimal = sqlx::query_scalar(
            "SELECT COALESCE(SUM(tax_amount), 0) FROM tax_transactions WHERE company_id = $1 AND tax_type = 'vat' AND is_reported = false",
        )
        .bind(company_id)
        .fetch_one(&self.pool)
        .await?;

        let vat_receivable: BigDecimal = BigDecimal::from(0);
        let wht_deducted: BigDecimal = sqlx::query_scalar(
            "SELECT COALESCE(SUM(tax_amount), 0) FROM tax_transactions WHERE company_id = $1 AND tax_type = 'wht' AND is_reported = false",
        )
        .bind(company_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(TaxDashboard {
            vat_payable,
            vat_receivable,
            wht_deducted,
            cit_estimate: BigDecimal::from(0),
            edu_tax_estimate: BigDecimal::from(0),
            pending_returns: vec!["VAT Monthly Return".to_string()],
            upcoming_deadlines: vec!["VAT Return due 21st of following month".to_string()],
        })
    }
}

#[cfg(test)]
mod tests {
    use bigdecimal::BigDecimal;

    use crate::models::tax::WhtRateCategory;
    use crate::services::tax_service::TaxService;

    fn mock_pool() -> sqlx::PgPool {
        sqlx::postgres::PgPoolOptions::new()
            .connect_lazy("postgres://test:test@localhost/test")
            .expect("mock pool")
    }

    #[test]
    fn test_vat_7_5_percent() {
        let svc = TaxService::new(mock_pool());
        let base = BigDecimal::from(1_000_000);
        let result = svc.compute_vat(&base);

        assert_eq!(result.tax_type, "VAT");
        assert_eq!(result.tax_amount, BigDecimal::from(75_000));
        assert_eq!(result.currency, "NGN");
    }

    #[test]
    fn test_wht_5_percent() {
        let svc = TaxService::new(mock_pool());
        let base = BigDecimal::from(500_000);
        let result = svc.compute_wht(&base, &WhtRateCategory::ContractGeneral);

        assert_eq!(result.tax_type, "WHT");
        assert_eq!(result.tax_amount, BigDecimal::from(25_000));
        assert_eq!(result.currency, "NGN");
    }

    #[test]
    fn test_cit_small_business_exemption() {
        let svc = TaxService::new(mock_pool());
        let profit = BigDecimal::from(5_000_000);
        let revenue = BigDecimal::from(40_000_000);
        let result = svc.compute_cit(&profit, &revenue);

        assert_eq!(result.tax_type, "CIT");
        assert_eq!(result.tax_amount, BigDecimal::from(0));
        assert_eq!(result.rate, BigDecimal::from(0));
    }

    #[test]
    fn test_cit_medium_company() {
        let svc = TaxService::new(mock_pool());
        let profit = BigDecimal::from(50_000_000);
        let revenue = BigDecimal::from(150_000_000);
        let result = svc.compute_cit(&profit, &revenue);

        assert_eq!(result.tax_type, "CIT");
        assert_eq!(result.rate, BigDecimal::from(15));
        assert_eq!(result.tax_amount, BigDecimal::from(7_500_000));
    }

    #[test]
    fn test_cit_large_company() {
        let svc = TaxService::new(mock_pool());
        let profit = BigDecimal::from(200_000_000);
        let revenue = BigDecimal::from(500_000_000);
        let result = svc.compute_cit(&profit, &revenue);

        assert_eq!(result.tax_type, "CIT");
        assert_eq!(result.rate, BigDecimal::from(25));
        assert_eq!(result.tax_amount, BigDecimal::from(50_000_000));
    }

    #[test]
    fn test_education_tax_1_percent() {
        let svc = TaxService::new(mock_pool());
        let profit = BigDecimal::from(50_000_000);
        let result = svc.compute_education_tax(&profit);

        assert_eq!(result.tax_type, "Education Tax");
        assert_eq!(result.rate, BigDecimal::from(1));
        assert_eq!(result.tax_amount, BigDecimal::from(500_000));
        assert_eq!(result.currency, "NGN");
    }

    #[test]
    fn test_cgt_progressive_10_percent() {
        let svc = TaxService::new(mock_pool());
        let gain = BigDecimal::from(30_000_000);
        let result = svc.compute_cgt(&gain);

        assert_eq!(result.tax_type, "CGT");
        assert_eq!(result.rate, BigDecimal::from(10));
        assert_eq!(result.tax_amount, BigDecimal::from(3_000_000));
    }

    #[test]
    fn test_cgt_progressive_15_percent() {
        let svc = TaxService::new(mock_pool());
        let gain = BigDecimal::from(100_000_000);
        let result = svc.compute_cgt(&gain);

        assert_eq!(result.tax_type, "CGT");
        assert_eq!(result.rate, BigDecimal::from(15));
        assert_eq!(result.tax_amount, BigDecimal::from(15_000_000));
    }

    #[test]
    fn test_cgt_progressive_20_percent() {
        let svc = TaxService::new(mock_pool());
        let gain = BigDecimal::from(300_000_000);
        let result = svc.compute_cgt(&gain);

        assert_eq!(result.tax_type, "CGT");
        assert_eq!(result.rate, BigDecimal::from(20));
        assert_eq!(result.tax_amount, BigDecimal::from(60_000_000));
    }
}
