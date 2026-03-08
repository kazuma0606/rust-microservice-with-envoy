use tonic::Status;

use crate::usecase::ingest_event::{IngestEventInput, IngestEventOutput};

use super::proto::collector::{IngestEventRequest, IngestEventResponse};

#[allow(clippy::result_large_err)]
pub fn to_ingest_event_input(req: IngestEventRequest) -> Result<IngestEventInput, Status> {
    if req.timestamp_unix_ms == 0 {
        return Err(Status::invalid_argument("timestamp_unix_ms is required"));
    }

    Ok(IngestEventInput {
        tenant_id: req.tenant_id,
        user_id: req.user_id,
        service: req.service,
        resource: req.resource,
        action: req.action,
        decision: req.decision,
        reason_code: if req.reason_code.is_empty() {
            None
        } else {
            Some(req.reason_code)
        },
        latency_ms: if req.latency_ms == 0 {
            None
        } else {
            Some(req.latency_ms)
        },
        source_ip: if req.source_ip.is_empty() {
            None
        } else {
            Some(req.source_ip)
        },
        trace_id: if req.trace_id.is_empty() {
            None
        } else {
            Some(req.trace_id)
        },
        timestamp_unix_ms: req.timestamp_unix_ms,
    })
}

pub fn to_ingest_event_response(output: IngestEventOutput) -> IngestEventResponse {
    IngestEventResponse {
        event_id: output.event_id.to_string(),
        recorded_at_unix_ms: output.recorded_at.timestamp_millis(),
    }
}
