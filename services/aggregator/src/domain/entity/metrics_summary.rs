use crate::domain::value_object::{LatencyPercentiles, TenantId};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct MetricsSummary {
    pub tenant_id: TenantId,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub allow_count: u64,
    pub deny_count: u64,
    pub allow_rate: f64,
    pub latency: LatencyPercentiles,
    pub rate_limit_count: u64,
    pub computed_at: DateTime<Utc>,
}

impl MetricsSummary {
    pub fn compute(
        tenant_id: TenantId,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        allow_count: u64,
        deny_count: u64,
        latency: LatencyPercentiles,
        rate_limit_count: u64,
    ) -> Self {
        let total = allow_count + deny_count;
        let allow_rate = if total == 0 {
            0.0
        } else {
            allow_count as f64 / total as f64
        };

        Self {
            tenant_id,
            period_start,
            period_end,
            allow_count,
            deny_count,
            allow_rate,
            latency,
            rate_limit_count,
            computed_at: Utc::now(),
        }
    }
}
