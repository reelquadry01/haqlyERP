// Author: Quadri Atharu
use anyhow::{anyhow, Result};
use bigdecimal::BigDecimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::crm::{
    Activity, ActivityStatus, ActivityType, Contact, ContactStatus, Deal, DealStage,
};

#[derive(Clone)]
pub struct CrmService {
    pub pool: PgPool,
}

impl CrmService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_contact(
        &self,
        company_id: Uuid,
        first_name: String,
        last_name: String,
        email: Option<String>,
        phone: Option<String>,
        organization: Option<String>,
        title: Option<String>,
        lead_source: Option<String>,
        status: ContactStatus,
        assigned_to: Option<Uuid>,
        notes: Option<String>,
    ) -> Result<Contact> {
        let id = Uuid::now_v7();
        sqlx::query(
            r#"INSERT INTO contacts (id, company_id, first_name, last_name, email, phone, organization, title, lead_source, status, assigned_to, notes, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, NOW(), NOW())"#,
        )
        .bind(id)
        .bind(company_id)
        .bind(&first_name)
        .bind(&last_name)
        .bind(&email)
        .bind(&phone)
        .bind(&organization)
        .bind(&title)
        .bind(&lead_source)
        .bind(&status)
        .bind(assigned_to)
        .bind(&notes)
        .execute(&self.pool)
        .await?;

        sqlx::query_as::<_, Contact>("SELECT * FROM contacts WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch contact: {}", e))
    }

    pub async fn list_contacts(&self, company_id: Uuid, page: i64, limit: i64) -> Result<Vec<Contact>> {
        let offset = (page - 1) * limit;
        sqlx::query_as::<_, Contact>(
            "SELECT * FROM contacts WHERE company_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(company_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| anyhow!("Failed to list contacts: {}", e))
    }

    pub async fn create_deal(
        &self,
        company_id: Uuid,
        contact_id: Uuid,
        title: String,
        value: BigDecimal,
        stage: DealStage,
        probability: i32,
        expected_close_date: Option<chrono::NaiveDate>,
        assigned_to: Option<Uuid>,
    ) -> Result<Deal> {
        let id = Uuid::now_v7();
        sqlx::query(
            r#"INSERT INTO deals (id, company_id, contact_id, title, value, stage, probability, expected_close_date, assigned_to, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW(), NOW())"#,
        )
        .bind(id)
        .bind(company_id)
        .bind(contact_id)
        .bind(&title)
        .bind(&value)
        .bind(&stage)
        .bind(probability)
        .bind(expected_close_date)
        .bind(assigned_to)
        .execute(&self.pool)
        .await?;

        sqlx::query_as::<_, Deal>("SELECT * FROM deals WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch deal: {}", e))
    }

    pub async fn update_deal_stage(
        &self,
        deal_id: Uuid,
        stage: DealStage,
        probability: Option<i32>,
    ) -> Result<Deal> {
        let deal = sqlx::query_as::<_, Deal>("SELECT * FROM deals WHERE id = $1")
            .bind(deal_id)
            .fetch_one(&self.pool)
            .await?;

        let new_probability = probability.unwrap_or_else(|| match stage {
            DealStage::Prospecting => 10,
            DealStage::Qualification => 25,
            DealStage::Proposal => 50,
            DealStage::Negotiation => 75,
            DealStage::ClosedWon => 100,
            DealStage::ClosedLost => 0,
        });

        sqlx::query(
            "UPDATE deals SET stage = $1, probability = $2, updated_at = NOW() WHERE id = $3",
        )
        .bind(&stage)
        .bind(new_probability)
        .bind(deal_id)
        .execute(&self.pool)
        .await?;

        sqlx::query_as::<_, Deal>("SELECT * FROM deals WHERE id = $1")
            .bind(deal_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch deal: {}", e))
    }

    pub async fn list_deals_by_stage(&self, company_id: Uuid, stage: DealStage) -> Result<Vec<Deal>> {
        sqlx::query_as::<_, Deal>(
            "SELECT * FROM deals WHERE company_id = $1 AND stage = $2 ORDER BY created_at DESC",
        )
        .bind(company_id)
        .bind(&stage)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| anyhow!("Failed to list deals: {}", e))
    }

    pub async fn create_activity(
        &self,
        company_id: Uuid,
        contact_id: Option<Uuid>,
        deal_id: Option<Uuid>,
        activity_type: ActivityType,
        subject: String,
        description: Option<String>,
        due_date: Option<chrono::NaiveDate>,
        assigned_to: Option<Uuid>,
    ) -> Result<Activity> {
        let id = Uuid::now_v7();
        sqlx::query(
            r#"INSERT INTO activities (id, company_id, contact_id, deal_id, activity_type, subject, description, due_date, status, assigned_to, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'pending', $9, NOW(), NOW())"#,
        )
        .bind(id)
        .bind(company_id)
        .bind(contact_id)
        .bind(deal_id)
        .bind(&activity_type)
        .bind(&subject)
        .bind(&description)
        .bind(due_date)
        .bind(assigned_to)
        .execute(&self.pool)
        .await?;

        sqlx::query_as::<_, Activity>("SELECT * FROM activities WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch activity: {}", e))
    }

    pub async fn list_activities(
        &self,
        company_id: Uuid,
        contact_id: Option<Uuid>,
        deal_id: Option<Uuid>,
        page: i64,
        limit: i64,
    ) -> Result<Vec<Activity>> {
        let offset = (page - 1) * limit;
        match (contact_id, deal_id) {
            (Some(cid), None) => {
                sqlx::query_as::<_, Activity>(
                    "SELECT * FROM activities WHERE company_id = $1 AND contact_id = $2 ORDER BY created_at DESC LIMIT $3 OFFSET $4",
                )
                .bind(company_id)
                .bind(cid)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
            (None, Some(did)) => {
                sqlx::query_as::<_, Activity>(
                    "SELECT * FROM activities WHERE company_id = $1 AND deal_id = $2 ORDER BY created_at DESC LIMIT $3 OFFSET $4",
                )
                .bind(company_id)
                .bind(did)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
            (Some(cid), Some(did)) => {
                sqlx::query_as::<_, Activity>(
                    "SELECT * FROM activities WHERE company_id = $1 AND contact_id = $2 AND deal_id = $3 ORDER BY created_at DESC LIMIT $4 OFFSET $5",
                )
                .bind(company_id)
                .bind(cid)
                .bind(did)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
            (None, None) => {
                sqlx::query_as::<_, Activity>(
                    "SELECT * FROM activities WHERE company_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
                )
                .bind(company_id)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
        }
        .map_err(|e| anyhow!("Failed to list activities: {}", e))
    }

    pub async fn get_pipeline_summary(&self, company_id: Uuid) -> Result<Vec<(DealStage, i64, BigDecimal)>> {
        let rows: Vec<(String, i64, BigDecimal)> = sqlx::query_as(
            r#"SELECT stage::text, COUNT(*) as count, COALESCE(SUM(value), 0) as total_value
               FROM deals WHERE company_id = $1
               GROUP BY stage ORDER BY stage"#,
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await?;

        let mut summary = Vec::new();
        for (stage_str, count, total_value) in rows {
            let stage = match stage_str.as_str() {
                "prospecting" => DealStage::Prospecting,
                "qualification" => DealStage::Qualification,
                "proposal" => DealStage::Proposal,
                "negotiation" => DealStage::Negotiation,
                "closed_won" => DealStage::ClosedWon,
                "closed_lost" => DealStage::ClosedLost,
                _ => continue,
            };
            summary.push((stage, count, total_value));
        }

        Ok(summary)
    }
}
