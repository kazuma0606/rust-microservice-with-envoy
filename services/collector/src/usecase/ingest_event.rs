use std::sync::Arc;

use chrono::{DateTime, TimeZone, Utc};
use uuid::Uuid;

use crate::domain::entity::AuthEvent;
use crate::domain::error::DomainError;
use crate::domain::value_object::{Decision, TenantId};
use crate::port::event_repository::EventRepository;

pub struct IngestEventUseCase {
    repository: Arc<dyn EventRepository>,
}

impl IngestEventUseCase {
    pub fn new(repository: Arc<dyn EventRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, input: IngestEventInput) -> Result<IngestEventOutput, DomainError> {
        let tenant_id = TenantId::new(input.tenant_id)?;
        let decision = Decision::from_proto(input.decision)?;

        let timestamp = Utc
            .timestamp_millis_opt(input.timestamp_unix_ms)
            .single()
            .ok_or_else(|| DomainError::Validation("invalid timestamp_unix_ms".to_string()))?;

        let event = AuthEvent::new(
            tenant_id,
            input.user_id,
            input.service,
            input.resource,
            input.action,
            decision,
            input.reason_code,
            input.latency_ms,
            input.source_ip,
            input.trace_id,
            timestamp,
        )?;

        let event_id = event.id;
        let recorded_at = event.recorded_at;

        self.repository.save(&event).await?;

        Ok(IngestEventOutput {
            event_id,
            recorded_at,
        })
    }
}

pub struct IngestEventInput {
    pub tenant_id: String,
    pub user_id: String,
    pub service: String,
    pub resource: String,
    pub action: String,
    pub decision: i32,
    pub reason_code: Option<String>,
    pub latency_ms: Option<u64>,
    pub source_ip: Option<String>,
    pub trace_id: Option<String>,
    pub timestamp_unix_ms: i64,
}

pub struct IngestEventOutput {
    pub event_id: Uuid,
    pub recorded_at: DateTime<Utc>,
}
