use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::domain::error::DomainError;
use crate::domain::value_object::TenantId;
use crate::port::event_read_repository::{EventAggregate, EventReadRepository};

pub struct PostgresEventReadRepository {
    pool: PgPool,
}

impl PostgresEventReadRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EventReadRepository for PostgresEventReadRepository {
    async fn aggregate_by_tenant_and_period(
        &self,
        tenant_id: &TenantId,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<EventAggregate, DomainError> {
        let rows: Vec<(String, Option<i64>)> = sqlx::query_as(
            r#"
            SELECT decision, latency_ms
            FROM auth_events
            WHERE tenant_id = $1
              AND timestamp >= $2
              AND timestamp < $3
            "#,
        )
        .bind(tenant_id.value())
        .bind(from)
        .bind(to)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        let mut agg = EventAggregate::default();
        for (decision, latency_ms) in rows {
            match decision.as_str() {
                "ALLOW" => agg.allow_count += 1,
                "DENY" => agg.deny_count += 1,
                _ => {}
            }
            if let Some(ms) = latency_ms {
                if ms >= 0 {
                    agg.latency_values.push(ms as u64);
                }
            }
        }

        Ok(agg)
    }
}
