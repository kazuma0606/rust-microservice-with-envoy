use crate::domain::entity::Alert;
use crate::domain::error::DomainError;
use crate::domain::value_object::TenantId;
use crate::port::alert_repository::AlertRepository;
use std::sync::Arc;

pub struct ListAlertsUseCase {
    repository: Arc<dyn AlertRepository>,
}

impl ListAlertsUseCase {
    pub fn new(repository: Arc<dyn AlertRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, input: ListAlertsInput) -> Result<Vec<Alert>, DomainError> {
        let tenant_id = TenantId::new(input.tenant_id)?;
        let mut alerts = self
            .repository
            .find_by_tenant(&tenant_id, input.include_resolved)
            .await?;

        let limit = if input.page_size > 0 {
            input.page_size as usize
        } else {
            100
        };
        alerts.truncate(limit);
        Ok(alerts)
    }
}

pub struct ListAlertsInput {
    pub tenant_id: String,
    pub include_resolved: bool,
    pub page_size: i32,
}
