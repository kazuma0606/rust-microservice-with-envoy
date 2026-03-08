use crate::domain::value_object::{AlertRuleName, AlertSeverity, TenantId};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Alert {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub rule_name: AlertRuleName,
    pub severity: AlertSeverity,
    pub detected_at: DateTime<Utc>,
    pub related_user_id: Option<String>,
    pub related_service: Option<String>,
    pub detail: String,
    pub is_resolved: bool,
    pub resolved_at: Option<DateTime<Utc>>,
}

impl Alert {
    pub fn new(
        tenant_id: TenantId,
        rule_name: AlertRuleName,
        severity: AlertSeverity,
        related_user_id: Option<String>,
        related_service: Option<String>,
        detail: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            rule_name,
            severity,
            detected_at: Utc::now(),
            related_user_id,
            related_service,
            detail,
            is_resolved: false,
            resolved_at: None,
        }
    }
}
