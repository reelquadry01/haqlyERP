// Author: Quadri Atharu
use anyhow::{anyhow, Result};
use bigdecimal::{BigDecimal, FromPrimitive};
use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::payroll::{Employee, EmploymentType, PayrollRun, PayrollRunStatus, Payslip};

fn safe_bd(val: f64) -> BigDecimal {
    BigDecimal::from_f64(val).unwrap_or_else(|| BigDecimal::from(0))
}

const HOUSING_ALLOWANCE_PERCENT: f64 = 0.20;
const TRANSPORT_ALLOWANCE_PERCENT: f64 = 0.05;
const MEAL_ALLOWANCE_PERCENT: f64 = 0.05;
const UTILITY_ALLOWANCE_PERCENT: f64 = 0.025;
const PENSION_EMPLOYEE_PERCENT: f64 = 0.08;
const PENSION_EMPLOYER_PERCENT: f64 = 0.10;
const NHF_PERCENT: f64 = 0.025;
const NSITF_PERCENT: f64 = 0.01;
const ITF_PERCENT: f64 = 0.01;

#[derive(Clone)]
pub struct PayrollService {
    pub pool: PgPool,
}

impl PayrollService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_employee(
        &self,
        company_id: Uuid,
        employee_number: String,
        first_name: String,
        last_name: String,
        email: Option<String>,
        phone: Option<String>,
        hire_date: NaiveDate,
        employment_type: EmploymentType,
        designation: Option<String>,
        grade_level: Option<String>,
        salary_amount: BigDecimal,
        currency_code: String,
        tax_id: Option<String>,
        pension_provider: Option<String>,
        bank_name: Option<String>,
        bank_account_number: Option<String>,
        branch_id: Option<Uuid>,
        department_id: Option<Uuid>,
    ) -> Result<Employee> {
        let id = Uuid::now_v7();
        sqlx::query(
            r#"INSERT INTO employees (id, company_id, branch_id, department_id, employee_number, first_name, last_name, email, phone, hire_date, termination_date, employment_type, designation, grade_level, salary_amount, currency_code, tax_id, pension_provider, bank_name, bank_account_number, is_active, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NULL, $11, $12, $13, $14, $15, $16, $17, $18, $19, true, NOW(), NOW())"#,
        )
        .bind(id)
        .bind(company_id)
        .bind(branch_id)
        .bind(department_id)
        .bind(&employee_number)
        .bind(&first_name)
        .bind(&last_name)
        .bind(&email)
        .bind(&phone)
        .bind(hire_date)
        .bind(&employment_type)
        .bind(&designation)
        .bind(&grade_level)
        .bind(&salary_amount)
        .bind(&currency_code)
        .bind(&tax_id)
        .bind(&pension_provider)
        .bind(&bank_name)
        .bind(&bank_account_number)
        .execute(&self.pool)
        .await?;

        self.get_employee(id).await
    }

    pub async fn get_employee(&self, id: Uuid) -> Result<Employee> {
        sqlx::query_as::<_, Employee>("SELECT * FROM employees WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| anyhow!("Employee not found"))
    }

    pub async fn list_employees(&self, company_id: Uuid) -> Result<Vec<Employee>> {
        sqlx::query_as::<_, Employee>(
            "SELECT * FROM employees WHERE company_id = $1 AND is_active = true ORDER BY employee_number",
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| anyhow!("Failed to list employees: {}", e))
    }

    pub async fn update_employee(&self, id: Uuid, updates: Vec<(&str, String)>) -> Result<Employee> {
        let employee = self.get_employee(id).await?;
        if !employee.is_active {
            return Err(anyhow!("Cannot update inactive employee"));
        }

        for (field, value) in updates {
            let query = match field {
                "first_name" => sqlx::query("UPDATE employees SET first_name = $1, updated_at = NOW() WHERE id = $2").bind(&value).bind(id),
                "last_name" => sqlx::query("UPDATE employees SET last_name = $1, updated_at = NOW() WHERE id = $2").bind(&value).bind(id),
                "email" => sqlx::query("UPDATE employees SET email = $1, updated_at = NOW() WHERE id = $2").bind(&value).bind(id),
                "phone" => sqlx::query("UPDATE employees SET phone = $1, updated_at = NOW() WHERE id = $2").bind(&value).bind(id),
                "designation" => sqlx::query("UPDATE employees SET designation = $1, updated_at = NOW() WHERE id = $2").bind(&value).bind(id),
                "grade_level" => sqlx::query("UPDATE employees SET grade_level = $1, updated_at = NOW() WHERE id = $2").bind(&value).bind(id),
                "tax_id" => sqlx::query("UPDATE employees SET tax_id = $1, updated_at = NOW() WHERE id = $2").bind(&value).bind(id),
                "pension_provider" => sqlx::query("UPDATE employees SET pension_provider = $1, updated_at = NOW() WHERE id = $2").bind(&value).bind(id),
                "bank_name" => sqlx::query("UPDATE employees SET bank_name = $1, updated_at = NOW() WHERE id = $2").bind(&value).bind(id),
                "bank_account_number" => sqlx::query("UPDATE employees SET bank_account_number = $1, updated_at = NOW() WHERE id = $2").bind(&value).bind(id),
                _ => continue,
            };
            query.execute(&self.pool).await?;
        }

        self.get_employee(id).await
    }

    pub async fn terminate_employee(&self, id: Uuid, termination_date: NaiveDate) -> Result<Employee> {
        let employee = self.get_employee(id).await?;
        if !employee.is_active {
            return Err(anyhow!("Employee already terminated"));
        }

        sqlx::query(
            "UPDATE employees SET is_active = false, termination_date = $1, updated_at = NOW() WHERE id = $2",
        )
        .bind(termination_date)
        .bind(id)
        .execute(&self.pool)
        .await?;

        self.get_employee(id).await
    }

    pub async fn create_payroll_run(
        &self,
        company_id: Uuid,
        fiscal_year_id: Uuid,
        period_id: Uuid,
        run_date: NaiveDate,
    ) -> Result<PayrollRun> {
        let id = Uuid::now_v7();
        sqlx::query(
            r#"INSERT INTO payroll_runs (id, company_id, fiscal_year_id, period_id, run_date, status, total_gross, total_deductions, total_net, total_employer_contributions, created_at)
               VALUES ($1, $2, $3, $4, $5, 'draft', 0, 0, 0, 0, NOW())"#,
        )
        .bind(id)
        .bind(company_id)
        .bind(fiscal_year_id)
        .bind(period_id)
        .bind(run_date)
        .execute(&self.pool)
        .await?;

        sqlx::query_as::<_, PayrollRun>("SELECT * FROM payroll_runs WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch payroll run: {}", e))
    }

    pub fn compute_annual_taxable_income(
        &self,
        annual_gross: &BigDecimal,
    ) -> BigDecimal {
        let housing = annual_gross * safe_bd(HOUSING_ALLOWANCE_PERCENT);
        let transport = annual_gross * safe_bd(TRANSPORT_ALLOWANCE_PERCENT);
        let meal = annual_gross * safe_bd(MEAL_ALLOWANCE_PERCENT);
        let utility = annual_gross * safe_bd(UTILITY_ALLOWANCE_PERCENT);
        let pension = annual_gross * safe_bd(PENSION_EMPLOYEE_PERCENT);
        let nhf = annual_gross * safe_bd(NHF_PERCENT);

        let total_tax_free = &housing + &transport + &meal + &utility + &pension + &nhf;
        annual_gross - total_tax_free
    }

    pub fn compute_paye(&self, annual_taxable_income: &BigDecimal) -> BigDecimal {
        let brackets: [(f64, f64); 6] = [
            (300_000.0, 0.07),
            (300_000.0, 0.11),
            (500_000.0, 0.15),
            (500_000.0, 0.19),
            (1_600_000.0, 0.21),
            (f64::MAX, 0.24),
        ];

        let mut remaining = annual_taxable_income.to_string().parse::<f64>().unwrap_or(0.0);
        if remaining <= 0.0 {
            return BigDecimal::from(0);
        }

        let mut tax = 0.0_f64;
        for (bracket_width, rate) in brackets {
            if remaining <= 0.0 {
                break;
            }
            let taxable = remaining.min(bracket_width);
            tax += taxable * rate;
            remaining -= taxable;
        }

        BigDecimal::from_f64(tax).unwrap_or(BigDecimal::from(0))
    }

    pub async fn process_payroll(&self, payroll_run_id: Uuid, processed_by: Uuid) -> Result<PayrollRun> {
        let run = sqlx::query_as::<_, PayrollRun>("SELECT * FROM payroll_runs WHERE id = $1")
            .bind(payroll_run_id)
            .fetch_one(&self.pool)
            .await?;

        if run.status != PayrollRunStatus::Draft {
            return Err(anyhow!("Payroll run must be in Draft status to process"));
        }

        let employees = sqlx::query_as::<_, Employee>(
            "SELECT * FROM employees WHERE company_id = $1 AND is_active = true",
        )
        .bind(run.company_id)
        .fetch_all(&self.pool)
        .await?;

        let mut tx = self.pool.begin().await.map_err(|e| anyhow!("Failed to begin transaction: {}", e))?;

        let mut total_gross = BigDecimal::from(0);
        let mut total_deductions = BigDecimal::from(0);
        let mut total_net = BigDecimal::from(0);
        let mut total_employer_contributions = BigDecimal::from(0);

        for emp in &employees {
            let annual_salary = &emp.salary_amount * BigDecimal::from(12);
            let monthly_basic = &emp.salary_amount;
            let housing = monthly_basic * safe_bd(HOUSING_ALLOWANCE_PERCENT);
            let transport = monthly_basic * safe_bd(TRANSPORT_ALLOWANCE_PERCENT);
            let meal = monthly_basic * safe_bd(MEAL_ALLOWANCE_PERCENT);
            let utility = monthly_basic * safe_bd(UTILITY_ALLOWANCE_PERCENT);

            let monthly_gross = &monthly_basic + &housing + &transport + &meal + &utility;

            let annual_taxable = self.compute_annual_taxable_income(&annual_salary);
            let annual_paye = self.compute_paye(&annual_taxable);
            let monthly_paye = &annual_paye / BigDecimal::from(12);

            let pension_employee = monthly_basic * safe_bd(PENSION_EMPLOYEE_PERCENT);
            let pension_employer = monthly_basic * safe_bd(PENSION_EMPLOYER_PERCENT);
            let nhf = monthly_basic * safe_bd(NHF_PERCENT);
            let nsitf = monthly_basic * safe_bd(NSITF_PERCENT);
            let itf_levy = monthly_basic * safe_bd(ITF_PERCENT);

            let monthly_deductions = &monthly_paye + &pension_employee + &nhf + &nsitf + &itf_levy;
            let monthly_net = &monthly_gross - &monthly_deductions;

            let payslip_id = Uuid::now_v7();
            sqlx::query(
                r#"INSERT INTO payslips (id, payroll_run_id, employee_id, basic_salary, housing_allowance, transport_allowance, meal_allowance, utility_allowance, other_allowances, total_gross, paye, pension_employee, pension_employer, nhf, nsitf, itf_levy, other_deductions, total_deductions, net_pay, currency_code, created_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 0, $9, $10, $11, $12, $13, $14, $15, 0, $16, $17, $18, NOW())"#,
            )
            .bind(payslip_id)
            .bind(payroll_run_id)
            .bind(emp.id)
            .bind(monthly_basic)
            .bind(&housing)
            .bind(&transport)
            .bind(&meal)
            .bind(&utility)
            .bind(&monthly_gross)
            .bind(&monthly_paye)
            .bind(&pension_employee)
            .bind(&pension_employer)
            .bind(&nhf)
            .bind(&nsitf)
            .bind(&itf_levy)
            .bind(&monthly_deductions)
            .bind(&monthly_net)
            .bind(&emp.currency_code)
            .execute(&mut *tx)
            .await?;

            total_gross = total_gross + monthly_gross;
            total_deductions = total_deductions + monthly_deductions;
            total_net = total_net + monthly_net;
            total_employer_contributions = total_employer_contributions + pension_employer.clone() + nsitf.clone() + itf_levy.clone();
        }

        sqlx::query(
            r#"UPDATE payroll_runs SET status = 'processed', total_gross = $1, total_deductions = $2, total_net = $3, total_employer_contributions = $4, processed_by = $5, processed_at = NOW() WHERE id = $6"#,
        )
        .bind(&total_gross)
        .bind(&total_deductions)
        .bind(&total_net)
        .bind(&total_employer_contributions)
        .bind(processed_by)
        .bind(payroll_run_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await.map_err(|e| anyhow!("Failed to commit payroll processing: {}", e))?;

        sqlx::query_as::<_, PayrollRun>("SELECT * FROM payroll_runs WHERE id = $1")
            .bind(payroll_run_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch payroll run: {}", e))
    }

    pub async fn generate_payslips(&self, payroll_run_id: Uuid) -> Result<Vec<Payslip>> {
        sqlx::query_as::<_, Payslip>(
            "SELECT * FROM payslips WHERE payroll_run_id = $1 ORDER BY created_at",
        )
        .bind(payroll_run_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| anyhow!("Failed to generate payslips: {}", e))
    }

    pub async fn get_payslip(&self, payslip_id: Uuid) -> Result<Payslip> {
        sqlx::query_as::<_, Payslip>("SELECT * FROM payslips WHERE id = $1")
            .bind(payslip_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| anyhow!("Payslip not found"))
    }

    pub async fn post_payroll_to_gl(&self, payroll_run_id: Uuid, posted_by: Uuid) -> Result<PayrollRun> {
        let run = sqlx::query_as::<_, PayrollRun>("SELECT * FROM payroll_runs WHERE id = $1")
            .bind(payroll_run_id)
            .fetch_one(&self.pool)
            .await?;

        if run.status != PayrollRunStatus::Processed {
            return Err(anyhow!("Payroll run must be in Processed status to post"));
        }

        let payslips = self.generate_payslips(payroll_run_id).await?;
        if payslips.is_empty() {
            return Err(anyhow!("No payslips found for payroll run"));
        }

        let currency = payslips[0].currency_code.clone();

        let payroll_expense_account: Option<Uuid> = sqlx::query_scalar(
            "SELECT id FROM accounts WHERE company_id = $1 AND code = '5101' AND is_active = true LIMIT 1",
        )
        .bind(run.company_id)
        .fetch_optional(&self.pool)
        .await?;

        let paye_payable_account: Option<Uuid> = sqlx::query_scalar(
            "SELECT id FROM accounts WHERE company_id = $1 AND code = '2105' AND is_active = true LIMIT 1",
        )
        .bind(run.company_id)
        .fetch_optional(&self.pool)
        .await?;

        let pension_payable_account: Option<Uuid> = sqlx::query_scalar(
            "SELECT id FROM accounts WHERE company_id = $1 AND code = '2106' AND is_active = true LIMIT 1",
        )
        .bind(run.company_id)
        .fetch_optional(&self.pool)
        .await?;

        let nhf_payable_account: Option<Uuid> = sqlx::query_scalar(
            "SELECT id FROM accounts WHERE company_id = $1 AND code = '2107' AND is_active = true LIMIT 1",
        )
        .bind(run.company_id)
        .fetch_optional(&self.pool)
        .await?;

        let net_salary_payable_account: Option<Uuid> = sqlx::query_scalar(
            "SELECT id FROM accounts WHERE company_id = $1 AND code = '2101' AND is_active = true LIMIT 1",
        )
        .bind(run.company_id)
        .fetch_optional(&self.pool)
        .await?;

        let employer_contributions_account: Option<Uuid> = sqlx::query_scalar(
            "SELECT id FROM accounts WHERE company_id = $1 AND code = '5102' AND is_active = true LIMIT 1",
        )
        .bind(run.company_id)
        .fetch_optional(&self.pool)
        .await?;

        let payroll_expense = payroll_expense_account.ok_or_else(|| anyhow!("Payroll expense account (5101) not found"))?;
        let net_salary_payable = net_salary_payable_account.ok_or_else(|| anyhow!("Net salary payable account (2101) not found"))?;

        let journal_id = Uuid::now_v7();
        let entry_number = format!("PR-{}", payroll_run_id);

        let total_debit = run.total_gross.clone();
        let total_credit = run.total_net.clone();

        let mut tx = self.pool.begin().await.map_err(|e| anyhow!("Failed to begin transaction: {}", e))?;

        sqlx::query(
            r#"INSERT INTO journal_headers (id, company_id, fiscal_year_id, period_id, entry_number, narration, status, journal_type, source_module, source_document_id, total_debit, total_credit, currency_code, posted_at, posted_by, created_by, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, 'posted', 'auto', 'payroll', $7, $8, $9, $10, NOW(), $11, $11, NOW(), NOW())"#,
        )
        .bind(journal_id)
        .bind(run.company_id)
        .bind(run.fiscal_year_id)
        .bind(run.period_id)
        .bind(&entry_number)
        .bind("Payroll posting")
        .bind(payroll_run_id)
        .bind(&total_debit)
        .bind(&total_credit)
        .bind(&currency)
        .bind(posted_by)
        .execute(&mut *tx)
        .await?;

        let mut line_num = 1i32;

        sqlx::query(
            r#"INSERT INTO journal_lines (id, journal_header_id, account_id, line_number, narration, debit, credit, currency_code, created_at)
               VALUES ($1, $2, $3, $4, 'Payroll expense', $5, 0, $6, NOW())"#,
        )
        .bind(Uuid::now_v7())
        .bind(journal_id)
        .bind(payroll_expense)
        .bind(line_num)
        .bind(&run.total_gross)
        .bind(&currency)
        .execute(&mut *tx)
        .await?;
        line_num += 1;

        if let Some(paye_acc) = paye_payable_account {
            let total_paye: BigDecimal = payslips.iter().map(|p| p.paye.clone()).sum();
            if total_paye != BigDecimal::from(0) {
                sqlx::query(
                    r#"INSERT INTO journal_lines (id, journal_header_id, account_id, line_number, narration, debit, credit, currency_code, created_at)
                       VALUES ($1, $2, $3, $4, 'PAYE payable', 0, $5, $6, NOW())"#,
                )
                .bind(Uuid::now_v7())
                .bind(journal_id)
                .bind(paye_acc)
                .bind(line_num)
                .bind(&total_paye)
                .bind(&currency)
                .execute(&mut *tx)
                .await?;
                line_num += 1;
            }
        }

        if let Some(pension_acc) = pension_payable_account {
            let total_pension_emp: BigDecimal = payslips.iter().map(|p| p.pension_employee.clone()).sum();
            let total_pension_er: BigDecimal = payslips.iter().map(|p| p.pension_employer.clone()).sum();
            let total_pension = &total_pension_emp + &total_pension_er;
            if total_pension != BigDecimal::from(0) {
                sqlx::query(
                    r#"INSERT INTO journal_lines (id, journal_header_id, account_id, line_number, narration, debit, credit, currency_code, created_at)
                       VALUES ($1, $2, $3, $4, 'Pension payable', 0, $5, $6, NOW())"#,
                )
                .bind(Uuid::now_v7())
                .bind(journal_id)
                .bind(pension_acc)
                .bind(line_num)
                .bind(&total_pension)
                .bind(&currency)
                .execute(&mut *tx)
                .await?;
                line_num += 1;
            }
        }

        if let Some(nhf_acc) = nhf_payable_account {
            let total_nhf: BigDecimal = payslips.iter().map(|p| p.nhf.clone()).sum();
            if total_nhf != BigDecimal::from(0) {
                sqlx::query(
                    r#"INSERT INTO journal_lines (id, journal_header_id, account_id, line_number, narration, debit, credit, currency_code, created_at)
                       VALUES ($1, $2, $3, $4, 'NHF payable', 0, $5, $6, NOW())"#,
                )
                .bind(Uuid::now_v7())
                .bind(journal_id)
                .bind(nhf_acc)
                .bind(line_num)
                .bind(&total_nhf)
                .bind(&currency)
                .execute(&mut *tx)
                .await?;
                line_num += 1;
            }
        }

        sqlx::query(
            r#"INSERT INTO journal_lines (id, journal_header_id, account_id, line_number, narration, debit, credit, currency_code, created_at)
               VALUES ($1, $2, $3, $4, 'Net salary payable', 0, $5, $6, NOW())"#,
        )
        .bind(Uuid::now_v7())
        .bind(journal_id)
        .bind(net_salary_payable)
        .bind(line_num)
        .bind(&run.total_net)
        .bind(&currency)
        .execute(&mut *tx)
        .await?;
        line_num += 1;

        if let Some(er_acc) = employer_contributions_account {
            let er_contributions = run.total_employer_contributions.clone();
            if er_contributions != BigDecimal::from(0) {
                sqlx::query(
                    r#"INSERT INTO journal_lines (id, journal_header_id, account_id, line_number, narration, debit, credit, currency_code, created_at)
                       VALUES ($1, $2, $3, $4, 'Employer contributions', $5, 0, $6, NOW())"#,
                )
                .bind(Uuid::now_v7())
                .bind(journal_id)
                .bind(er_acc)
                .bind(line_num)
                .bind(&er_contributions)
                .bind(&currency)
                .execute(&mut *tx)
                .await?;
            }
        }

        sqlx::query(
            "UPDATE payroll_runs SET status = 'posted' WHERE id = $1",
        )
        .bind(payroll_run_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await.map_err(|e| anyhow!("Failed to commit payroll GL posting: {}", e))?;

        sqlx::query_as::<_, PayrollRun>("SELECT * FROM payroll_runs WHERE id = $1")
            .bind(payroll_run_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch payroll run: {}", e))
    }
}
