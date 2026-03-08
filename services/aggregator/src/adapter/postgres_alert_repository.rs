use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entity::Alert;
use crate::domain::error::DomainError;
use crate::domain::value_object::{AlertRuleName, AlertSeverity, TenantId};
use crate::port::alert_repository::AlertRepository;

pub struct PostgresAlertRepository {
    pool: PgPool,
}

impl PostgresAlertRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// (id, tenant_id, rule_name, severity, detected_at, related_user_id, related_service, detail, is_resolved, resolved_at)
type AlertRow = (
    Uuid,
    String,
    String,
    String,
    DateTime<Utc>,
    Option<String>,
    Option<String>,
    String,
    bool,
    Option<DateTime<Utc>>,
);

fn row_to_alert(row: AlertRow) -> Result<Alert, DomainError> {
    let (
        id,
        tenant_id_str,
        rule_name_str,
        severity_str,
        detected_at,
        related_user_id,
        related_service,
        detail,
        is_resolved,
        resolved_at,
    ) = row;

    let tenant_id = TenantId::new(&tenant_id_str)?;
    let rule_name = match rule_name_str.as_str() {
        "DenyThresholdExceeded" => AlertRuleName::DenyThresholdExceeded,
        "ConsecutiveAuthFailure" => AlertRuleName::ConsecutiveAuthFailure,
        other => {
            return Err(DomainError::Infrastructure(format!(
                "Unknown rule_name: {}",
                other
            )))
        }
    };
    let severity = match severity_str.as_str() {
        "HIGH" => AlertSeverity::High,
        "MEDIUM" => AlertSeverity::Medium,
        "LOW" => AlertSeverity::Low,
        other => {
            return Err(DomainError::Infrastructure(format!(
                "Unknown severity: {}",
                other
            )))
        }
    };
    Ok(Alert {
        id,
        tenant_id,
        rule_name,
        severity,
        detected_at,
        related_user_id,
        related_service,
        detail,
        is_resolved,
        resolved_at,
    })
}

#[async_trait]
impl AlertRepository for PostgresAlertRepository {
    async fn save(&self, alert: &Alert) -> Result<(), DomainError> {
        sqlx::query(
            r#"
            INSERT INTO alerts
                (id, tenant_id, rule_name, severity, detected_at,
                 related_user_id, related_service, detail, is_resolved)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(alert.id)
        .bind(alert.tenant_id.value())
        .bind(alert.rule_name.as_str())
        .bind(alert.severity.as_str())
        .bind(alert.detected_at)
        .bind(&alert.related_user_id)
        .bind(&alert.related_service)
        .bind(&alert.detail)
        .bind(alert.is_resolved)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Infrastructure(e.to_string()))?;
        Ok(())
    }

    async fn find_by_tenant(
        &self,
        tenant_id: &TenantId,
        include_resolved: bool,
    ) -> Result<Vec<Alert>, DomainError> {
        let rows: Vec<AlertRow> = if include_resolved {
            sqlx::query_as(
                r#"SELECT id, tenant_id, rule_name, severity, detected_at,
                          related_user_id, related_service, detail, is_resolved, resolved_at
                   FROM alerts
                   WHERE tenant_id = $1
                   ORDER BY detected_at DESC
                   LIMIT 100"#,
            )
            .bind(tenant_id.value())
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DomainError::Infrastructure(e.to_string()))?
        } else {
            sqlx::query_as(
                r#"SELECT id, tenant_id, rule_name, severity, detected_at,
                          related_user_id, related_service, detail, is_resolved, resolved_at
                   FROM alerts
                   WHERE tenant_id = $1 AND is_resolved = false
                   ORDER BY detected_at DESC
                   LIMIT 100"#,
            )
            .bind(tenant_id.value())
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DomainError::Infrastructure(e.to_string()))?
        };

        rows.into_iter().map(row_to_alert).collect()
    }

    async fn resolve(&self, tenant_id: &TenantId, alert_id: Uuid) -> Result<Alert, DomainError> {
        let now = Utc::now();
        let row: Option<AlertRow> = sqlx::query_as(
            r#"UPDATE alerts
               SET is_resolved = true, resolved_at = $1
               WHERE id = $2 AND tenant_id = $3
               RETURNING id, tenant_id, rule_name, severity, detected_at,
                         related_user_id, related_service, detail, is_resolved, resolved_at"#,
        )
        .bind(now)
        .bind(alert_id)
        .bind(tenant_id.value())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        let row =
            row.ok_or_else(|| DomainError::NotFound(format!("Alert {} not found", alert_id)))?;
        row_to_alert(row)
    }

    async fn count_deny_events_in_window(
        &self,
        tenant_id: &TenantId,
        window_secs: i64,
    ) -> Result<u64, DomainError> {
        let row: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM auth_events
               WHERE tenant_id = $1
                 AND decision = 'DENY'
                 AND timestamp >= NOW() - ($2 || ' seconds')::INTERVAL"#,
        )
        .bind(tenant_id.value())
        .bind(window_secs.to_string())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        Ok(row.0 as u64)
    }

    async fn count_consecutive_failures_for_user(
        &self,
        tenant_id: &TenantId,
        user_id: &str,
        window_secs: i64,
    ) -> Result<u64, DomainError> {
        let row: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM auth_events
               WHERE tenant_id = $1
                 AND user_id = $2
                 AND decision = 'DENY'
                 AND timestamp >= NOW() - ($3 || ' seconds')::INTERVAL"#,
        )
        .bind(tenant_id.value())
        .bind(user_id)
        .bind(window_secs.to_string())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        Ok(row.0 as u64)
    }

    async fn get_recently_failed_users(
        &self,
        tenant_id: &TenantId,
        window_secs: i64,
    ) -> Result<Vec<String>, DomainError> {
        let rows: Vec<(String,)> = sqlx::query_as(
            r#"SELECT DISTINCT user_id FROM auth_events
               WHERE tenant_id = $1
                 AND decision = 'DENY'
                 AND timestamp >= NOW() - ($2 || ' seconds')::INTERVAL"#,
        )
        .bind(tenant_id.value())
        .bind(window_secs.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        Ok(rows.into_iter().map(|(s,)| s).collect())
    }

    async fn get_active_tenant_ids(&self) -> Result<Vec<String>, DomainError> {
        let rows: Vec<(String,)> = sqlx::query_as(
            r#"SELECT DISTINCT tenant_id FROM auth_events
               WHERE timestamp >= NOW() - INTERVAL '10 minutes'"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        Ok(rows.into_iter().map(|(s,)| s).collect())
    }
}
