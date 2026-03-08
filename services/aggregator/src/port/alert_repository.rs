use crate::domain::entity::Alert;
use crate::domain::error::DomainError;
use crate::domain::value_object::TenantId;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait AlertRepository: Send + Sync {
    async fn save(&self, alert: &Alert) -> Result<(), DomainError>;

    async fn find_by_tenant(
        &self,
        tenant_id: &TenantId,
        include_resolved: bool,
    ) -> Result<Vec<Alert>, DomainError>;

    async fn resolve(&self, tenant_id: &TenantId, alert_id: Uuid) -> Result<Alert, DomainError>;

    /// Count DENY events in the last `window_secs` seconds for anomaly detection
    async fn count_deny_events_in_window(
        &self,
        tenant_id: &TenantId,
        window_secs: i64,
    ) -> Result<u64, DomainError>;

    /// Count consecutive failures for a specific user in the last `window_secs` seconds
    async fn count_consecutive_failures_for_user(
        &self,
        tenant_id: &TenantId,
        user_id: &str,
        window_secs: i64,
    ) -> Result<u64, DomainError>;

    /// Get distinct users who failed in the last `window_secs` seconds
    async fn get_recently_failed_users(
        &self,
        tenant_id: &TenantId,
        window_secs: i64,
    ) -> Result<Vec<String>, DomainError>;

    /// Get all active tenant IDs that have events
    async fn get_active_tenant_ids(&self) -> Result<Vec<String>, DomainError>;
}
