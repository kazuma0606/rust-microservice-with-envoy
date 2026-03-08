use std::sync::Arc;
use std::time::Instant;

use tonic::{Request, Response, Status};

use crate::domain::error::DomainError;
use crate::usecase::get_metrics::GetMetricsUseCase;
use crate::usecase::list_alerts::ListAlertsUseCase;
use crate::usecase::resolve_alert::ResolveAlertUseCase;
use crate::usecase::upsert_webhook_config::UpsertWebhookConfigUseCase;

use super::dto;
use super::proto::aggregator::aggregator_service_server::AggregatorService;
use super::proto::aggregator::{
    GetMetricsRequest, GetMetricsResponse, ListAlertsRequest, ListAlertsResponse,
    ResolveAlertRequest, ResolveAlertResponse, UpsertWebhookConfigRequest,
    UpsertWebhookConfigResponse,
};

pub struct AggregatorGrpcService {
    get_metrics: Arc<GetMetricsUseCase>,
    list_alerts: Arc<ListAlertsUseCase>,
    resolve_alert: Arc<ResolveAlertUseCase>,
    upsert_webhook: Arc<UpsertWebhookConfigUseCase>,
}

impl AggregatorGrpcService {
    pub fn new(
        get_metrics: Arc<GetMetricsUseCase>,
        list_alerts: Arc<ListAlertsUseCase>,
        resolve_alert: Arc<ResolveAlertUseCase>,
        upsert_webhook: Arc<UpsertWebhookConfigUseCase>,
    ) -> Self {
        Self {
            get_metrics,
            list_alerts,
            resolve_alert,
            upsert_webhook,
        }
    }
}

#[tonic::async_trait]
impl AggregatorService for AggregatorGrpcService {
    async fn get_metrics(
        &self,
        request: Request<GetMetricsRequest>,
    ) -> Result<Response<GetMetricsResponse>, Status> {
        let start = Instant::now();
        let req = request.into_inner();
        tracing::info!(tenant_id = %req.tenant_id, "GetMetrics request");
        let input = dto::to_get_metrics_input(req)?;
        let result = self.get_metrics.execute(input).await;
        let elapsed = start.elapsed().as_secs_f64();

        match result {
            Ok(summary) => {
                metrics::counter!("authpulse_get_metrics_total", "status" => "ok").increment(1);
                metrics::histogram!("authpulse_get_metrics_duration_seconds", "status" => "ok")
                    .record(elapsed);
                Ok(Response::new(dto::to_get_metrics_response(summary)))
            }
            Err(e) => {
                metrics::counter!("authpulse_get_metrics_total", "status" => "error").increment(1);
                metrics::histogram!("authpulse_get_metrics_duration_seconds", "status" => "error")
                    .record(elapsed);
                Err(domain_error_to_status(e))
            }
        }
    }

    async fn list_alerts(
        &self,
        request: Request<ListAlertsRequest>,
    ) -> Result<Response<ListAlertsResponse>, Status> {
        let start = Instant::now();
        let req = request.into_inner();
        tracing::info!(tenant_id = %req.tenant_id, "ListAlerts request");
        let input = dto::to_list_alerts_input(req);
        let result = self.list_alerts.execute(input).await;
        let elapsed = start.elapsed().as_secs_f64();

        match result {
            Ok(alerts) => {
                metrics::counter!("authpulse_list_alerts_total", "status" => "ok").increment(1);
                metrics::histogram!("authpulse_list_alerts_duration_seconds", "status" => "ok")
                    .record(elapsed);
                let total = alerts.len() as i32;
                let protos = alerts.iter().map(dto::alert_to_proto).collect();
                Ok(Response::new(ListAlertsResponse {
                    alerts: protos,
                    total_count: total,
                }))
            }
            Err(e) => {
                metrics::counter!("authpulse_list_alerts_total", "status" => "error").increment(1);
                metrics::histogram!("authpulse_list_alerts_duration_seconds", "status" => "error")
                    .record(elapsed);
                Err(domain_error_to_status(e))
            }
        }
    }

    async fn resolve_alert(
        &self,
        request: Request<ResolveAlertRequest>,
    ) -> Result<Response<ResolveAlertResponse>, Status> {
        let start = Instant::now();
        let req = request.into_inner();
        tracing::info!(tenant_id = %req.tenant_id, alert_id = %req.alert_id, "ResolveAlert request");
        let input = dto::to_resolve_alert_input(req);
        let result = self.resolve_alert.execute(input).await;
        let elapsed = start.elapsed().as_secs_f64();

        match result {
            Ok(alert) => {
                metrics::counter!("authpulse_resolve_alert_total", "status" => "ok").increment(1);
                metrics::histogram!("authpulse_resolve_alert_duration_seconds", "status" => "ok")
                    .record(elapsed);
                Ok(Response::new(ResolveAlertResponse {
                    alert: Some(dto::alert_to_proto(&alert)),
                }))
            }
            Err(e) => {
                metrics::counter!("authpulse_resolve_alert_total", "status" => "error")
                    .increment(1);
                metrics::histogram!(
                    "authpulse_resolve_alert_duration_seconds",
                    "status" => "error"
                )
                .record(elapsed);
                Err(domain_error_to_status(e))
            }
        }
    }

    async fn upsert_webhook_config(
        &self,
        request: Request<UpsertWebhookConfigRequest>,
    ) -> Result<Response<UpsertWebhookConfigResponse>, Status> {
        let req = request.into_inner();
        tracing::info!(tenant_id = %req.tenant_id, "UpsertWebhookConfig request");
        let input = dto::to_upsert_webhook_config_input(req);
        match self.upsert_webhook.execute(input).await {
            Ok(config) => Ok(Response::new(dto::webhook_config_to_response(config))),
            Err(e) => Err(domain_error_to_status(e)),
        }
    }
}

fn domain_error_to_status(e: DomainError) -> Status {
    match e {
        DomainError::Validation(msg) => Status::invalid_argument(msg),
        DomainError::NotFound(msg) => Status::not_found(msg),
        DomainError::Infrastructure(msg) => {
            tracing::error!("Infrastructure error: {}", msg);
            Status::internal("Internal server error")
        }
    }
}
