use crate::domain::entity::WebhookConfig;
use crate::domain::error::DomainError;
use crate::domain::value_object::TenantId;
use crate::port::webhook_config_repository::WebhookConfigRepository;
use std::sync::Arc;

pub struct UpsertWebhookConfigUseCase {
    repository: Arc<dyn WebhookConfigRepository>,
}

impl UpsertWebhookConfigUseCase {
    pub fn new(repository: Arc<dyn WebhookConfigRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        input: UpsertWebhookConfigInput,
    ) -> Result<WebhookConfig, DomainError> {
        let tenant_id = TenantId::new(input.tenant_id)?;
        let config = WebhookConfig::new(tenant_id, input.url, input.is_active)?;
        self.repository.upsert(&config).await?;
        Ok(config)
    }
}

pub struct UpsertWebhookConfigInput {
    pub tenant_id: String,
    pub url: String,
    pub is_active: bool,
}
