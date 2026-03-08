use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::entity::AuthEvent;
use crate::domain::error::DomainError;
use crate::port::event_repository::EventRepository;

pub struct PostgresEventRepository {
    pool: PgPool,
}

impl PostgresEventRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EventRepository for PostgresEventRepository {
    async fn save(&self, event: &AuthEvent) -> Result<(), DomainError> {
        let decision_str = event.decision.to_db_string();

        sqlx::query(
            r#"
            INSERT INTO auth_events
                (id, tenant_id, user_id, service, resource, action, decision,
                 reason_code, latency_ms, source_ip, trace_id, timestamp, recorded_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#,
        )
        .bind(event.id)
        .bind(event.tenant_id.value())
        .bind(&event.user_id)
        .bind(&event.service)
        .bind(&event.resource)
        .bind(&event.action)
        .bind(decision_str)
        .bind(&event.reason_code)
        .bind(event.latency_ms.map(|v| v as i64))
        .bind(&event.source_ip)
        .bind(&event.trace_id)
        .bind(event.timestamp)
        .bind(event.recorded_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        Ok(())
    }
}
