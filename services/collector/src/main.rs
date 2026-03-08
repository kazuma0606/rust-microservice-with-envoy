mod adapter;
mod domain;
mod entrypoint;
mod port;
mod usecase;

use std::sync::Arc;

use sqlx::postgres::PgPoolOptions;
use tonic::transport::Server;

use adapter::postgres_event_repository::PostgresEventRepository;
use entrypoint::grpc_handler::CollectorGrpcService;
use entrypoint::proto::collector::collector_service_server::CollectorServiceServer;
use usecase::ingest_event::IngestEventUseCase;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    // Initialize Prometheus metrics exporter (HTTP scrape endpoint)
    let metrics_addr: std::net::SocketAddr = std::env::var("METRICS_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:9091".to_string())
        .parse()?;
    metrics_exporter_prometheus::PrometheusBuilder::new()
        .with_http_listener(metrics_addr)
        .install()
        .expect("Failed to install Prometheus metrics exporter");
    tracing::info!("Prometheus metrics available on {}", metrics_addr);

    // Initialize OpenTelemetry OTLP exporter (when OTEL_EXPORTER_OTLP_ENDPOINT is set)
    let _tracer_provider = init_otlp_if_configured();

    // Database connection
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set");

    tracing::info!("Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    // Run migrations
    tracing::info!("Running database migrations...");
    sqlx::migrate!().run(&pool).await?;
    tracing::info!("Migrations complete");

    // Wire up dependencies (Dependency Injection)
    let repository = Arc::new(PostgresEventRepository::new(pool));
    let use_case = Arc::new(IngestEventUseCase::new(repository));
    let service = CollectorGrpcService::new(use_case);

    // Start gRPC server with trace context propagation interceptor
    let addr = std::env::var("COLLECTOR_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:50051".to_string())
        .parse()?;

    tracing::info!("Collector gRPC server listening on {}", addr);

    Server::builder()
        .add_service(CollectorServiceServer::with_interceptor(
            service,
            trace_interceptor,
        ))
        .serve(addr)
        .await?;

    Ok(())
}

/// Initialize OpenTelemetry OTLP exporter when OTEL_EXPORTER_OTLP_ENDPOINT is set.
/// Tracing spans (via `tracing` crate) are logged separately; this provider enables
/// direct span creation via `opentelemetry::global::tracer()`.
fn init_otlp_if_configured() -> Option<opentelemetry_sdk::trace::TracerProvider> {
    use opentelemetry_otlp::WithExportConfig;
    use opentelemetry_sdk::runtime;

    let endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok()?;
    tracing::info!("Initializing OpenTelemetry OTLP exporter: {}", endpoint);

    let exporter = opentelemetry_otlp::new_exporter()
        .http()
        .with_endpoint(&endpoint)
        .build_span_exporter()
        .map_err(|e| tracing::warn!("OTLP exporter build failed: {}", e))
        .ok()?;

    let provider = opentelemetry_sdk::trace::TracerProvider::builder()
        .with_batch_exporter(exporter, runtime::Tokio)
        .with_config(opentelemetry_sdk::trace::config().with_resource(
            opentelemetry_sdk::Resource::new(vec![opentelemetry::KeyValue::new(
                "service.name",
                env!("CARGO_PKG_NAME"),
            )]),
        ))
        .build();

    opentelemetry::global::set_tracer_provider(provider.clone());
    tracing::info!("OpenTelemetry OTLP exporter initialized");
    Some(provider)
}

/// Tonic interceptor: extracts W3C `traceparent` / `tracestate` headers and stores
/// the OpenTelemetry context in request extensions for downstream span creation.
#[allow(clippy::result_large_err)]
fn trace_interceptor(mut request: tonic::Request<()>) -> Result<tonic::Request<()>, tonic::Status> {
    use opentelemetry::propagation::TextMapPropagator;

    let propagator = opentelemetry_sdk::propagation::TraceContextPropagator::new();
    let extractor = MetadataExtractor(request.metadata());
    let context = propagator.extract(&extractor);
    request.extensions_mut().insert(context);
    Ok(request)
}

/// Adapts tonic `MetadataMap` to the OpenTelemetry `Extractor` trait,
/// avoiding the `http` crate version conflict introduced by `opentelemetry-http`.
struct MetadataExtractor<'a>(&'a tonic::metadata::MetadataMap);

impl<'a> opentelemetry::propagation::Extractor for MetadataExtractor<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|v| v.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.0
            .keys()
            .filter_map(|k| match k {
                tonic::metadata::KeyRef::Ascii(key) => Some(key.as_str()),
                tonic::metadata::KeyRef::Binary(_) => None,
            })
            .collect()
    }
}
