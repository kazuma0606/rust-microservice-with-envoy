use std::sync::Arc;

use crate::domain::entity::Alert;
use crate::domain::error::DomainError;
use crate::domain::value_object::{AlertRuleName, AlertSeverity, TenantId};
use crate::port::alert_repository::AlertRepository;
use crate::port::notifier::Notifier;
use crate::port::webhook_config_repository::WebhookConfigRepository;

pub struct DetectAnomalyUseCase {
    alert_repo: Arc<dyn AlertRepository>,
    webhook_config_repo: Arc<dyn WebhookConfigRepository>,
    notifier: Arc<dyn Notifier>,
    deny_threshold: u64,
    consecutive_failure_threshold: u64,
    window_secs: i64,
}

impl DetectAnomalyUseCase {
    pub fn new(
        alert_repo: Arc<dyn AlertRepository>,
        webhook_config_repo: Arc<dyn WebhookConfigRepository>,
        notifier: Arc<dyn Notifier>,
        deny_threshold: u64,
        consecutive_failure_threshold: u64,
    ) -> Self {
        Self {
            alert_repo,
            webhook_config_repo,
            notifier,
            deny_threshold,
            consecutive_failure_threshold,
            window_secs: 300, // 5 minutes
        }
    }

    pub async fn run_detection_cycle(&self) -> Result<(), DomainError> {
        let tenant_ids = self.alert_repo.get_active_tenant_ids().await?;

        for tenant_id_str in tenant_ids {
            let tenant_id = match TenantId::new(&tenant_id_str) {
                Ok(id) => id,
                Err(e) => {
                    tracing::warn!("Skipping invalid tenant_id {}: {}", tenant_id_str, e);
                    continue;
                }
            };

            if let Err(e) = self.check_deny_threshold(&tenant_id).await {
                tracing::error!("deny threshold check failed for {}: {}", tenant_id_str, e);
            }

            if let Err(e) = self.check_consecutive_failures(&tenant_id).await {
                tracing::error!(
                    "consecutive failure check failed for {}: {}",
                    tenant_id_str,
                    e
                );
            }
        }

        Ok(())
    }

    async fn check_deny_threshold(&self, tenant_id: &TenantId) -> Result<(), DomainError> {
        let deny_count = self
            .alert_repo
            .count_deny_events_in_window(tenant_id, self.window_secs)
            .await?;

        if deny_count > self.deny_threshold {
            let detail = format!(
                "{}分間で DENY イベントが {}件（閾値: {}件）を超過しました",
                self.window_secs / 60,
                deny_count,
                self.deny_threshold
            );

            let alert = Alert::new(
                tenant_id.clone(),
                AlertRuleName::DenyThresholdExceeded,
                AlertSeverity::High,
                None,
                None,
                detail,
            );

            self.save_and_notify(alert).await?;
        }

        Ok(())
    }

    async fn check_consecutive_failures(&self, tenant_id: &TenantId) -> Result<(), DomainError> {
        let failed_users = self
            .alert_repo
            .get_recently_failed_users(tenant_id, 60)
            .await?;

        for user_id in failed_users {
            let count = self
                .alert_repo
                .count_consecutive_failures_for_user(tenant_id, &user_id, 60)
                .await?;

            if count >= self.consecutive_failure_threshold {
                let detail = format!(
                    "ユーザー {} が60秒以内に {}回連続認証失敗しました（閾値: {}回）",
                    user_id, count, self.consecutive_failure_threshold
                );

                let alert = Alert::new(
                    tenant_id.clone(),
                    AlertRuleName::ConsecutiveAuthFailure,
                    AlertSeverity::Medium,
                    Some(user_id.clone()),
                    None,
                    detail,
                );

                self.save_and_notify(alert).await?;
            }
        }

        Ok(())
    }

    async fn save_and_notify(&self, alert: Alert) -> Result<(), DomainError> {
        self.alert_repo.save(&alert).await?;

        let rule_name = alert.rule_name.as_str();
        tracing::info!(
            tenant_id = %alert.tenant_id,
            rule = rule_name,
            "Alert generated"
        );
        metrics::counter!(
            "authpulse_alert_generated_total",
            "rule" => rule_name.to_string()
        )
        .increment(1);

        if let Ok(Some(config)) = self
            .webhook_config_repo
            .find_by_tenant(&alert.tenant_id)
            .await
        {
            if config.is_active {
                match self.notifier.notify(&config, &alert).await {
                    Ok(()) => {
                        metrics::counter!("authpulse_webhook_notification_total", "status" => "ok")
                            .increment(1);
                    }
                    Err(e) => {
                        tracing::warn!("Webhook notification failed: {}", e);
                        metrics::counter!(
                            "authpulse_webhook_notification_total",
                            "status" => "error"
                        )
                        .increment(1);
                    }
                }
            }
        }

        Ok(())
    }
}
