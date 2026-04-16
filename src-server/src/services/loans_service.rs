// Author: Quadri Atharu
use anyhow::{anyhow, Result};
use bigdecimal::{BigDecimal, FromPrimitive};
use chrono::{Datelike, NaiveDate};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::loan::{
    AmortizationScheduleEntry, Loan, LoanDisbursement, LoanRepayment, LoanType,
};

#[derive(Clone)]
pub struct LoansService {
    pub pool: PgPool,
}

impl LoansService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_loan(
        &self,
        company_id: Uuid,
        branch_id: Option<Uuid>,
        loan_type: LoanType,
        lender_name: String,
        principal_amount: BigDecimal,
        interest_rate: BigDecimal,
        tenure_months: i32,
        start_date: NaiveDate,
        currency_code: String,
        loan_account_id: Uuid,
        interest_account_id: Uuid,
        bank_account_id: Option<Uuid>,
        narration: Option<String>,
        created_by: Uuid,
    ) -> Result<Loan> {
        let maturity_date = if tenure_months <= 0 {
            start_date
        } else {
            let years = tenure_months / 12;
            let remaining_months = tenure_months % 12;
            let mut year = start_date.year() + years;
            let mut month = start_date.month() as i32 + remaining_months;
            if month > 12 {
                year += (month - 1) / 12;
                month = ((month - 1) % 12) + 1;
            }
            NaiveDate::from_ymd_opt(year, month as u32, start_date.day())
                .unwrap_or(start_date)
        };

        let id = Uuid::now_v7();
        sqlx::query(
            r#"INSERT INTO loans (id, company_id, branch_id, loan_type, lender_name, principal_amount, interest_rate, tenure_months, start_date, maturity_date, outstanding_principal, outstanding_interest, currency_code, status, loan_account_id, interest_account_id, bank_account_id, narration, created_by, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $6, 0, $11, 'active', $12, $13, $14, $15, $16, NOW(), NOW())"#,
        )
        .bind(id)
        .bind(company_id)
        .bind(branch_id)
        .bind(&loan_type)
        .bind(&lender_name)
        .bind(&principal_amount)
        .bind(&interest_rate)
        .bind(tenure_months)
        .bind(start_date)
        .bind(maturity_date)
        .bind(&currency_code)
        .bind(loan_account_id)
        .bind(interest_account_id)
        .bind(bank_account_id)
        .bind(&narration)
        .bind(created_by)
        .execute(&self.pool)
        .await?;

        sqlx::query_as::<_, Loan>("SELECT * FROM loans WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch loan: {}", e))
    }

    pub fn generate_amortization_schedule(
        loan: &Loan,
    ) -> Vec<AmortizationScheduleEntryInput> {
        let principal = loan.principal_amount.to_string().parse::<f64>().unwrap_or(0.0);
        let annual_rate = loan.interest_rate.to_string().parse::<f64>().unwrap_or(0.0) / 100.0;
        let monthly_rate = annual_rate / 12.0;
        let n = loan.tenure_months as f64;

        let monthly_payment = if monthly_rate > 0.0 && n > 0.0 {
            let factor = (1.0 + monthly_rate).powf(n);
            principal * monthly_rate * factor / (factor - 1.0)
        } else if n > 0.0 {
            principal / n
        } else {
            0.0
        };

        let mut schedule = Vec::new();
        let mut balance = principal;
        let mut period = 1i32;
        let mut current_date = loan.start_date;

        while balance > 0.01 && period <= loan.tenure_months {
            let interest_payment = balance * monthly_rate;
            let principal_payment = if (monthly_payment - interest_payment) > balance {
                balance
            } else {
                monthly_payment - interest_payment
            };
            let total_payment = interest_payment + principal_payment;
            let closing_balance = balance - principal_payment;

            if closing_balance < 0.01 {
                break;
            }

            let next_month = if current_date.month() == 12 {
                NaiveDate::from_ymd_opt(current_date.year() + 1, 1, 1).unwrap()
            } else {
                NaiveDate::from_ymd_opt(current_date.year(), current_date.month() + 1, 1).unwrap()
            };

            schedule.push(AmortizationScheduleEntryInput {
                period_number: period,
                payment_date: next_month,
                opening_balance: BigDecimal::from_f64(balance).unwrap_or(BigDecimal::from(0)),
                principal_payment: BigDecimal::from_f64(principal_payment).unwrap_or(BigDecimal::from(0)),
                interest_payment: BigDecimal::from_f64(interest_payment).unwrap_or(BigDecimal::from(0)),
                total_payment: BigDecimal::from_f64(total_payment).unwrap_or(BigDecimal::from(0)),
                closing_balance: BigDecimal::from_f64(closing_balance).unwrap_or(BigDecimal::from(0)),
            });

            balance = closing_balance;
            current_date = next_month;
            period += 1;
        }

        schedule
    }

    pub async fn record_disbursement(
        &self,
        loan_id: Uuid,
        amount: BigDecimal,
        disbursement_date: NaiveDate,
        reference: Option<String>,
        created_by: Uuid,
    ) -> Result<LoanDisbursement> {
        let id = Uuid::now_v7();
        sqlx::query(
            r#"INSERT INTO loan_disbursements (id, loan_id, amount, disbursement_date, reference, created_by, created_at)
               VALUES ($1, $2, $3, $4, $5, $6, NOW())"#,
        )
        .bind(id)
        .bind(loan_id)
        .bind(&amount)
        .bind(disbursement_date)
        .bind(&reference)
        .bind(created_by)
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "UPDATE loans SET outstanding_principal = outstanding_principal + $1, updated_at = NOW() WHERE id = $2",
        )
        .bind(&amount)
        .bind(loan_id)
        .execute(&self.pool)
        .await?;

        sqlx::query_as::<_, LoanDisbursement>("SELECT * FROM loan_disbursements WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch disbursement: {}", e))
    }

    pub async fn record_repayment(
        &self,
        loan_id: Uuid,
        principal_amount: BigDecimal,
        interest_amount: BigDecimal,
        repayment_date: NaiveDate,
        reference: Option<String>,
        created_by: Uuid,
    ) -> Result<LoanRepayment> {
        let id = Uuid::now_v7();
        sqlx::query(
            r#"INSERT INTO loan_repayments (id, loan_id, principal_amount, interest_amount, repayment_date, reference, created_by, created_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())"#,
        )
        .bind(id)
        .bind(loan_id)
        .bind(&principal_amount)
        .bind(&interest_amount)
        .bind(repayment_date)
        .bind(&reference)
        .bind(created_by)
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"UPDATE loans SET outstanding_principal = outstanding_principal - $1, outstanding_interest = outstanding_interest - $2, updated_at = NOW() WHERE id = $3"#,
        )
        .bind(&principal_amount)
        .bind(&interest_amount)
        .bind(loan_id)
        .execute(&self.pool)
        .await?;

        sqlx::query_as::<_, LoanRepayment>("SELECT * FROM loan_repayments WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch repayment: {}", e))
    }

    pub async fn post_disbursement_to_gl(
        &self,
        loan_id: Uuid,
        disbursement_id: Uuid,
        posted_by: Uuid,
    ) -> Result<()> {
        let loan = sqlx::query_as::<_, Loan>("SELECT * FROM loans WHERE id = $1")
            .bind(loan_id)
            .fetch_one(&self.pool)
            .await?;

        let disbursement = sqlx::query_as::<_, LoanDisbursement>(
            "SELECT * FROM loan_disbursements WHERE id = $1",
        )
        .bind(disbursement_id)
        .fetch_one(&self.pool)
        .await?;

        let journal_id = Uuid::now_v7();
        let entry_number = format!("LD-{}", disbursement_id);

        sqlx::query(
            r#"INSERT INTO journal_headers (id, company_id, branch_id, fiscal_year_id, period_id, entry_number, narration, status, journal_type, source_module, source_document_id, total_debit, total_credit, currency_code, posted_at, posted_by, created_by, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, 'Loan disbursement', 'posted', 'auto', 'loans', $7, $8, $8, $9, NOW(), $10, $10, NOW(), NOW())"#,
        )
        .bind(journal_id)
        .bind(loan.company_id)
        .bind(loan.branch_id)
        .bind(Uuid::nil())
        .bind(Uuid::nil())
        .bind(&entry_number)
        .bind(disbursement_id)
        .bind(&disbursement.amount)
        .bind(&loan.currency_code)
        .bind(posted_by)
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"INSERT INTO journal_lines (id, journal_header_id, account_id, line_number, narration, debit, credit, currency_code, created_at)
               VALUES ($1, $2, $3, 1, 'Loan receivable', $4, 0, $5, NOW())"#,
        )
        .bind(Uuid::now_v7())
        .bind(journal_id)
        .bind(loan.loan_account_id)
        .bind(&disbursement.amount)
        .bind(&loan.currency_code)
        .execute(&self.pool)
        .await?;

        let bank_account_id = loan
            .bank_account_id
            .ok_or_else(|| anyhow!("Bank account not configured for loan"))?;

        sqlx::query(
            r#"INSERT INTO journal_lines (id, journal_header_id, account_id, line_number, narration, debit, credit, currency_code, created_at)
               VALUES ($1, $2, $3, 2, 'Bank payment for loan', 0, $4, $5, NOW())"#,
        )
        .bind(Uuid::now_v7())
        .bind(journal_id)
        .bind(bank_account_id)
        .bind(&disbursement.amount)
        .bind(&loan.currency_code)
        .execute(&self.pool)
        .await?;

        sqlx::query("UPDATE loan_disbursements SET journal_header_id = $1 WHERE id = $2")
            .bind(journal_id)
            .bind(disbursement_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn post_repayment_to_gl(
        &self,
        loan_id: Uuid,
        repayment_id: Uuid,
        posted_by: Uuid,
    ) -> Result<()> {
        let loan = sqlx::query_as::<_, Loan>("SELECT * FROM loans WHERE id = $1")
            .bind(loan_id)
            .fetch_one(&self.pool)
            .await?;

        let repayment = sqlx::query_as::<_, LoanRepayment>(
            "SELECT * FROM loan_repayments WHERE id = $1",
        )
        .bind(repayment_id)
        .fetch_one(&self.pool)
        .await?;

        let total = &repayment.principal_amount + &repayment.interest_amount;
        let journal_id = Uuid::now_v7();
        let entry_number = format!("LR-{}", repayment_id);

        sqlx::query(
            r#"INSERT INTO journal_headers (id, company_id, branch_id, fiscal_year_id, period_id, entry_number, narration, status, journal_type, source_module, source_document_id, total_debit, total_credit, currency_code, posted_at, posted_by, created_by, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, 'Loan repayment', 'posted', 'auto', 'loans', $7, $8, $8, $9, NOW(), $10, $10, NOW(), NOW())"#,
        )
        .bind(journal_id)
        .bind(loan.company_id)
        .bind(loan.branch_id)
        .bind(Uuid::nil())
        .bind(Uuid::nil())
        .bind(&entry_number)
        .bind(repayment_id)
        .bind(&total)
        .bind(&loan.currency_code)
        .bind(posted_by)
        .execute(&self.pool)
        .await?;

        let bank_account_id = loan
            .bank_account_id
            .ok_or_else(|| anyhow!("Bank account not configured for loan"))?;

        sqlx::query(
            r#"INSERT INTO journal_lines (id, journal_header_id, account_id, line_number, narration, debit, credit, currency_code, created_at)
               VALUES ($1, $2, $3, 1, 'Bank receipt for loan repayment', $4, 0, $5, NOW())"#,
        )
        .bind(Uuid::now_v7())
        .bind(journal_id)
        .bind(bank_account_id)
        .bind(&total)
        .bind(&loan.currency_code)
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"INSERT INTO journal_lines (id, journal_header_id, account_id, line_number, narration, debit, credit, currency_code, created_at)
               VALUES ($1, $2, $3, 2, 'Loan principal repayment', 0, $4, $5, NOW())"#,
        )
        .bind(Uuid::now_v7())
        .bind(journal_id)
        .bind(loan.loan_account_id)
        .bind(&repayment.principal_amount)
        .bind(&loan.currency_code)
        .execute(&self.pool)
        .await?;

        if repayment.interest_amount != BigDecimal::from(0) {
            sqlx::query(
                r#"INSERT INTO journal_lines (id, journal_header_id, account_id, line_number, narration, debit, credit, currency_code, created_at)
                   VALUES ($1, $2, $3, 3, 'Loan interest repayment', 0, $4, $5, NOW())"#,
            )
            .bind(Uuid::now_v7())
            .bind(journal_id)
            .bind(loan.interest_account_id)
            .bind(&repayment.interest_amount)
            .bind(&loan.currency_code)
            .execute(&self.pool)
            .await?;
        }

        sqlx::query("UPDATE loan_repayments SET journal_header_id = $1 WHERE id = $2")
            .bind(journal_id)
            .bind(repayment_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

struct AmortizationScheduleEntryInput {
    period_number: i32,
    payment_date: NaiveDate,
    opening_balance: BigDecimal,
    principal_payment: BigDecimal,
    interest_payment: BigDecimal,
    total_payment: BigDecimal,
    closing_balance: BigDecimal,
}
