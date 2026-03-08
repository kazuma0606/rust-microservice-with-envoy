use crate::domain::entity::{Alert, WebhookConfig};
use crate::domain::error::DomainError;
use async_trait::async_trait;

#[async_trait]
pub trait Notifier: Send + Sync {
    async fn notify(&self, config: &WebhookConfig, alert: &Alert) -> Result<(), DomainError>;
}
