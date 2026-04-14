// Author: Quadri Atharu

use sha2::{Sha256, Digest};
use sqlx::PgPool;
use uuid::Uuid;

pub struct AuditChainEntry {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub entity: String,
    pub entity_id: Option<Uuid>,
    pub details: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub previous_hash: Option<String>,
    pub entry_hash: String,
}

pub fn compute_entry_hash(entry: &AuditChainEntry) -> String {
    let mut hasher = Sha256::new();
    hasher.update(entry.id.to_string().as_bytes());
    hasher.update(b"|");
    hasher.update(
        entry
            .user_id
            .map(|u| u.to_string())
            .unwrap_or_default()
            .as_bytes(),
    );
    hasher.update(b"|");
    hasher.update(entry.action.as_bytes());
    hasher.update(b"|");
    hasher.update(entry.entity.as_bytes());
    hasher.update(b"|");
    hasher.update(
        entry
            .entity_id
            .map(|id| id.to_string())
            .unwrap_or_default()
            .as_bytes(),
    );
    hasher.update(b"|");
    hasher.update(
        entry
            .details
            .as_ref()
            .map(|d| d.to_string())
            .unwrap_or_default()
            .as_bytes(),
    );
    hasher.update(b"|");
    hasher.update(
        entry
            .ip_address
            .as_deref()
            .unwrap_or_default()
            .as_bytes(),
    );
    hasher.update(b"|");
    hasher.update(entry.created_at.to_string().as_bytes());
    hasher.update(b"|");
    hasher.update(
        entry
            .previous_hash
            .as_deref()
            .unwrap_or_default()
            .as_bytes(),
    );
    format!("{:x}", hasher.finalize())
}

pub async fn append_to_chain(pool: &PgPool, entry: &AuditChainEntry) -> Result<Uuid, String> {
    let previous_hash: Option<String> = sqlx::query_scalar(
        "SELECT entry_hash FROM audit_logs ORDER BY created_at DESC, id DESC LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Failed to fetch previous hash: {}", e))?;

    let mut entry_with_prev = AuditChainEntry {
        id: entry.id,
        user_id: entry.user_id,
        action: entry.action.clone(),
        entity: entry.entity.clone(),
        entity_id: entry.entity_id,
        details: entry.details.clone(),
        ip_address: entry.ip_address.clone(),
        created_at: entry.created_at,
        previous_hash: previous_hash.clone(),
        entry_hash: String::new(),
    };

    let entry_hash = compute_entry_hash(&entry_with_prev);
    entry_with_prev.entry_hash = entry_hash.clone();

    sqlx::query(
        r#"INSERT INTO audit_logs (id, user_id, action, entity, entity_id, details, ip_address, previous_hash, entry_hash)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#,
    )
    .bind(entry_with_prev.id)
    .bind(entry_with_prev.user_id)
    .bind(&entry_with_prev.action)
    .bind(&entry_with_prev.entity)
    .bind(entry_with_prev.entity_id)
    .bind(&entry_with_prev.details)
    .bind(&entry_with_prev.ip_address)
    .bind(&entry_with_prev.previous_hash)
    .bind(&entry_with_prev.entry_hash)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to insert chain entry: {}", e))?;

    Ok(entry_with_prev.id)
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct AuditChainRow {
    id: Uuid,
    user_id: Option<Uuid>,
    action: String,
    entity: String,
    entity_id: Option<Uuid>,
    details: Option<serde_json::Value>,
    ip_address: Option<String>,
    created_at: chrono::NaiveDateTime,
    previous_hash: Option<String>,
    entry_hash: String,
}

pub struct ChainVerificationResult {
    pub is_intact: bool,
    pub total_entries: usize,
    pub first_broken_entry_id: Option<Uuid>,
    pub verified_at: chrono::Utc,
}

pub async fn verify_chain(pool: &PgPool) -> Result<ChainVerificationResult, String> {
    let rows = sqlx::query_as::<_, AuditChainRow>(
        "SELECT id, user_id, action, entity, entity_id, details, ip_address, created_at, previous_hash, entry_hash FROM audit_logs ORDER BY created_at ASC, id ASC",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to fetch audit entries: {}", e))?;

    let total_entries = rows.len();
    let mut is_intact = true;
    let mut first_broken_entry_id: Option<Uuid> = None;
    let mut last_valid_entry_id: Option<Uuid> = None;

    for (i, row) in rows.iter().enumerate() {
        let entry = AuditChainEntry {
            id: row.id,
            user_id: row.user_id,
            action: row.action.clone(),
            entity: row.entity.clone(),
            entity_id: row.entity_id,
            details: row.details.clone(),
            ip_address: row.ip_address.clone(),
            created_at: row.created_at,
            previous_hash: row.previous_hash.clone(),
            entry_hash: String::new(),
        };

        let computed_hash = compute_entry_hash(&entry);

        if computed_hash != row.entry_hash {
            if is_intact {
                is_intact = false;
                first_broken_entry_id = Some(row.id);
                sqlx::query("UPDATE audit_logs SET chain_valid = false WHERE id = $1")
                    .bind(row.id)
                    .execute(pool)
                    .await
                    .map_err(|e| format!("Failed to mark broken entry: {}", e))?;
            }
            continue;
        }

        if i > 0 {
            let expected_prev = &rows[i - 1].entry_hash;
            let actual_prev = row.previous_hash.as_deref().unwrap_or("");
            if expected_prev != actual_prev {
                if is_intact {
                    is_intact = false;
                    first_broken_entry_id = Some(row.id);
                    sqlx::query("UPDATE audit_logs SET chain_valid = false WHERE id = $1")
                        .bind(row.id)
                        .execute(pool)
                        .await
                        .map_err(|e| format!("Failed to mark broken entry: {}", e))?;
                }
                continue;
            }
        } else if row.previous_hash.is_some() {
            if is_intact {
                is_intact = false;
                first_broken_entry_id = Some(row.id);
                sqlx::query("UPDATE audit_logs SET chain_valid = false WHERE id = $1")
                    .bind(row.id)
                    .execute(pool)
                    .await
                    .map_err(|e| format!("Failed to mark broken entry: {}", e))?;
            }
            continue;
        }

        last_valid_entry_id = Some(row.id);
    }

    if is_intact && total_entries > 0 {
        last_valid_entry_id = Some(rows[total_entries - 1].id);
    }

    let verification_id = Uuid::now_v7();
    sqlx::query(
        r#"INSERT INTO audit_chain_verification (id, verified_at, last_valid_entry_id, broken_at_entry_id, is_intact, total_entries, verified_by)
           VALUES ($1, NOW(), $2, $3, $4, $5, 'system')"#,
    )
    .bind(verification_id)
    .bind(last_valid_entry_id)
    .bind(first_broken_entry_id)
    .bind(is_intact)
    .bind(total_entries as i32)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to store verification result: {}", e))?;

    Ok(ChainVerificationResult {
        is_intact,
        total_entries,
        first_broken_entry_id,
        verified_at: chrono::Utc::now(),
    })
}

pub async fn get_last_verification(
    pool: &PgPool,
) -> Result<Option<ChainVerificationResult>, String> {
    let row: Option<(bool, i32, Option<Uuid>, chrono::NaiveDateTime)> = sqlx::query_as(
        "SELECT is_intact, total_entries, broken_at_entry_id, verified_at FROM audit_chain_verification ORDER BY verified_at DESC LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Failed to fetch last verification: {}", e))?;

    Ok(row.map(|(is_intact, total_entries, first_broken_entry_id, verified_at)| {
        ChainVerificationResult {
            is_intact,
            total_entries: total_entries as usize,
            first_broken_entry_id,
            verified_at: chrono::DateTime::from_naive_utc_and_offset(verified_at, chrono::Utc),
        }
    }))
}
