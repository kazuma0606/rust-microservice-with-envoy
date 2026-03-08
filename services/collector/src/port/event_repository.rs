use async_trait::async_trait;

use crate::domain::entity::AuthEvent;
use crate::domain::error::DomainError;

#[async_trait]
pub trait EventRepository: Send + Sync {
    async fn save(&self, event: &AuthEvent) -> Result<(), DomainError>;
}
