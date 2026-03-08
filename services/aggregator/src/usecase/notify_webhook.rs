use crate::domain::entity::{Alert, WebhookConfig};
use crate::domain::error::DomainError;
use crate::port::notifier::Notifier;
use std::sync::Arc;

#[allow(dead_code)]
pub struct NotifyWebhookUseCase {
    notifier: Arc<dyn Notifier>,
}

impl NotifyWebhookUseCase {
    #[allow(dead_code)]
    pub fn new(notifier: Arc<dyn Notifier>) -> Self {
        Self { notifier }
    }

    #[allow(dead_code)]
    pub async fn execute(&self, config: &WebhookConfig, alert: &Alert) -> Result<(), DomainError> {
        if !config.is_active {
            tracing::debug!(
                tenant_id = %config.tenant_id,
                "Webhook config is inactive, skipping notification"
            );
            return Ok(());
        }
        self.notifier.notify(config, alert).await
    }
}
