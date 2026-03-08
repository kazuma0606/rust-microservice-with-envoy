use crate::domain::entity::WebhookConfig;
use crate::domain::error::DomainError;
use crate::domain::value_object::TenantId;
use async_trait::async_trait;

#[async_trait]
pub trait WebhookConfigRepository: Send + Sync {
    async fn upsert(&self, config: &WebhookConfig) -> Result<(), DomainError>;
    async fn find_by_tenant(
        &self,
        tenant_id: &TenantId,
    ) -> Result<Option<WebhookConfig>, DomainError>;
}
