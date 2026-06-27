//! Event emission for subscription events

use crate::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionEvent {
    pub id: String,
    pub subscription_id: String,
    pub event_type: String,
    pub data: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Event emitter for subscription events
pub struct EventEmitter {
    pool: Arc<PgPool>,
}

impl EventEmitter {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// Emit a subscription event
    pub async fn emit(&self, subscription_id: &str, event_type: &str, data: serde_json::Value) -> Result<String> {
        let event_id = uuid::Uuid::new_v4().to_string();

        sqlx::query(
            r#"
            INSERT INTO subscription_events (id, subscription_id, event_type, data, created_at)
            VALUES ($1, $2, $3, $4, NOW())
            "#,
        )
        .bind(&event_id)
        .bind(subscription_id)
        .bind(event_type)
        .bind(serde_json::to_string(&data)?)
        .execute(self.pool.as_ref())
        .await?;

        tracing::info!("Event emitted: {} for subscription {}", event_type, subscription_id);
        Ok(event_id)
    }

    /// Get events for subscription
    pub async fn get_events(&self, subscription_id: &str) -> Result<Vec<SubscriptionEvent>> {
        let rows = sqlx::query(
            r#"
            SELECT id, subscription_id, event_type, data, created_at
            FROM subscription_events
            WHERE subscription_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(subscription_id)
        .fetch_all(self.pool.as_ref())
        .await?;

        let mut events = Vec::new();
        for row in rows {
            use sqlx::Row;
            events.push(SubscriptionEvent {
                id: row.get("id"),
                subscription_id: row.get("subscription_id"),
                event_type: row.get("event_type"),
                data: serde_json::from_str(&row.get::<String, _>("data"))?,
                created_at: row.get("created_at"),
            });
        }

        Ok(events)
    }
}
