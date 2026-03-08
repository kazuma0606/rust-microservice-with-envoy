# ─── Stage 1: build ───────────────────────────────────────────────────────────
FROM rust:slim AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    protobuf-compiler pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY services/Cargo.toml ./
COPY services/collector/Cargo.toml collector/
COPY services/aggregator/Cargo.toml aggregator/
COPY proto/ proto/

# Stub main.rs only — no build.rs here, so cargo skips proto codegen
# and just fetches/compiles dependencies for caching
RUN mkdir -p collector/src aggregator/src && \
    echo "fn main() {}" > collector/src/main.rs && \
    echo "fn main() {}" > aggregator/src/main.rs

ENV DOCKER_BUILD=1
RUN cargo build --release --package aggregator 2>/dev/null; true

# Copy actual source + build.rs (triggers proto codegen on next build)
COPY services/aggregator/src/ aggregator/src/
COPY services/aggregator/build.rs aggregator/

RUN touch aggregator/src/main.rs && \
    cargo build --release --package aggregator && \
    strip target/release/aggregator

# ─── Stage 2: runtime ─────────────────────────────────────────────────────────
FROM ubuntu:24.04

RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl3 ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/aggregator /app/aggregator

EXPOSE 50052 9092
CMD ["/app/aggregator"]
