use crate::domain::error::DomainError;
use crate::domain::value_object::TenantId;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct WebhookConfig {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub url: String,
    pub is_active: bool,
    #[allow(dead_code)]
    pub last_notified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl WebhookConfig {
    pub fn new(tenant_id: TenantId, url: String, is_active: bool) -> Result<Self, DomainError> {
        if !url.starts_with("https://") {
            return Err(DomainError::Validation(
                "Webhook URL must use https://".to_string(),
            ));
        }
        if url.len() > 2048 {
            return Err(DomainError::Validation(
                "Webhook URL too long (max 2048 chars)".to_string(),
            ));
        }
        Ok(Self {
            id: Uuid::new_v4(),
            tenant_id,
            url,
            is_active,
            last_notified_at: None,
            created_at: Utc::now(),
        })
    }
}
