// Author: Quadri Atharu
use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Loan {
    pub id: Uuid,
    pub company_id: Uuid,
    pub lender_name: String,
    pub principal_amount: BigDecimal,
    pub interest_rate: BigDecimal,
    pub loan_type: String,
    pub start_date: NaiveDate,
    pub maturity_date: NaiveDate,
    pub payment_frequency: String,
    pub status: String,
    pub outstanding_balance: BigDecimal,
    pub disbursed_amount: BigDecimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LoanDisbursement {
    pub id: Uuid,
    pub loan_id: Uuid,
    pub amount: BigDecimal,
    pub disbursement_date: NaiveDate,
    pub bank_account_id: Option<Uuid>,
    pub reference: Option<String>,
    pub posted_to_gl: bool,
    pub gl_journal_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LoanRepayment {
    pub id: Uuid,
    pub loan_id: Uuid,
    pub amount: BigDecimal,
    pub principal_portion: BigDecimal,
    pub interest_portion: BigDecimal,
    pub fee_portion: BigDecimal,
    pub payment_date: NaiveDate,
    pub bank_account_id: Option<Uuid>,
    pub reference: Option<String>,
    pub posted_to_gl: bool,
    pub gl_journal_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AmortizationEntry {
    pub id: Uuid,
    pub loan_id: Uuid,
    pub period_number: i32,
    pub payment_date: NaiveDate,
    pub opening_balance: BigDecimal,
    pub payment_amount: BigDecimal,
    pub principal_portion: BigDecimal,
    pub interest_portion: BigDecimal,
    pub closing_balance: BigDecimal,
    pub is_paid: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewLoan {
    pub company_id: Uuid,
    pub lender_name: String,
    pub principal_amount: BigDecimal,
    pub interest_rate: BigDecimal,
    pub loan_type: String,
    pub start_date: NaiveDate,
    pub maturity_date: NaiveDate,
    pub payment_frequency: String,
    pub outstanding_balance: BigDecimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewLoanDisbursement {
    pub loan_id: Uuid,
    pub amount: BigDecimal,
    pub disbursement_date: NaiveDate,
    pub bank_account_id: Option<Uuid>,
    pub reference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewLoanRepayment {
    pub loan_id: Uuid,
    pub amount: BigDecimal,
    pub principal_portion: BigDecimal,
    pub interest_portion: BigDecimal,
    pub fee_portion: BigDecimal,
    pub payment_date: NaiveDate,
    pub bank_account_id: Option<Uuid>,
    pub reference: Option<String>,
}

pub struct LoanRepo {
    pool: PgPool,
}

impl LoanRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_loan(&self, new_loan: NewLoan) -> Result<Loan, sqlx::Error> {
        sqlx::query_as::<_, Loan>(
            r#"INSERT INTO loans (company_id, lender_name, principal_amount, interest_rate, loan_type, start_date, maturity_date, payment_frequency, outstanding_balance)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, company_id, lender_name, principal_amount, interest_rate, loan_type, start_date, maturity_date, payment_frequency, status, outstanding_balance, disbursed_amount, created_at, updated_at"#,
        )
        .bind(new_loan.company_id)
        .bind(&new_loan.lender_name)
        .bind(&new_loan.principal_amount)
        .bind(&new_loan.interest_rate)
        .bind(&new_loan.loan_type)
        .bind(new_loan.start_date)
        .bind(new_loan.maturity_date)
        .bind(&new_loan.payment_frequency)
        .bind(&new_loan.outstanding_balance)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn find_loans(&self, company_id: Uuid) -> Result<Vec<Loan>, sqlx::Error> {
        sqlx::query_as::<_, Loan>(
            "SELECT id, company_id, lender_name, principal_amount, interest_rate, loan_type, start_date, maturity_date, payment_frequency, status, outstanding_balance, disbursed_amount, created_at, updated_at FROM loans WHERE company_id = $1 ORDER BY start_date DESC",
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn record_disbursement(
        &self,
        new_disb: NewLoanDisbursement,
    ) -> Result<LoanDisbursement, sqlx::Error> {
        sqlx::query_as::<_, LoanDisbursement>(
            r#"INSERT INTO loan_disbursements (loan_id, amount, disbursement_date, bank_account_id, reference)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, loan_id, amount, disbursement_date, bank_account_id, reference, posted_to_gl, gl_journal_id, created_at"#,
        )
        .bind(new_disb.loan_id)
        .bind(&new_disb.amount)
        .bind(new_disb.disbursement_date)
        .bind(new_disb.bank_account_id)
        .bind(&new_disb.reference)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn record_repayment(
        &self,
        new_repay: NewLoanRepayment,
    ) -> Result<LoanRepayment, sqlx::Error> {
        sqlx::query_as::<_, LoanRepayment>(
            r#"INSERT INTO loan_repayments (loan_id, amount, principal_portion, interest_portion, fee_portion, payment_date, bank_account_id, reference)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, loan_id, amount, principal_portion, interest_portion, fee_portion, payment_date, bank_account_id, reference, posted_to_gl, gl_journal_id, created_at"#,
        )
        .bind(new_repay.loan_id)
        .bind(&new_repay.amount)
        .bind(&new_repay.principal_portion)
        .bind(&new_repay.interest_portion)
        .bind(&new_repay.fee_portion)
        .bind(new_repay.payment_date)
        .bind(new_repay.bank_account_id)
        .bind(&new_repay.reference)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn generate_amortization(
        &self,
        loan_id: Uuid,
        entries: Vec<AmortizationEntry>,
    ) -> Result<Vec<AmortizationEntry>, sqlx::Error> {
        for entry in &entries {
            sqlx::query(
                r#"INSERT INTO loan_amortization_schedule (loan_id, period_number, payment_date, opening_balance, payment_amount, principal_portion, interest_portion, closing_balance, is_paid)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#,
            )
            .bind(entry.loan_id)
            .bind(entry.period_number)
            .bind(entry.payment_date)
            .bind(&entry.opening_balance)
            .bind(&entry.payment_amount)
            .bind(&entry.principal_portion)
            .bind(&entry.interest_portion)
            .bind(&entry.closing_balance)
            .bind(entry.is_paid)
            .execute(&self.pool)
            .await?;
        }
        Ok(entries)
    }
}
