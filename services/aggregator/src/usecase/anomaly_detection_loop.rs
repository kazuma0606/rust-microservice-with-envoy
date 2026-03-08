use std::sync::Arc;
use std::time::Instant;

use tokio::time::{interval, Duration};

use crate::usecase::detect_anomaly::DetectAnomalyUseCase;

pub async fn run_anomaly_detection_loop(use_case: Arc<DetectAnomalyUseCase>) {
    let interval_secs = std::env::var("DETECTION_INTERVAL_SECS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(300);

    tracing::info!(
        "Starting anomaly detection loop (interval: {}s)",
        interval_secs
    );

    let mut ticker = interval(Duration::from_secs(interval_secs));
    ticker.tick().await; // skip first immediate tick

    loop {
        ticker.tick().await;
        tracing::debug!("Running anomaly detection cycle");

        let start = Instant::now();
        match use_case.run_detection_cycle().await {
            Ok(()) => {
                let elapsed = start.elapsed().as_secs_f64();
                metrics::histogram!("authpulse_detection_cycle_duration_seconds").record(elapsed);
                tracing::debug!("Anomaly detection cycle completed in {:.3}s", elapsed);
            }
            Err(e) => {
                tracing::error!("Anomaly detection cycle failed: {}", e);
                metrics::counter!("authpulse_detection_cycle_errors_total").increment(1);
            }
        }
    }
}
