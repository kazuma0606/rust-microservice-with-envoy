CREATE TABLE IF NOT EXISTS alerts (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id        VARCHAR(64)  NOT NULL,
    rule_name        VARCHAR(64)  NOT NULL,
    severity         VARCHAR(16)  NOT NULL CHECK (severity IN ('HIGH', 'MEDIUM', 'LOW')),
    detected_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    related_user_id  VARCHAR(128),
    related_service  VARCHAR(128),
    detail           TEXT         NOT NULL,
    is_resolved      BOOLEAN      NOT NULL DEFAULT FALSE,
    resolved_at      TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_alerts_tenant_resolved
    ON alerts (tenant_id, is_resolved, detected_at DESC);
