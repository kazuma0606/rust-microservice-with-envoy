use async_trait::async_trait;
use serde::Serialize;
use std::time::Duration;

use crate::domain::entity::{Alert, WebhookConfig};
use crate::domain::error::DomainError;
use crate::port::notifier::Notifier;

#[derive(Serialize)]
struct WebhookPayload {
    alert_id: String,
    tenant_id: String,
    rule_name: String,
    severity: String,
    detail: String,
    detected_at_unix_ms: i64,
    related_user_id: Option<String>,
    related_service: Option<String>,
}

pub struct WebhookNotifier {
    client: reqwest::Client,
    max_retries: u32,
}

impl WebhookNotifier {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            max_retries: 3,
        }
    }
}

impl Default for WebhookNotifier {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Notifier for WebhookNotifier {
    async fn notify(&self, config: &WebhookConfig, alert: &Alert) -> Result<(), DomainError> {
        let payload = WebhookPayload {
            alert_id: alert.id.to_string(),
            tenant_id: alert.tenant_id.to_string(),
            rule_name: alert.rule_name.as_str().to_string(),
            severity: alert.severity.as_str().to_string(),
            detail: alert.detail.clone(),
            detected_at_unix_ms: alert.detected_at.timestamp_millis(),
            related_user_id: alert.related_user_id.clone(),
            related_service: alert.related_service.clone(),
        };

        let mut last_error = String::new();

        for attempt in 1..=self.max_retries {
            let result = self.client.post(&config.url).json(&payload).send().await;

            match result {
                Ok(resp) if resp.status().is_success() => {
                    tracing::info!(
                        tenant_id = %config.tenant_id,
                        url = %config.url,
                        "Webhook notification sent successfully"
                    );
                    return Ok(());
                }
                Ok(resp) => {
                    last_error = format!("HTTP {}", resp.status());
                    tracing::warn!(
                        attempt,
                        error = %last_error,
                        "Webhook notification failed, will retry"
                    );
                }
                Err(e) => {
                    last_error = e.to_string();
                    tracing::warn!(
                        attempt,
                        error = %last_error,
                        "Webhook notification error, will retry"
                    );
                }
            }

            if attempt < self.max_retries {
                let backoff = Duration::from_millis(500 * 2u64.pow(attempt - 1));
                tokio::time::sleep(backoff).await;
            }
        }

        tracing::error!(
            tenant_id = %config.tenant_id,
            url = %config.url,
            error = %last_error,
            "Webhook notification failed after {} retries", self.max_retries
        );

        Err(DomainError::Infrastructure(format!(
            "Webhook notification failed after {} retries: {}",
            self.max_retries, last_error
        )))
    }
}
