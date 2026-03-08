#[derive(Debug, Clone)]
pub struct LatencyPercentiles {
    pub p50_ms: u64,
    pub p95_ms: u64,
    pub p99_ms: u64,
    pub no_data: bool,
}

impl LatencyPercentiles {
    pub fn no_data() -> Self {
        Self {
            p50_ms: 0,
            p95_ms: 0,
            p99_ms: 0,
            no_data: true,
        }
    }

    pub fn new(p50_ms: u64, p95_ms: u64, p99_ms: u64) -> Self {
        Self {
            p50_ms,
            p95_ms,
            p99_ms,
            no_data: false,
        }
    }
}
