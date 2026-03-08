use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::error::DomainError;
use crate::domain::value_object::{Decision, TenantId};

#[derive(Debug, Clone)]
pub struct AuthEvent {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub user_id: String,
    pub service: String,
    pub resource: String,
    pub action: String,
    pub decision: Decision,
    pub reason_code: Option<String>,
    pub latency_ms: Option<u64>,
    pub source_ip: Option<String>,
    pub trace_id: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub recorded_at: DateTime<Utc>,
}

impl AuthEvent {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tenant_id: TenantId,
        user_id: String,
        service: String,
        resource: String,
        action: String,
        decision: Decision,
        reason_code: Option<String>,
        latency_ms: Option<u64>,
        source_ip: Option<String>,
        trace_id: Option<String>,
        timestamp: DateTime<Utc>,
    ) -> Result<Self, DomainError> {
        if user_id.is_empty() {
            return Err(DomainError::Validation(
                "user_id cannot be empty".to_string(),
            ));
        }
        if user_id.len() > 128 {
            return Err(DomainError::Validation(
                "user_id too long (max 128 chars)".to_string(),
            ));
        }
        if service.is_empty() {
            return Err(DomainError::Validation(
                "service cannot be empty".to_string(),
            ));
        }
        if service.len() > 128 {
            return Err(DomainError::Validation(
                "service too long (max 128 chars)".to_string(),
            ));
        }
        if resource.is_empty() {
            return Err(DomainError::Validation(
                "resource cannot be empty".to_string(),
            ));
        }
        if resource.len() > 256 {
            return Err(DomainError::Validation(
                "resource too long (max 256 chars)".to_string(),
            ));
        }
        if action.is_empty() {
            return Err(DomainError::Validation(
                "action cannot be empty".to_string(),
            ));
        }
        if action.len() > 64 {
            return Err(DomainError::Validation(
                "action too long (max 64 chars)".to_string(),
            ));
        }

        Ok(AuthEvent {
            id: Uuid::new_v4(),
            tenant_id,
            user_id,
            service,
            resource,
            action,
            decision,
            reason_code,
            latency_ms,
            source_ip,
            trace_id,
            timestamp,
            recorded_at: Utc::now(),
        })
    }
}
