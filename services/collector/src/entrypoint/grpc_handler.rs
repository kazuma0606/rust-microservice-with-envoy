use std::sync::Arc;
use std::time::Instant;

use tonic::{Request, Response, Status};

use crate::domain::error::DomainError;
use crate::usecase::ingest_event::IngestEventUseCase;

use super::dto;
use super::proto::collector::collector_service_server::CollectorService;
use super::proto::collector::{IngestEventRequest, IngestEventResponse};

pub struct CollectorGrpcService {
    use_case: Arc<IngestEventUseCase>,
}

impl CollectorGrpcService {
    pub fn new(use_case: Arc<IngestEventUseCase>) -> Self {
        Self { use_case }
    }
}

#[tonic::async_trait]
impl CollectorService for CollectorGrpcService {
    async fn ingest_event(
        &self,
        request: Request<IngestEventRequest>,
    ) -> Result<Response<IngestEventResponse>, Status> {
        let start = Instant::now();
        let req = request.into_inner();

        tracing::info!(
            tenant_id = %req.tenant_id,
            user_id = %req.user_id,
            service = %req.service,
            decision = req.decision,
            "Received IngestEvent request"
        );

        let input = dto::to_ingest_event_input(req)?;

        let result = self.use_case.execute(input).await;
        let elapsed = start.elapsed().as_secs_f64();

        match result {
            Ok(output) => {
                metrics::counter!("authpulse_ingest_event_total", "status" => "ok").increment(1);
                metrics::histogram!("authpulse_ingest_event_duration_seconds", "status" => "ok")
                    .record(elapsed);
                let response = dto::to_ingest_event_response(output);
                Ok(Response::new(response))
            }
            Err(e) => {
                let status_label = match &e {
                    DomainError::Validation(_) => "invalid_argument",
                    DomainError::NotFound(_) => "not_found",
                    DomainError::Infrastructure(_) => "internal",
                };
                metrics::counter!("authpulse_ingest_event_total", "status" => status_label)
                    .increment(1);
                metrics::histogram!("authpulse_ingest_event_duration_seconds", "status" => status_label)
                    .record(elapsed);
                Err(domain_error_to_status(e))
            }
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
