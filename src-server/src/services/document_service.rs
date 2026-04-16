// Author: Quadri Atharu
use anyhow::{anyhow, Result};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::document::DocumentAttachment;

#[derive(Clone)]
pub struct DocumentService {
    pub pool: PgPool,
}

impl DocumentService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upload_document(
        &self,
        company_id: Uuid,
        entity_type: String,
        entity_id: Uuid,
        file_name: String,
        file_data: Vec<u8>,
        mime_type: String,
        description: Option<String>,
        uploaded_by: Option<Uuid>,
    ) -> Result<DocumentAttachment> {
        let upload_dir = format!("./uploads/{}/{}", company_id, entity_type);
        tokio::fs::create_dir_all(&upload_dir).await
            .map_err(|e| anyhow!("Failed to create upload directory: {}", e))?;

        let file_id = Uuid::now_v7();
        let extension = file_name.rsplit('.').next().unwrap_or("bin");
        let stored_name = format!("{}_{}.{}", entity_id, file_id, extension);
        let file_path = format!("{}/{}", upload_dir, stored_name);

        tokio::fs::write(&file_path, &file_data).await
            .map_err(|e| anyhow!("Failed to write file: {}", e))?;

        let file_size = file_data.len() as i64;

        sqlx::query(
            r#"INSERT INTO document_attachments
               (id, company_id, entity_type, entity_id, file_name, file_path, file_size, mime_type, description, uploaded_by, is_deleted, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, false, NOW(), NOW())"#,
        )
        .bind(file_id)
        .bind(company_id)
        .bind(&entity_type)
        .bind(entity_id)
        .bind(&file_name)
        .bind(&file_path)
        .bind(file_size)
        .bind(&mime_type)
        .bind(&description)
        .bind(uploaded_by)
        .execute(&self.pool)
        .await?;

        self.get_document(file_id).await
    }

    pub async fn list_documents(
        &self,
        company_id: Uuid,
        entity_type: String,
        entity_id: Uuid,
    ) -> Result<Vec<DocumentAttachment>> {
        sqlx::query_as::<_, DocumentAttachment>(
            "SELECT * FROM document_attachments WHERE company_id = $1 AND entity_type = $2 AND entity_id = $3 AND is_deleted = false ORDER BY created_at DESC",
        )
        .bind(company_id)
        .bind(&entity_type)
        .bind(entity_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| anyhow!("Failed to list documents: {}", e))
    }

    pub async fn get_document(&self, id: Uuid) -> Result<DocumentAttachment> {
        sqlx::query_as::<_, DocumentAttachment>(
            "SELECT * FROM document_attachments WHERE id = $1 AND is_deleted = false",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow!("Document not found"))
    }

    pub async fn delete_document(&self, id: Uuid) -> Result<DocumentAttachment> {
        let doc = self.get_document(id).await?;

        sqlx::query(
            "UPDATE document_attachments SET is_deleted = true, updated_at = NOW() WHERE id = $1",
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        self.get_document_with_deleted(id).await
    }

    async fn get_document_with_deleted(&self, id: Uuid) -> Result<DocumentAttachment> {
        sqlx::query_as::<_, DocumentAttachment>(
            "SELECT * FROM document_attachments WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow!("Document not found"))
    }

    pub async fn get_file_path(&self, id: Uuid) -> Result<String> {
        let doc = self.get_document(id).await?;
        Ok(doc.file_path)
    }
}
