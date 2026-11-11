use async_trait::async_trait;
use chrono::Utc;
use serde_json::json;
use sqlx::SqlitePool;

use crate::config::AppConfig;

#[async_trait]
pub trait NotificationGateway: Send + Sync {
    async fn notify_wali(
        &self,
        visit_id: Option<&str>,
        recipient_phone: &str,
        event_type: &str,
        message_body: &str,
    );
}

pub struct ConfigurableWaGateway {
    db: SqlitePool,
    enabled: bool,
    gateway_url: String,
    api_key: String,
    http_client: reqwest::Client,
}

impl ConfigurableWaGateway {
    pub fn new(db: SqlitePool, cfg: &AppConfig) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .unwrap_or_default();

        Self {
            db,
            enabled: cfg.wa_gateway_enabled,
            gateway_url: cfg.wa_gateway_url.clone(),
            api_key: cfg.wa_gateway_api_key.clone(),
            http_client,
        }
    }
}

#[async_trait]
impl NotificationGateway for ConfigurableWaGateway {
    async fn notify_wali(
        &self,
        visit_id: Option<&str>,
        recipient_phone: &str,
        event_type: &str,
        message_body: &str,
    ) {
        if !self.enabled || self.gateway_url.trim().is_empty() {
            let _ = sqlx::query(
                "INSERT INTO notification_outbox (visit_id, recipient_phone, event_type, message_body, delivery_status)
                 VALUES (?1, ?2, ?3, ?4, 'SKIPPED_DISABLED')"
            )
            .bind(visit_id)
            .bind(recipient_phone)
            .bind(event_type)
            .bind(message_body)
            .execute(&self.db)
            .await;
            return;
        }

        let insert_res = sqlx::query(
            "INSERT INTO notification_outbox (visit_id, recipient_phone, event_type, message_body, delivery_status)
             VALUES (?1, ?2, ?3, ?4, 'PENDING')"
        )
        .bind(visit_id)
        .bind(recipient_phone)
        .bind(event_type)
        .bind(message_body)
        .execute(&self.db)
        .await;

        let Ok(done) = insert_res else { return };
        let outbox_id = done.last_insert_rowid();

        let payload = json!({
            "chatId": format!("{}@c.us", recipient_phone.trim_start_matches('+')),
            "phone": recipient_phone,
            "text": message_body,
        });

        let mut req = self.http_client.post(&self.gateway_url).json(&payload);
        if !self.api_key.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", self.api_key));
        }

        match req.send().await {
            Ok(resp) if resp.status().is_success() => {
                let now_iso = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
                let _ = sqlx::query(
                    "UPDATE notification_outbox SET delivery_status = 'SENT', sent_at = ?1 WHERE id = ?2"
                )
                .bind(now_iso)
                .bind(outbox_id)
                .execute(&self.db)
                .await;
            }
            Ok(resp) => {
                let err_msg = format!("HTTP {}", resp.status());
                let _ = sqlx::query(
                    "UPDATE notification_outbox SET delivery_status = 'FAILED', error_message = ?1 WHERE id = ?2"
                )
                .bind(err_msg)
                .bind(outbox_id)
                .execute(&self.db)
                .await;
            }
            Err(err) => {
                let _ = sqlx::query(
                    "UPDATE notification_outbox SET delivery_status = 'FAILED', error_message = ?1 WHERE id = ?2"
                )
                .bind(err.to_string())
                .bind(outbox_id)
                .execute(&self.db)
                .await;
            }
        }
    }
}
