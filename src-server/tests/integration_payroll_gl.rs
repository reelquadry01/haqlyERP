// Author: Quadri Atharu
use bigdecimal::{BigDecimal, FromPrimitive};
use haqly_erp_server::dtos::journal_dto::{CreateJournalRequest, JournalLineDto};
use haqly_erp_server::models::journal::JournalStatus;
use haqly_erp_server::models::payroll::{EmploymentType, PayrollRunStatus};
use haqly_erp_server::services::journals_service::JournalsService;
use haqly_erp_server::services::payroll_service::PayrollService;
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
               VALUES ($1, 'Test Co PayrollGL', 'NGN', true, NOW(), NOW())
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

    async fn seed_payroll_accounts(pool: &PgPool, company_id: Uuid) {
        let accounts = vec![
            ("5101", "Salary Expense", "expense"),
            ("5102", "Employer Contributions", "expense"),
            ("2101", "Net Salary Payable", "liability"),
            ("2105", "PAYE Payable", "liability"),
            ("2106", "Pension Payable", "liability"),
            ("2107", "NHF Payable", "liability"),
        ];

        for (code, name, account_type) in accounts {
            let id = Uuid::now_v7();
            sqlx::query(
                r#"INSERT INTO accounts (id, company_id, code, name, account_type, is_active, allowed_posting, currency_code, balance, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, true, true, 'NGN', 0, NOW(), NOW())
                   ON CONFLICT (company_id, code) DO NOTHING"#,
            )
            .bind(id)
            .bind(company_id)
            .bind(code)
            .bind(name)
            .bind(account_type)
            .execute(pool)
            .await
            .expect("seed payroll account");
        }
    }

    async fn seed_employee(
        pool: &PgPool,
        company_id: Uuid,
        employee_number: &str,
        first_name: &str,
        last_name: &str,
        salary: BigDecimal,
    ) -> Uuid {
        let id = Uuid::now_v7();
        sqlx::query(
            r#"INSERT INTO employees (id, company_id, employee_number, first_name, last_name, email, phone, hire_date, employment_type, salary_amount, currency_code, tax_id, pension_provider, bank_name, bank_account_number, is_active, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, 'test@haqly.com', '08012345678', '2023-01-01', 'full_time', $6, 'NGN', 'TIN-001', 'Pension Co', 'First Bank', '0012345678', true, NOW(), NOW())"#,
        )
        .bind(id)
        .bind(company_id)
        .bind(employee_number)
        .bind(first_name)
        .bind(last_name)
        .bind(salary)
        .execute(pool)
        .await
        .expect("seed employee");
        id
    }

    #[tokio::test]
    #[ignore]
    async fn test_payroll_run_creates_payslips() {
        let pool = get_test_pool().await;
        let company_id = seed_test_company(&pool).await;
        let (fy_id, period_id) = seed_fiscal_year_and_open_period(&pool, company_id).await;

        let emp1 = seed_employee(&pool, company_id, "EMP-001", "Ada", "Okonkwo", BigDecimal::from(300_000)).await;
        let emp2 = seed_employee(&pool, company_id, "EMP-002", "Bola", "Adeyemi", BigDecimal::from(500_000)).await;
        let emp3 = seed_employee(&pool, company_id, "EMP-003", "Chidi", "Nwosu", BigDecimal::from(200_000)).await;

        let payroll_svc = PayrollService::new(pool.clone());

        let payroll_run = payroll_svc
            .create_payroll_run(company_id, fy_id, period_id, chrono::NaiveDate::from_ymd_opt(2024, 1, 31).unwrap())
            .await
            .expect("create payroll run");

        assert_eq!(payroll_run.status, PayrollRunStatus::Draft);

        let processed = payroll_svc
            .process_payroll(payroll_run.id, Uuid::now_v7())
            .await
            .expect("process payroll");

        assert_eq!(processed.status, PayrollRunStatus::Processed);

        let payslips = payroll_svc
            .generate_payslips(payroll_run.id)
            .await
            .expect("generate payslips");

        assert_eq!(payslips.len(), 3);

        for payslip in &payslips {
            assert!(payslip.paye > BigDecimal::from(0));
            assert!(payslip.pension_employee > BigDecimal::from(0));
            assert!(payslip.pension_employer > BigDecimal::from(0));
            assert!(payslip.nhf > BigDecimal::from(0));
            assert!(payslip.net_pay > BigDecimal::from(0));
            assert_eq!(payslip.total_gross, &payslip.basic_salary + &payslip.housing_allowance + &payslip.transport_allowance + &payslip.meal_allowance + &payslip.utility_allowance);
        }

        let total_gross: BigDecimal = payslips.iter().map(|p| p.total_gross.clone()).sum();
        assert_eq!(total_gross, processed.total_gross);

        let total_net: BigDecimal = payslips.iter().map(|p| p.net_pay.clone()).sum();
        assert_eq!(total_net, processed.total_net);
    }

    #[tokio::test]
    #[ignore]
    async fn test_payroll_gl_posting() {
        let pool = get_test_pool().await;
        let company_id = seed_test_company(&pool).await;
        let (fy_id, period_id) = seed_fiscal_year_and_open_period(&pool, company_id).await;
        seed_payroll_accounts(&pool, company_id).await;

        let _emp1 = seed_employee(&pool, company_id, "EMP-010", "Dayo", "Ojo", BigDecimal::from(400_000)).await;
        let _emp2 = seed_employee(&pool, company_id, "EMP-011", "Efe", "Igbinosa", BigDecimal::from(350_000)).await;

        let payroll_svc = PayrollService::new(pool.clone());

        let payroll_run = payroll_svc
            .create_payroll_run(company_id, fy_id, period_id, chrono::NaiveDate::from_ymd_opt(2024, 1, 31).unwrap())
            .await
            .expect("create payroll run");

        let processed = payroll_svc
            .process_payroll(payroll_run.id, Uuid::now_v7())
            .await
            .expect("process payroll");

        assert_eq!(processed.status, PayrollRunStatus::Processed);

        let posted = payroll_svc
            .post_payroll_to_gl(payroll_run.id, Uuid::now_v7())
            .await
            .expect("post payroll to GL");

        assert_eq!(posted.status, PayrollRunStatus::Posted);

        let journal: (Uuid, String,) = sqlx::query_as(
            "SELECT id, entry_number FROM journal_headers WHERE source_module = 'payroll' AND source_document_id = $1",
        )
        .bind(payroll_run.id)
        .fetch_one(&pool)
        .await
        .expect("find payroll journal");

        let lines: Vec<(Uuid, BigDecimal, BigDecimal)> = sqlx::query_as(
            "SELECT account_id, debit, credit FROM journal_lines WHERE journal_header_id = $1 ORDER BY line_number",
        )
        .bind(journal.0)
        .fetch_all(&pool)
        .await
        .expect("fetch journal lines");

        assert!(lines.len() >= 3);

        let total_debit: BigDecimal = lines.iter().map(|l| l.1.clone()).sum();
        let total_credit: BigDecimal = lines.iter().map(|l| l.2.clone()).sum();

        assert_eq!(total_debit, processed.total_gross + processed.total_employer_contributions);

        let salary_expense_debit: BigDecimal = lines
            .iter()
            .filter(|l| l.1 > BigDecimal::from(0))
            .map(|l| l.1.clone())
            .sum();

        assert!(salary_expense_debit > BigDecimal::from(0));

        let payslips = payroll_svc
            .generate_payslips(payroll_run.id)
            .await
            .expect("payslips");

        let total_paye: BigDecimal = payslips.iter().map(|p| p.paye.clone()).sum();
        let total_pension_emp: BigDecimal = payslips.iter().map(|p| p.pension_employee.clone()).sum();
        let total_pension_er: BigDecimal = payslips.iter().map(|p| p.pension_employer.clone()).sum();
        let total_nhf: BigDecimal = payslips.iter().map(|p| p.nhf.clone()).sum();

        assert!(total_paye > BigDecimal::from(0));
        assert!(total_pension_emp > BigDecimal::from(0));
        assert!(total_pension_er > BigDecimal::from(0));
        assert!(total_nhf > BigDecimal::from(0));
    }

    #[tokio::test]
    #[ignore]
    async fn test_paye_brackets_nigeria() {
        let pool = get_test_pool().await;
        let payroll_svc = PayrollService::new(pool.clone());

        let annual_salary = BigDecimal::from(5_000_000);
        let annual_taxable = payroll_svc.compute_annual_taxable_income(&annual_salary);

        let housing = &annual_salary * BigDecimal::from_f64(0.20).unwrap();
        let transport = &annual_salary * BigDecimal::from_f64(0.05).unwrap();
        let meal = &annual_salary * BigDecimal::from_f64(0.05).unwrap();
        let utility = &annual_salary * BigDecimal::from_f64(0.025).unwrap();
        let pension = &annual_salary * BigDecimal::from_f64(0.08).unwrap();
        let nhf = &annual_salary * BigDecimal::from_f64(0.025).unwrap();
        let total_relief = &housing + &transport + &meal + &utility + &pension + &nhf;
        let expected_taxable = &annual_salary - &total_relief;

        let taxable_f64 = annual_taxable.to_string().parse::<f64>().unwrap();
        let expected_f64 = expected_taxable.to_string().parse::<f64>().unwrap();
        assert!((taxable_f64 - expected_f64).abs() < 1.0);

        let paye = payroll_svc.compute_paye(&annual_taxable);
        let paye_f64 = paye.to_string().parse::<f64>().unwrap();

        // PAYE brackets per Tax Reform Acts 2025 (effective 2026)
        let bracket1 = 800_000.0 * 0.00;
        let bracket2 = taxable_f64.min(3_200_000.0).max(800_000.0) - 800_000.0;
        let bracket2 = bracket2.max(0.0) * 0.15;
        let bracket3 = (taxable_f64.min(7_200_000.0) - 3_200_000.0).max(0.0) * 0.20;
        let bracket4 = (taxable_f64.min(14_000_000.0) - 7_200_000.0).max(0.0) * 0.25;
        let bracket5 = (taxable_f64.min(25_000_000.0) - 14_000_000.0).max(0.0) * 0.30;
        let bracket6 = (taxable_f64 - 25_000_000.0).max(0.0) * 0.35;

        let expected_paye = bracket1 + bracket2 + bracket3 + bracket4 + bracket5 + bracket6;
        assert!((paye_f64 - expected_paye).abs() < 50.0);

        assert!((bracket1 - 0.0).abs() < 1.0);
        assert!(bracket2 >= 0.0);
        assert!(bracket3 >= 0.0);

        assert!(paye_f64 > 0.0);
        assert!(paye_f64 < annual_salary.to_string().parse::<f64>().unwrap());
    }

    #[tokio::test]
    #[ignore]
    async fn test_pension_employee_employer_split() {
        let pool = get_test_pool().await;
        let company_id = seed_test_company(&pool).await;
        let (fy_id, period_id) = seed_fiscal_year_and_open_period(&pool, company_id).await;

        let monthly_salary = BigDecimal::from(500_000);
        let _emp = seed_employee(&pool, company_id, "EMP-020", "Femi", "Bakare", monthly_salary.clone()).await;

        let payroll_svc = PayrollService::new(pool.clone());

        let payroll_run = payroll_svc
            .create_payroll_run(company_id, fy_id, period_id, chrono::NaiveDate::from_ymd_opt(2024, 1, 31).unwrap())
            .await
            .expect("create payroll run");

        payroll_svc
            .process_payroll(payroll_run.id, Uuid::now_v7())
            .await
            .expect("process payroll");

        let payslips = payroll_svc
            .generate_payslips(payroll_run.id)
            .await
            .expect("payslips");

        assert_eq!(payslips.len(), 1);

        let payslip = &payslips[0];

        let expected_employee_pension = &monthly_salary * BigDecimal::from_f64(0.08).unwrap();
        let expected_employer_pension = &monthly_salary * BigDecimal::from_f64(0.10).unwrap();

        let pension_emp_f64 = payslip.pension_employee.to_string().parse::<f64>().unwrap();
        let expected_emp_f64 = expected_employee_pension.to_string().parse::<f64>().unwrap();
        assert!((pension_emp_f64 - expected_emp_f64).abs() < 1.0);

        let pension_er_f64 = payslip.pension_employer.to_string().parse::<f64>().unwrap();
        let expected_er_f64 = expected_employer_pension.to_string().parse::<f64>().unwrap();
        assert!((pension_er_f64 - expected_er_f64).abs() < 1.0);

        let total_pension = &payslip.pension_employee + &payslip.pension_employer;
        let total_f64 = total_pension.to_string().parse::<f64>().unwrap();
        let expected_total_f64 = monthly_salary.to_string().parse::<f64>().unwrap() * 0.18;
        assert!((total_f64 - expected_total_f64).abs() < 1.0);

        let nhf_f64 = payslip.nhf.to_string().parse::<f64>().unwrap();
        let expected_nhf_f64 = monthly_salary.to_string().parse::<f64>().unwrap() * 0.025;
        assert!((nhf_f64 - expected_nhf_f64).abs() < 1.0);
    }
}
