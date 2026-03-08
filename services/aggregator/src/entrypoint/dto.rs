use tonic::Status;

use crate::domain::entity::{Alert, MetricsSummary, WebhookConfig};
use crate::usecase::get_metrics::GetMetricsInput;
use crate::usecase::list_alerts::ListAlertsInput;
use crate::usecase::resolve_alert::ResolveAlertInput;
use crate::usecase::upsert_webhook_config::UpsertWebhookConfigInput;

use super::proto::aggregator::{
    AlertProto, GetMetricsRequest, GetMetricsResponse, ListAlertsRequest, ResolveAlertRequest,
    UpsertWebhookConfigRequest, UpsertWebhookConfigResponse,
};

#[allow(clippy::result_large_err)]
pub fn to_get_metrics_input(req: GetMetricsRequest) -> Result<GetMetricsInput, Status> {
    if req.period_start_unix_ms == 0 || req.period_end_unix_ms == 0 {
        return Err(Status::invalid_argument(
            "period_start_unix_ms and period_end_unix_ms are required",
        ));
    }
    Ok(GetMetricsInput {
        tenant_id: req.tenant_id,
        period_start_unix_ms: req.period_start_unix_ms,
        period_end_unix_ms: req.period_end_unix_ms,
    })
}

pub fn to_get_metrics_response(summary: MetricsSummary) -> GetMetricsResponse {
    GetMetricsResponse {
        tenant_id: summary.tenant_id.to_string(),
        period_start_unix_ms: summary.period_start.timestamp_millis(),
        period_end_unix_ms: summary.period_end.timestamp_millis(),
        allow_count: summary.allow_count,
        deny_count: summary.deny_count,
        allow_rate: summary.allow_rate,
        latency_p50_ms: summary.latency.p50_ms,
        latency_p95_ms: summary.latency.p95_ms,
        latency_p99_ms: summary.latency.p99_ms,
        latency_no_data: summary.latency.no_data,
        rate_limit_count: summary.rate_limit_count,
        computed_at_unix_ms: summary.computed_at.timestamp_millis(),
    }
}

pub fn to_list_alerts_input(req: ListAlertsRequest) -> ListAlertsInput {
    ListAlertsInput {
        tenant_id: req.tenant_id,
        include_resolved: req.include_resolved,
        page_size: req.page_size,
    }
}

pub fn alert_to_proto(alert: &Alert) -> AlertProto {
    AlertProto {
        id: alert.id.to_string(),
        tenant_id: alert.tenant_id.to_string(),
        rule_name: alert.rule_name.as_str().to_string(),
        severity: alert.severity.to_proto_i32(),
        detected_at_unix_ms: alert.detected_at.timestamp_millis(),
        related_user_id: alert.related_user_id.clone().unwrap_or_default(),
        related_service: alert.related_service.clone().unwrap_or_default(),
        detail: alert.detail.clone(),
        is_resolved: alert.is_resolved,
        resolved_at_unix_ms: alert.resolved_at.map(|t| t.timestamp_millis()).unwrap_or(0),
    }
}

pub fn to_resolve_alert_input(req: ResolveAlertRequest) -> ResolveAlertInput {
    ResolveAlertInput {
        tenant_id: req.tenant_id,
        alert_id: req.alert_id,
    }
}

pub fn to_upsert_webhook_config_input(req: UpsertWebhookConfigRequest) -> UpsertWebhookConfigInput {
    UpsertWebhookConfigInput {
        tenant_id: req.tenant_id,
        url: req.url,
        is_active: req.is_active,
    }
}

pub fn webhook_config_to_response(config: WebhookConfig) -> UpsertWebhookConfigResponse {
    UpsertWebhookConfigResponse {
        id: config.id.to_string(),
        tenant_id: config.tenant_id.to_string(),
        url: config.url,
        is_active: config.is_active,
    }
}
