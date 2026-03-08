use crate::domain::error::DomainError;
use crate::domain::value_object::TenantId;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

/// Raw event data fetched from DB for aggregation
#[allow(dead_code)]
#[derive(Debug)]
pub struct RawEventData {
    pub decision: String,
    pub latency_ms: Option<i64>,
}

/// Aggregated counts for a tenant/period
#[derive(Debug, Default)]
pub struct EventAggregate {
    pub allow_count: u64,
    pub deny_count: u64,
    pub latency_values: Vec<u64>,
}

#[async_trait]
pub trait EventReadRepository: Send + Sync {
    async fn aggregate_by_tenant_and_period(
        &self,
        tenant_id: &TenantId,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<EventAggregate, DomainError>;
}
