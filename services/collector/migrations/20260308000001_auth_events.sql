CREATE TABLE IF NOT EXISTS auth_events (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id    VARCHAR(64)  NOT NULL,
    user_id      VARCHAR(128) NOT NULL,
    service      VARCHAR(128) NOT NULL,
    resource     VARCHAR(256) NOT NULL,
    action       VARCHAR(64)  NOT NULL,
    decision     VARCHAR(8)   NOT NULL CHECK (decision IN ('ALLOW', 'DENY')),
    reason_code  VARCHAR(64),
    latency_ms   BIGINT,
    source_ip    VARCHAR(45),
    trace_id     VARCHAR(64),
    timestamp    TIMESTAMPTZ  NOT NULL,
    recorded_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_auth_events_tenant_timestamp
    ON auth_events (tenant_id, timestamp DESC);

CREATE INDEX IF NOT EXISTS idx_auth_events_tenant_user_timestamp
    ON auth_events (tenant_id, user_id, timestamp DESC);
