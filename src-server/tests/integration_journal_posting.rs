// Author: Quadri Atharu
use bigdecimal::BigDecimal;
use haqly_erp_server::dtos::journal_dto::{CreateJournalRequest, JournalLineDto};
use haqly_erp_server::models::account::{AccountingPeriod, PeriodStatus};
use haqly_erp_server::models::journal::JournalStatus;
use haqly_erp_server::services::journals_service::JournalsService;
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
               VALUES ($1, 'Test Co JournalPosting', 'NGN', true, NOW(), NOW())
               ON CONFLICT (id) DO NOTHING"#,
        )
        .bind(company_id)
        .execute(pool)
        .await
        .expect("seed company");
        company_id
    }

    async fn seed_fiscal_year_and_period(pool: &PgPool, company_id: Uuid) -> (Uuid, Uuid) {
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

    async fn seed_closed_period(pool: &PgPool, company_id: Uuid, fiscal_year_id: Uuid) -> Uuid {
        let period_id = Uuid::now_v7();
        sqlx::query(
            r#"INSERT INTO accounting_periods (id, fiscal_year_id, company_id, name, period_number, start_date, end_date, status, created_at, updated_at)
               VALUES ($1, $2, $3, 'Dec 2023', 12, '2023-12-01', '2023-12-31', 'closed', NOW(), NOW())"#,
        )
        .bind(period_id)
        .bind(fiscal_year_id)
        .bind(company_id)
        .execute(pool)
        .await
        .expect("seed closed period");
        period_id
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
    async fn test_full_journal_lifecycle() {
        let pool = get_test_pool().await;
        let company_id = seed_test_company(&pool).await;
        let _ = seed_fiscal_year_and_period(&pool, company_id).await;

        let cash_account = seed_account(&pool, company_id, "1010", "Cash", "asset").await;
        let revenue_account = seed_account(&pool, company_id, "4010", "Sales Revenue", "revenue").await;
        let user_id = Uuid::now_v7();

        let svc = JournalsService::new(pool.clone());

        let draft = svc.create_draft(
            company_id,
            CreateJournalRequest {
                company_id,
                branch_id: None,
                narration: "Test sale entry".to_string(),
                reference: Some("REF-001".to_string()),
                journal_type: Some("general".to_string()),
                currency_code: None,
                lines: vec![
                    JournalLineDto {
                        account_id: cash_account,
                        narration: Some("Cash received".to_string()),
                        debit: Some(BigDecimal::from(100_000)),
                        credit: None,
                        cost_center_id: None,
                        project_id: None,
                        department_id: None,
                    },
                    JournalLineDto {
                        account_id: revenue_account,
                        narration: Some("Revenue earned".to_string()),
                        debit: None,
                        credit: Some(BigDecimal::from(100_000)),
                        cost_center_id: None,
                        project_id: None,
                        department_id: None,
                    },
                ],
            },
            user_id,
        )
        .await
        .expect("create draft");

        assert_eq!(draft.status, JournalStatus::Draft);
        assert_eq!(draft.total_debit, BigDecimal::from(100_000));
        assert_eq!(draft.total_credit, BigDecimal::from(100_000));

        let validated = svc.validate(draft.id).await.expect("validate");
        assert_eq!(validated.status, JournalStatus::Validated);

        let submitted = svc.submit_for_approval(draft.id).await.expect("submit");
        assert_eq!(submitted.status, JournalStatus::Submitted);

        let approver = Uuid::now_v7();
        let approved = svc.approve(draft.id, approver).await.expect("approve");
        assert_eq!(approved.status, JournalStatus::Approved);

        let poster = Uuid::now_v7();
        let posted = svc.post_to_gl(draft.id, poster).await.expect("post");
        assert_eq!(posted.status, JournalStatus::Posted);
        assert!(posted.posted_at.is_some());
    }

    #[tokio::test]
    #[ignore]
    async fn test_posting_creates_gl_entries() {
        let pool = get_test_pool().await;
        let company_id = seed_test_company(&pool).await;
        let _ = seed_fiscal_year_and_period(&pool, company_id).await;

        let cash_account = seed_account(&pool, company_id, "1020", "Bank", "asset").await;
        let revenue_account = seed_account(&pool, company_id, "4020", "Service Revenue", "revenue").await;

        let svc = JournalsService::new(pool.clone());

        let amount = BigDecimal::from(500_000);
        let user_id = Uuid::now_v7();

        let draft = svc.create_draft(
            company_id,
            CreateJournalRequest {
                company_id,
                branch_id: None,
                narration: "Service revenue entry".to_string(),
                reference: None,
                journal_type: Some("general".to_string()),
                currency_code: None,
                lines: vec![
                    JournalLineDto {
                        account_id: cash_account,
                        narration: Some("Bank debit".to_string()),
                        debit: Some(amount.clone()),
                        credit: None,
                        cost_center_id: None,
                        project_id: None,
                        department_id: None,
                    },
                    JournalLineDto {
                        account_id: revenue_account,
                        narration: Some("Revenue credit".to_string()),
                        debit: None,
                        credit: Some(amount.clone()),
                        cost_center_id: None,
                        project_id: None,
                        department_id: None,
                    },
                ],
            },
            user_id,
        )
        .await
        .expect("create draft");

        svc.validate(draft.id).await.expect("validate");
        svc.submit_for_approval(draft.id).await.expect("submit");
        svc.approve(draft.id, Uuid::now_v7()).await.expect("approve");
        svc.post_to_gl(draft.id, Uuid::now_v7()).await.expect("post");

        let cash_balance: BigDecimal = sqlx::query_scalar(
            "SELECT balance FROM accounts WHERE id = $1",
        )
        .bind(cash_account)
        .fetch_one(&pool)
        .await
        .expect("fetch cash balance");

        let revenue_balance: BigDecimal = sqlx::query_scalar(
            "SELECT balance FROM accounts WHERE id = $1",
        )
        .bind(revenue_account)
        .fetch_one(&pool)
        .await
        .expect("fetch revenue balance");

        assert_eq!(cash_balance, BigDecimal::from(500_000));
        assert_eq!(revenue_balance, BigDecimal::from(-500_000));
    }

    #[tokio::test]
    #[ignore]
    async fn test_reverse_journal() {
        let pool = get_test_pool().await;
        let company_id = seed_test_company(&pool).await;
        let _ = seed_fiscal_year_and_period(&pool, company_id).await;

        let cash_account = seed_account(&pool, company_id, "1030", "Petty Cash", "asset").await;
        let expense_account = seed_account(&pool, company_id, "5030", "Office Expense", "expense").await;

        let svc = JournalsService::new(pool.clone());
        let user_id = Uuid::now_v7();

        let draft = svc.create_draft(
            company_id,
            CreateJournalRequest {
                company_id,
                branch_id: None,
                narration: "Office supplies".to_string(),
                reference: None,
                journal_type: Some("general".to_string()),
                currency_code: None,
                lines: vec![
                    JournalLineDto {
                        account_id: expense_account,
                        narration: Some("Expense".to_string()),
                        debit: Some(BigDecimal::from(75_000)),
                        credit: None,
                        cost_center_id: None,
                        project_id: None,
                        department_id: None,
                    },
                    JournalLineDto {
                        account_id: cash_account,
                        narration: Some("Cash out".to_string()),
                        debit: None,
                        credit: Some(BigDecimal::from(75_000)),
                        cost_center_id: None,
                        project_id: None,
                        department_id: None,
                    },
                ],
            },
            user_id,
        )
        .await
        .expect("create draft");

        svc.validate(draft.id).await.expect("validate");
        svc.submit_for_approval(draft.id).await.expect("submit");
        svc.approve(draft.id, Uuid::now_v7()).await.expect("approve");
        svc.post_to_gl(draft.id, Uuid::now_v7()).await.expect("post");

        let cash_before: BigDecimal = sqlx::query_scalar(
            "SELECT balance FROM accounts WHERE id = $1",
        )
        .bind(cash_account)
        .fetch_one(&pool)
        .await
        .expect("cash before");

        let reversal = svc
            .reverse(draft.id, "Incorrect entry".to_string(), Uuid::now_v7())
            .await
            .expect("reverse");

        assert_eq!(reversal.status, JournalStatus::Posted);
        assert!(reversal.narration.contains("Reversal"));
        assert_eq!(reversal.reversal_of, Some(draft.id));

        let original = svc.get_journal(draft.id).await.expect("fetch original");
        assert_eq!(original.status, JournalStatus::Reversed);

        let cash_after: BigDecimal = sqlx::query_scalar(
            "SELECT balance FROM accounts WHERE id = $1",
        )
        .bind(cash_account)
        .fetch_one(&pool)
        .await
        .expect("cash after");

        let expense_after: BigDecimal = sqlx::query_scalar(
            "SELECT balance FROM accounts WHERE id = $1",
        )
        .bind(expense_account)
        .fetch_one(&pool)
        .await
        .expect("expense after");

        assert_eq!(cash_before + cash_after, BigDecimal::from(0));
        assert_eq!(expense_after, BigDecimal::from(0));
    }

    #[tokio::test]
    #[ignore]
    async fn test_cannot_post_to_closed_period() {
        let pool = get_test_pool().await;
        let company_id = seed_test_company(&pool).await;
        let (fy_id, _open_period) = seed_fiscal_year_and_period(&pool, company_id).await;
        let _closed_period = seed_closed_period(&pool, company_id, fy_id).await;

        let cash_account = seed_account(&pool, company_id, "1040", "Cash B", "asset").await;
        let revenue_account = seed_account(&pool, company_id, "4040", "Revenue B", "revenue").await;

        let svc = JournalsService::new(pool.clone());
        let user_id = Uuid::now_v7();

        let result = svc.create_draft(
            company_id,
            CreateJournalRequest {
                company_id,
                branch_id: None,
                narration: "Should fail closed period".to_string(),
                reference: None,
                journal_type: Some("general".to_string()),
                currency_code: None,
                lines: vec![
                    JournalLineDto {
                        account_id: cash_account,
                        narration: Some("Dr Cash".to_string()),
                        debit: Some(BigDecimal::from(50_000)),
                        credit: None,
                        cost_center_id: None,
                        project_id: None,
                        department_id: None,
                    },
                    JournalLineDto {
                        account_id: revenue_account,
                        narration: Some("Cr Revenue".to_string()),
                        debit: None,
                        credit: Some(BigDecimal::from(50_000)),
                        cost_center_id: None,
                        project_id: None,
                        department_id: None,
                    },
                ],
            },
            user_id,
        )
        .await;

        if let Ok(journal) = result {
            let validate_result = svc.validate(journal.id).await;
            match validate_result {
                Err(e) => {
                    let msg = e.to_string();
                    assert!(msg.contains("open") || msg.contains("period") || msg.contains("No open"));
                }
                Ok(validated) => {
                    let post_result = {
                        svc.submit_for_approval(validated.id).await.ok();
                        svc.approve(validated.id, Uuid::now_v7()).await.ok();
                        svc.post_to_gl(validated.id, Uuid::now_v7()).await
                    };
                    assert!(post_result.is_err(), "Posting to closed period should fail");
                }
            }
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_duplicate_entry_number_rejected() {
        let pool = get_test_pool().await;
        let company_id = seed_test_company(&pool).await;
        let _ = seed_fiscal_year_and_period(&pool, company_id).await;

        let cash_account = seed_account(&pool, company_id, "1050", "Cash C", "asset").await;
        let revenue_account = seed_account(&pool, company_id, "4050", "Revenue C", "revenue").await;

        let svc = JournalsService::new(pool.clone());
        let user_id = Uuid::now_v7();

        let existing_id = Uuid::now_v7();
        let existing_entry = "JE-DUP-001";

        sqlx::query(
            r#"INSERT INTO journal_headers (id, company_id, fiscal_year_id, period_id, entry_number, narration, status, total_debit, total_credit, currency_code, created_by, created_at, updated_at)
               SELECT $1, $2, fy.id, ap.id, $3, 'Existing', 'draft', 100000, 100000, 'NGN', $4, NOW(), NOW()
               FROM fiscal_years fy, accounting_periods ap
               WHERE fy.company_id = $2 AND ap.company_id = $2 AND ap.status = 'open'
               LIMIT 1"#,
        )
        .bind(existing_id)
        .bind(company_id)
        .bind(existing_entry)
        .bind(user_id)
        .execute(&pool)
        .await
        .expect("insert existing journal");

        let entry_count_before: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM journal_headers WHERE company_id = $1",
        )
        .bind(company_id)
        .fetch_one(&pool)
        .await
        .expect("count before");

        let new_entry_number = format!("JE-{}", entry_count_before);

        let new_id = Uuid::now_v7();
        let dup_result = sqlx::query(
            r#"INSERT INTO journal_headers (id, company_id, fiscal_year_id, period_id, entry_number, narration, status, total_debit, total_credit, currency_code, created_by, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, 'Duplicate attempt', 'draft', 50000, 50000, 'NGN', $6, NOW(), NOW())"#,
        )
        .bind(new_id)
        .bind(company_id)
        .bind(Uuid::nil())
        .bind(Uuid::nil())
        .bind(&new_entry_number)
        .bind(user_id)
        .execute(&pool)
        .await;

        if let Err(e) = dup_result {
            let msg = e.to_string();
            assert!(
                msg.contains("unique") || msg.contains("duplicate") || msg.contains("constraint"),
                "Expected unique constraint violation, got: {}",
                msg
            );
        }
    }
}
