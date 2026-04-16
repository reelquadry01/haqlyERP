// Author: Quadri Atharu
use anyhow::{anyhow, Result};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::voucher::{
    PaymentGatewayRequest, PaymentVoucher, VoucherAttachment, VoucherComment, VoucherLine,
    VoucherStatus,
};

#[derive(Clone)]
pub struct PaymentVouchersService {
    pub pool: PgPool,
}

impl PaymentVouchersService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_draft(
        &self,
        company_id: Uuid,
        branch_id: Option<Uuid>,
        payee_name: String,
        payee_account: Option<String>,
        payee_bank: Option<String>,
        payee_bank_code: Option<String>,
        amount: bigdecimal::BigDecimal,
        currency_code: String,
        narration: Option<String>,
        reference: Option<String>,
        voucher_type: Option<String>,
        due_date: Option<chrono::NaiveDate>,
        supplier_id: Option<Uuid>,
        payment_method: Option<String>,
        created_by: Uuid,
    ) -> Result<PaymentVoucher> {
        let id = Uuid::now_v7();
        let voucher_number = self.generate_voucher_number(company_id).await?;

        sqlx::query(
            r#"INSERT INTO payment_vouchers (id, company_id, branch_id, voucher_number, payee_name, payee_account, payee_bank, payee_bank_code, amount, currency_code, narration, reference, status, voucher_type, due_date, supplier_id, payment_method, created_by, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, 'draft', $13, $14, $15, $16, $17, NOW(), NOW())"#,
        )
        .bind(id)
        .bind(company_id)
        .bind(branch_id)
        .bind(&voucher_number)
        .bind(&payee_name)
        .bind(&payee_account)
        .bind(&payee_bank)
        .bind(&payee_bank_code)
        .bind(&amount)
        .bind(&currency_code)
        .bind(&narration)
        .bind(&reference)
        .bind(&voucher_type)
        .bind(due_date)
        .bind(supplier_id)
        .bind(&payment_method)
        .bind(created_by)
        .execute(&self.pool)
        .await?;

        self.get_voucher(id).await
    }

    pub async fn update(
        &self,
        voucher_id: Uuid,
        payee_name: Option<String>,
        amount: Option<bigdecimal::BigDecimal>,
        narration: Option<String>,
        reference: Option<String>,
        due_date: Option<chrono::NaiveDate>,
        payment_method: Option<String>,
    ) -> Result<PaymentVoucher> {
        let voucher = self.get_voucher(voucher_id).await?;
        if voucher.status != VoucherStatus::Draft {
            return Err(anyhow!("Only draft vouchers can be updated"));
        }

        sqlx::query(
            r#"UPDATE payment_vouchers SET
               payee_name = COALESCE($2, payee_name),
               amount = COALESCE($3, amount),
               narration = COALESCE($4, narration),
               reference = COALESCE($5, reference),
               due_date = COALESCE($6, due_date),
               payment_method = COALESCE($7, payment_method),
               updated_at = NOW()
               WHERE id = $1"#,
        )
        .bind(voucher_id)
        .bind(&payee_name)
        .bind(&amount)
        .bind(&narration)
        .bind(&reference)
        .bind(due_date)
        .bind(&payment_method)
        .execute(&self.pool)
        .await?;

        self.get_voucher(voucher_id).await
    }

    pub async fn validate(&self, voucher_id: Uuid) -> Result<PaymentVoucher> {
        let voucher = self.get_voucher(voucher_id).await?;
        if voucher.status != VoucherStatus::Draft {
            return Err(anyhow!("Only draft vouchers can be validated"));
        }
        if voucher.amount == bigdecimal::BigDecimal::from(0) {
            return Err(anyhow!("Voucher amount cannot be zero"));
        }
        self.update_status(voucher_id, VoucherStatus::Validated).await
    }

    pub async fn submit(&self, voucher_id: Uuid) -> Result<PaymentVoucher> {
        let voucher = self.get_voucher(voucher_id).await?;
        if voucher.status != VoucherStatus::Validated {
            return Err(anyhow!("Only validated vouchers can be submitted"));
        }
        self.update_status(voucher_id, VoucherStatus::Submitted).await
    }

    pub async fn approve(&self, voucher_id: Uuid, approved_by: Uuid) -> Result<PaymentVoucher> {
        let voucher = self.get_voucher(voucher_id).await?;
        if voucher.status != VoucherStatus::Submitted {
            return Err(anyhow!("Only submitted vouchers can be approved"));
        }
        sqlx::query(
            "UPDATE payment_vouchers SET status = 'approved', approved_by = $1, updated_at = NOW() WHERE id = $2",
        )
        .bind(approved_by)
        .bind(voucher_id)
        .execute(&self.pool)
        .await?;
        self.get_voucher(voucher_id).await
    }

    pub async fn reject(&self, voucher_id: Uuid) -> Result<PaymentVoucher> {
        let voucher = self.get_voucher(voucher_id).await?;
        if voucher.status != VoucherStatus::Submitted && voucher.status != VoucherStatus::Approved {
            return Err(anyhow!("Only submitted or approved vouchers can be rejected"));
        }
        self.update_status(voucher_id, VoucherStatus::Draft).await
    }

    pub async fn return_voucher(&self, voucher_id: Uuid) -> Result<PaymentVoucher> {
        let voucher = self.get_voucher(voucher_id).await?;
        if voucher.status != VoucherStatus::Approved {
            return Err(anyhow!("Only approved vouchers can be returned"));
        }
        self.update_status(voucher_id, VoucherStatus::Submitted).await
    }

    pub async fn post(&self, voucher_id: Uuid) -> Result<PaymentVoucher> {
        let voucher = self.get_voucher(voucher_id).await?;
        if voucher.status != VoucherStatus::Approved {
            return Err(anyhow!("Only approved vouchers can be posted"));
        }
        self.update_status(voucher_id, VoucherStatus::Posted).await
    }

    pub async fn initiate_payment(
        &self,
        voucher_id: Uuid,
    ) -> Result<PaymentGatewayRequest> {
        let voucher = self.get_voucher(voucher_id).await?;
        if voucher.status != VoucherStatus::Posted {
            return Err(anyhow!("Only posted vouchers can initiate payment"));
        }
        if voucher.payee_account.is_none() || voucher.payee_bank_code.is_none() {
            return Err(anyhow!("Payee account and bank code required for payment"));
        }

        Ok(PaymentGatewayRequest {
            voucher_id: voucher.id,
            payee_account: voucher.payee_account.ok_or_else(|| anyhow!("Missing payee account"))?,
            payee_bank_code: voucher.payee_bank_code.ok_or_else(|| anyhow!("Missing payee bank code"))?,
            amount: voucher.amount,
            currency: voucher.currency_code,
            narration: voucher.narration.unwrap_or_default(),
            reference: voucher.reference.unwrap_or_default(),
        })
    }

    pub async fn mark_paid(&self, voucher_id: Uuid, paid_by: Uuid, gateway_ref: Option<String>) -> Result<PaymentVoucher> {
        sqlx::query(
            "UPDATE payment_vouchers SET status = 'paid', paid_by = $1, payment_gateway_ref = $2, payment_date = CURRENT_DATE, updated_at = NOW() WHERE id = $3",
        )
        .bind(paid_by)
        .bind(&gateway_ref)
        .bind(voucher_id)
        .execute(&self.pool)
        .await?;
        self.get_voucher(voucher_id).await
    }

    pub async fn cancel(&self, voucher_id: Uuid) -> Result<PaymentVoucher> {
        let voucher = self.get_voucher(voucher_id).await?;
        if voucher.status == VoucherStatus::Paid {
            return Err(anyhow!("Paid vouchers cannot be cancelled"));
        }
        self.update_status(voucher_id, VoucherStatus::Cancelled).await
    }

    pub async fn add_comment(&self, voucher_id: Uuid, user_id: Uuid, comment: String) -> Result<VoucherComment> {
        let id = Uuid::now_v7();
        sqlx::query(
            "INSERT INTO voucher_comments (id, voucher_id, user_id, comment, created_at) VALUES ($1, $2, $3, $4, NOW())",
        )
        .bind(id)
        .bind(voucher_id)
        .bind(user_id)
        .bind(&comment)
        .execute(&self.pool)
        .await?;

        Ok(VoucherComment {
            id,
            voucher_id,
            user_id,
            comment,
            created_at: chrono::Utc::now().naive_utc(),
        })
    }

    pub async fn upload_attachment(
        &self,
        voucher_id: Uuid,
        file_name: String,
        file_path: String,
        file_size: i64,
        content_type: String,
        uploaded_by: Uuid,
    ) -> Result<VoucherAttachment> {
        let id = Uuid::now_v7();
        sqlx::query(
            r#"INSERT INTO voucher_attachments (id, voucher_id, file_name, file_path, file_size, content_type, uploaded_by, created_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())"#,
        )
        .bind(id)
        .bind(voucher_id)
        .bind(&file_name)
        .bind(&file_path)
        .bind(file_size)
        .bind(&content_type)
        .bind(uploaded_by)
        .execute(&self.pool)
        .await?;

        Ok(VoucherAttachment {
            id,
            voucher_id,
            file_name,
            file_path,
            file_size,
            content_type,
            uploaded_by,
            created_at: chrono::Utc::now().naive_utc(),
        })
    }

    async fn get_voucher(&self, id: Uuid) -> Result<PaymentVoucher> {
        sqlx::query_as::<_, PaymentVoucher>("SELECT * FROM payment_vouchers WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| anyhow!("Voucher not found"))
    }

    async fn update_status(&self, id: Uuid, status: VoucherStatus) -> Result<PaymentVoucher> {
        sqlx::query(
            "UPDATE payment_vouchers SET status = $1, updated_at = NOW() WHERE id = $2",
        )
        .bind(&status)
        .bind(id)
        .execute(&self.pool)
        .await?;
        self.get_voucher(id).await
    }

    async fn generate_voucher_number(&self, company_id: Uuid) -> Result<String> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM payment_vouchers WHERE company_id = $1",
        )
        .bind(company_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(format!("PV-{}", count + 1))
    }
}
