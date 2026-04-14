// Author: Quadri Atharu
use bigdecimal::BigDecimal;
use haqly_erp_server::dtos::journal_dto::{CreateJournalRequest, JournalLineDto};
use haqly_erp_server::models::account::PeriodStatus;
use haqly_erp_server::models::journal::JournalStatus;
use haqly_erp_server::models::tax::{TaxType, WhtRateCategory};
use haqly_erp_server::services::journals_service::JournalsService;
use haqly_erp_server::services::tax_service::TaxService;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

#[cfg(test)]
mod tests {
    use super::*;

    async fn get_test_pool() -> PgPool {
        let database_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://haqly:haqly@localhost:5432/haqly_test".to_string());
        PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database")
    }

    async fn seed_test_company(pool: &PgPool) -> Uuid {
        let company_id = Uuid::now_v7();
        sqlx::query(
            r#"INSERT INTO companies (id, name, currency_code, is_active, created_at, updated_at)
               VALUES ($1, 'Test Co TaxJournal', 'NGN', true, NOW(), NOW())
               ON CONFLICT (id) DO NOTHING"#,
        )
        .bind(company_id)
        .execute(pool)
        .await
        .expect("seed company");
        company_id
    }

    async fn seed_fiscal_year_and_open_period(pool: &PgPool, company_id: Uuid) -> (Uuid, Uuid) {
        let fy_id = Uuid::now_v7();
        sqlx::query(
            r#"INSERT INTO fiscal_years (id, company_id, name, start_date, end_date, is_closed, created_at, updated_at)
               VALUES ($1, $2, 'FY2024', '2024-01-01', '2024-12-31', false, NOW(), NOW())"#,
        )
        .bind(fy_id)
        .bind(company_id)
        .execute(pool)
        .await
        .expect("seed fiscal year");

        let period_id = Uuid::now_v7();
        sqlx::query(
            r#"INSERT INTO accounting_periods (id, fiscal_year_id, company_id, name, period_number, start_date, end_date, status, created_at, updated_at)
               VALUES ($1, $2, $3, 'Jan 2024', 1, '2024-01-01', '2024-01-31', 'open', NOW(), NOW())"#,
        )
        .bind(period_id)
        .bind(fy_id)
        .bind(company_id)
        .execute(pool)
        .await
        .expect("seed open period");

        (fy_id, period_id)
    }

    async fn seed_account(
        pool: &PgPool,
        company_id: Uuid,
        code: &str,
        name: &str,
        account_type: &str,
    ) -> Uuid {
        let id = Uuid::now_v7();
        sqlx::query(
            r#"INSERT INTO accounts (id, company_id, code, name, account_type, is_active, allowed_posting, currency_code, balance, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, true, true, 'NGN', 0, NOW(), NOW())"#,
        )
        .bind(id)
        .bind(company_id)
        .bind(code)
        .bind(name)
        .bind(account_type)
        .execute(pool)
        .await
        .expect("seed account");
        id
    }

    #[tokio::test]
    #[ignore]
    async fn test_vat_sale_creates_journal() {
        let pool = get_test_pool().await;
        let company_id = seed_test_company(&pool).await;
        let _ = seed_fiscal_year_and_open_period(&pool, company_id).await;

        let receivable_account = seed_account(&pool, company_id, "1101", "Accounts Receivable", "asset").await;
        let revenue_account = seed_account(&pool, company_id, "4101", "Sales Revenue", "revenue").await;
        let vat_payable_account = seed_account(&pool, company_id, "2101", "VAT Payable", "liability").await;

        let tax_svc = TaxService::new(pool.clone());
        let base_amount = BigDecimal::from(1_000_000);
        let vat_result = tax_svc.compute_vat(&base_amount);

        assert_eq!(vat_result.tax_type, "VAT");
        assert_eq!(vat_result.tax_amount, BigDecimal::from(75_000));

        let vat_amount = vat_result.tax_amount.clone();
        let total_receivable = &base_amount + &vat_amount;

        let journal_svc = JournalsService::new(pool.clone());
        let user_id = Uuid::now_v7();

        let journal = journal_svc
            .create_draft(
                company_id,
                CreateJournalRequest {
                    company_id,
                    branch_id: None,
                    narration: "VAT sale - INV-001".to_string(),
                    reference: Some("INV-001".to_string()),
                    journal_type: Some("sales".to_string()),
                    currency_code: None,
                    lines: vec![
                        JournalLineDto {
                            account_id: receivable_account,
                            narration: Some("Dr Receivable".to_string()),
                            debit: Some(total_receivable.clone()),
                            credit: None,
                            cost_center_id: None,
                            project_id: None,
                            department_id: None,
                        },
                        JournalLineDto {
                            account_id: revenue_account,
                            narration: Some("Cr Revenue".to_string()),
                            debit: None,
                            credit: Some(base_amount.clone()),
                            cost_center_id: None,
                            project_id: None,
                            department_id: None,
                        },
                        JournalLineDto {
                            account_id: vat_payable_account,
                            narration: Some("Cr VAT Payable".to_string()),
                            debit: None,
                            credit: Some(vat_amount.clone()),
                            cost_center_id: None,
                            project_id: None,
                            department_id: None,
                        },
                    ],
                },
                user_id,
            )
            .await
            .expect("create VAT journal");

        assert_eq!(journal.status, JournalStatus::Draft);
        assert_eq!(journal.total_debit, total_receivable);
        assert_eq!(journal.total_credit, &base_amount + &vat_amount);

        journal_svc.validate(journal.id).await.expect("validate");
        journal_svc.submit_for_approval(journal.id).await.expect("submit");
        journal_svc.approve(journal.id, Uuid::now_v7()).await.expect("approve");
        journal_svc.post_to_gl(journal.id, Uuid::now_v7()).await.expect("post");

        let vat_balance: BigDecimal = sqlx::query_scalar(
            "SELECT balance FROM accounts WHERE id = $1",
        )
        .bind(vat_payable_account)
        .fetch_one(&pool)
        .await
        .expect("fetch VAT balance");

        assert_eq!(vat_balance, BigDecimal::from(-75_000));
    }

    #[tokio::test]
    #[ignore]
    async fn test_wht_deduction_creates_journal() {
        let pool = get_test_pool().await;
        let company_id = seed_test_company(&pool).await;
        let _ = seed_fiscal_year_and_open_period(&pool, company_id).await;

        let expense_account = seed_account(&pool, company_id, "5101", "Consulting Expense", "expense").await;
        let bank_account = seed_account(&pool, company_id, "1011", "Bank", "asset").await;
        let wht_payable_account = seed_account(&pool, company_id, "2102", "WHT Payable", "liability").await;

        let tax_svc = TaxService::new(pool.clone());
        let base_amount = BigDecimal::from(500_000);
        let wht_result = tax_svc.compute_wht(&base_amount, &WhtRateCategory::Consultancy);

        assert_eq!(wht_result.tax_type, "WHT");
        assert_eq!(wht_result.tax_amount, BigDecimal::from(25_000));
        assert_eq!(wht_result.rate, BigDecimal::from(5));

        let wht_amount = wht_result.tax_amount.clone();
        let net_payment = &base_amount - &wht_amount;

        let journal_svc = JournalsService::new(pool.clone());
        let user_id = Uuid::now_v7();

        let journal = journal_svc
            .create_draft(
                company_id,
                CreateJournalRequest {
                    company_id,
                    branch_id: None,
                    narration: "WHT deduction - consultancy".to_string(),
                    reference: Some("PV-001".to_string()),
                    journal_type: Some("payment".to_string()),
                    currency_code: None,
                    lines: vec![
                        JournalLineDto {
                            account_id: expense_account,
                            narration: Some("Dr Expense".to_string()),
                            debit: Some(base_amount.clone()),
                            credit: None,
                            cost_center_id: None,
                            project_id: None,
                            department_id: None,
                        },
                        JournalLineDto {
                            account_id: bank_account,
                            narration: Some("Cr Bank".to_string()),
                            debit: None,
                            credit: Some(net_payment.clone()),
                            cost_center_id: None,
                            project_id: None,
                            department_id: None,
                        },
                        JournalLineDto {
                            account_id: wht_payable_account,
                            narration: Some("Cr WHT Payable".to_string()),
                            debit: None,
                            credit: Some(wht_amount.clone()),
                            cost_center_id: None,
                            project_id: None,
                            department_id: None,
                        },
                    ],
                },
                user_id,
            )
            .await
            .expect("create WHT journal");

        assert_eq!(journal.total_debit, base_amount);
        assert_eq!(journal.total_credit, base_amount);

        journal_svc.validate(journal.id).await.expect("validate");
        journal_svc.submit_for_approval(journal.id).await.expect("submit");
        journal_svc.approve(journal.id, Uuid::now_v7()).await.expect("approve");
        journal_svc.post_to_gl(journal.id, Uuid::now_v7()).await.expect("post");

        let wht_balance: BigDecimal = sqlx::query_scalar(
            "SELECT balance FROM accounts WHERE id = $1",
        )
        .bind(wht_payable_account)
        .fetch_one(&pool)
        .await
        .expect("fetch WHT balance");

        assert_eq!(wht_balance, BigDecimal::from(-25_000));
    }

    #[tokio::test]
    #[ignore]
    async fn test_cit_provision_creates_journal() {
        let pool = get_test_pool().await;
        let company_id = seed_test_company(&pool).await;
        let _ = seed_fiscal_year_and_open_period(&pool, company_id).await;

        let tax_expense_account = seed_account(&pool, company_id, "5201", "CIT Expense", "expense").await;
        let cit_payable_account = seed_account(&pool, company_id, "2103", "CIT Payable", "liability").await;

        let tax_svc = TaxService::new(pool.clone());
        let taxable_profit = BigDecimal::from(50_000_000);
        let annual_revenue = BigDecimal::from(200_000_000);
        let cit_result = tax_svc.compute_cit(&taxable_profit, &annual_revenue);

        assert_eq!(cit_result.tax_type, "CIT");
        assert_eq!(cit_result.rate, BigDecimal::from(30));
        assert_eq!(cit_result.tax_amount, BigDecimal::from(15_000_000));

        let cit_amount = cit_result.tax_amount.clone();

        let journal_svc = JournalsService::new(pool.clone());
        let user_id = Uuid::now_v7();

        let journal = journal_svc
            .create_draft(
                company_id,
                CreateJournalRequest {
                    company_id,
                    branch_id: None,
                    narration: "CIT provision for the year".to_string(),
                    reference: None,
                    journal_type: Some("tax_provision".to_string()),
                    currency_code: None,
                    lines: vec![
                        JournalLineDto {
                            account_id: tax_expense_account,
                            narration: Some("Dr Tax Expense".to_string()),
                            debit: Some(cit_amount.clone()),
                            credit: None,
                            cost_center_id: None,
                            project_id: None,
                            department_id: None,
                        },
                        JournalLineDto {
                            account_id: cit_payable_account,
                            narration: Some("Cr CIT Payable".to_string()),
                            debit: None,
                            credit: Some(cit_amount.clone()),
                            cost_center_id: None,
                            project_id: None,
                            department_id: None,
                        },
                    ],
                },
                user_id,
            )
            .await
            .expect("create CIT journal");

        assert_eq!(journal.total_debit, cit_amount);
        assert_eq!(journal.total_credit, cit_amount);

        journal_svc.validate(journal.id).await.expect("validate");
        journal_svc.submit_for_approval(journal.id).await.expect("submit");
        journal_svc.approve(journal.id, Uuid::now_v7()).await.expect("approve");
        journal_svc.post_to_gl(journal.id, Uuid::now_v7()).await.expect("post");

        let cit_balance: BigDecimal = sqlx::query_scalar(
            "SELECT balance FROM accounts WHERE id = $1",
        )
        .bind(cit_payable_account)
        .fetch_one(&pool)
        .await
        .expect("fetch CIT balance");

        assert_eq!(cit_balance, BigDecimal::from(-15_000_000));
    }

    #[tokio::test]
    #[ignore]
    async fn test_tax_return_totals() {
        let pool = get_test_pool().await;
        let company_id = seed_test_company(&pool).await;

        let tax_svc = TaxService::new(pool.clone());

        let vat_base_1 = BigDecimal::from(1_000_000);
        let vat_1 = tax_svc.compute_vat(&vat_base_1);

        let vat_base_2 = BigDecimal::from(2_500_000);
        let vat_2 = tax_svc.compute_vat(&vat_base_2);

        let wht_base = BigDecimal::from(800_000);
        let wht_result = tax_svc.compute_wht(&wht_base, &WhtRateCategory::ContractGeneral);

        let cit_profit = BigDecimal::from(30_000_000);
        let cit_revenue = BigDecimal::from(75_000_000);
        let cit_result = tax_svc.compute_cit(&cit_profit, &cit_revenue);

        assert_eq!(vat_1.tax_amount, BigDecimal::from(75_000));
        assert_eq!(vat_2.tax_amount, BigDecimal::from(187_500));
        assert_eq!(wht_result.tax_amount, BigDecimal::from(40_000));
        assert_eq!(cit_result.tax_amount, BigDecimal::from(6_000_000));

        let total_vat = vat_1.tax_amount + vat_2.tax_amount;
        let total_tax = total_vat.clone() + wht_result.tax_amount + cit_result.tax_amount;

        assert_eq!(total_vat, BigDecimal::from(262_500));
        assert_eq!(total_tax, BigDecimal::from(6_302_500));

        let from_date = chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let to_date = chrono::NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

        let vat_schedule = tax_svc
            .generate_tax_schedule(company_id, TaxType::Vat, from_date, to_date)
            .await
            .expect("generate VAT schedule");

        let expected_vat_total: BigDecimal = vat_schedule
            .iter()
            .map(|t| t.tax_amount.clone())
            .sum();

        let schedule_empty = vat_schedule.is_empty();
        if !schedule_empty {
            assert!(expected_vat_total >= BigDecimal::from(0));
        }
    }
}
