use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entity::WebhookConfig;
use crate::domain::error::DomainError;
use crate::domain::value_object::TenantId;
use crate::port::webhook_config_repository::WebhookConfigRepository;

pub struct PostgresWebhookConfigRepository {
    pool: PgPool,
}

impl PostgresWebhookConfigRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// (id, tenant_id, url, is_active, last_notified_at, created_at)
type WebhookRow = (
    Uuid,
    String,
    String,
    bool,
    Option<DateTime<Utc>>,
    DateTime<Utc>,
);

#[async_trait]
impl WebhookConfigRepository for PostgresWebhookConfigRepository {
    async fn upsert(&self, config: &WebhookConfig) -> Result<(), DomainError> {
        sqlx::query(
            r#"
            INSERT INTO webhook_configs (id, tenant_id, url, is_active, created_at)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (tenant_id)
            DO UPDATE SET url = EXCLUDED.url, is_active = EXCLUDED.is_active
            "#,
        )
        .bind(config.id)
        .bind(config.tenant_id.value())
        .bind(&config.url)
        .bind(config.is_active)
        .bind(config.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Infrastructure(e.to_string()))?;
        Ok(())
    }

    async fn find_by_tenant(
        &self,
        tenant_id: &TenantId,
    ) -> Result<Option<WebhookConfig>, DomainError> {
        let row: Option<WebhookRow> = sqlx::query_as(
            r#"SELECT id, tenant_id, url, is_active, last_notified_at, created_at
               FROM webhook_configs
               WHERE tenant_id = $1"#,
        )
        .bind(tenant_id.value())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        match row {
            None => Ok(None),
            Some((id, tenant_id_str, url, is_active, last_notified_at, created_at)) => {
                let tid = TenantId::new(&tenant_id_str)?;
                Ok(Some(WebhookConfig {
                    id,
                    tenant_id: tid,
                    url,
                    is_active,
                    last_notified_at,
                    created_at,
                }))
            }
        }
    }
}
