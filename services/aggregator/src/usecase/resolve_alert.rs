use crate::domain::entity::Alert;
use crate::domain::error::DomainError;
use crate::domain::value_object::TenantId;
use crate::port::alert_repository::AlertRepository;
use std::sync::Arc;
use uuid::Uuid;

pub struct ResolveAlertUseCase {
    repository: Arc<dyn AlertRepository>,
}

impl ResolveAlertUseCase {
    pub fn new(repository: Arc<dyn AlertRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, input: ResolveAlertInput) -> Result<Alert, DomainError> {
        let tenant_id = TenantId::new(input.tenant_id)?;
        let alert_id = Uuid::parse_str(&input.alert_id)
            .map_err(|_| DomainError::Validation("invalid alert_id format".to_string()))?;

        self.repository.resolve(&tenant_id, alert_id).await
    }
}

pub struct ResolveAlertInput {
    pub tenant_id: String,
    pub alert_id: String,
}
