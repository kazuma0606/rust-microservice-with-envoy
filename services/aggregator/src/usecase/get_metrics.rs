use crate::domain::entity::MetricsSummary;
use crate::domain::error::DomainError;
use crate::domain::value_object::{LatencyPercentiles, TenantId};
use crate::port::event_read_repository::EventReadRepository;
use chrono::{TimeZone, Utc};
use std::sync::Arc;

pub struct GetMetricsUseCase {
    repository: Arc<dyn EventReadRepository>,
}

impl GetMetricsUseCase {
    pub fn new(repository: Arc<dyn EventReadRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, input: GetMetricsInput) -> Result<MetricsSummary, DomainError> {
        let tenant_id = TenantId::new(input.tenant_id)?;

        let from = Utc
            .timestamp_millis_opt(input.period_start_unix_ms)
            .single()
            .ok_or_else(|| DomainError::Validation("invalid period_start_unix_ms".to_string()))?;

        let to = Utc
            .timestamp_millis_opt(input.period_end_unix_ms)
            .single()
            .ok_or_else(|| DomainError::Validation("invalid period_end_unix_ms".to_string()))?;

        if from >= to {
            return Err(DomainError::Validation(
                "period_start must be before period_end".to_string(),
            ));
        }

        let agg = self
            .repository
            .aggregate_by_tenant_and_period(&tenant_id, from, to)
            .await?;

        let latency = if agg.latency_values.is_empty() {
            LatencyPercentiles::no_data()
        } else {
            let mut sorted = agg.latency_values.clone();
            sorted.sort_unstable();
            let p50 = percentile(&sorted, 50);
            let p95 = percentile(&sorted, 95);
            let p99 = percentile(&sorted, 99);
            LatencyPercentiles::new(p50, p95, p99)
        };

        Ok(MetricsSummary::compute(
            tenant_id,
            from,
            to,
            agg.allow_count,
            agg.deny_count,
            latency,
            0, // rate_limit_count: collected via Envoy metrics in Phase N
        ))
    }
}

fn percentile(sorted: &[u64], p: usize) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let idx = ((p as f64 / 100.0) * (sorted.len() - 1) as f64).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

pub struct GetMetricsInput {
    pub tenant_id: String,
    pub period_start_unix_ms: i64,
    pub period_end_unix_ms: i64,
}
