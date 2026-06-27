//! Payment processing for subscriptions

use crate::{models::*, SubscriptionConfig, Error, Result};
use chrono::Utc;
use sqlx::PgPool;
use std::sync::Arc;

/// Payment processor for subscriptions
pub struct PaymentProcessor {
    pool: Arc<PgPool>,
    config: SubscriptionConfig,
}

impl PaymentProcessor {
    pub fn new(pool: Arc<PgPool>, config: SubscriptionConfig) -> Self {
        Self { pool, config }
    }

    /// Create a new subscription
    pub async fn create_subscription(&self, subscription: Subscription) -> Result<String> {
        let sub_id = &subscription.id;
        
        sqlx::query(
            r#"
            INSERT INTO subscriptions 
            (id, merchant_id, customer_id, amount, currency, frequency, status, started_at, next_payment_at, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(sub_id)
        .bind(&subscription.merchant_id)
        .bind(&subscription.customer_id)
        .bind(&subscription.amount)
        .bind(&subscription.currency)
        .bind(subscription.frequency.as_str())
        .bind(subscription.status.as_str())
        .bind(subscription.started_at)
        .bind(subscription.next_payment_at)
        .bind(serde_json::to_string(&subscription.metadata)?)
        .execute(self.pool.as_ref())
        .await?;

        tracing::info!("Subscription created: {}", sub_id);
        Ok(sub_id.clone())
    }

    /// Get subscription details
    pub async fn get_subscription(&self, subscription_id: &str) -> Result<Option<Subscription>> {
        let row = sqlx::query(
            r#"
            SELECT id, merchant_id, customer_id, amount, currency, frequency, status,
                   started_at, next_payment_at, paused_at, cancelled_at, cancellation_reason,
                   metadata, created_at, updated_at
            FROM subscriptions WHERE id = $1
            "#,
        )
        .bind(subscription_id)
        .fetch_optional(self.pool.as_ref())
        .await?;

        if let Some(row) = row {
            use sqlx::Row;
            let sub = Subscription {
                id: row.get("id"),
                merchant_id: row.get("merchant_id"),
                customer_id: row.get("customer_id"),
                amount: row.get("amount"),
                currency: row.get("currency"),
                frequency: match row.get::<String, _>("frequency").as_str() {
                    "daily" => BillingFrequency::Daily,
                    "weekly" => BillingFrequency::Weekly,
                    "biweekly" => BillingFrequency::BiWeekly,
                    "monthly" => BillingFrequency::Monthly,
                    "quarterly" => BillingFrequency::Quarterly,
                    "annually" => BillingFrequency::Annually,
                    _ => BillingFrequency::Monthly,
                },
                status: match row.get::<String, _>("status").as_str() {
                    "active" => SubscriptionStatus::Active,
                    "paused" => SubscriptionStatus::Paused,
                    "pending_payment" => SubscriptionStatus::PendingPayment,
                    "past_due" => SubscriptionStatus::PastDue,
                    "cancelled" => SubscriptionStatus::Cancelled,
                    _ => SubscriptionStatus::Active,
                },
                started_at: row.get("started_at"),
                next_payment_at: row.get("next_payment_at"),
                paused_at: row.get("paused_at"),
                cancelled_at: row.get("cancelled_at"),
                cancellation_reason: row.get("cancellation_reason"),
                metadata: serde_json::from_str(&row.get::<String, _>("metadata"))?,
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            Ok(Some(sub))
        } else {
            Ok(None)
        }
    }

    /// Pause subscription
    pub async fn pause_subscription(&self, subscription_id: &str) -> Result<()> {
        let result = sqlx::query("UPDATE subscriptions SET status = $1, paused_at = NOW() WHERE id = $2")
            .bind(SubscriptionStatus::Paused.as_str())
            .bind(subscription_id)
            .execute(self.pool.as_ref())
            .await?;

        if result.rows_affected() == 0 {
            return Err(Error::SubscriptionNotFound(subscription_id.to_string()));
        }

        tracing::info!("Subscription paused: {}", subscription_id);
        Ok(())
    }

    /// Resume subscription
    pub async fn resume_subscription(&self, subscription_id: &str) -> Result<()> {
        let result = sqlx::query(
            "UPDATE subscriptions SET status = $1, paused_at = NULL WHERE id = $2 AND status = $3"
        )
        .bind(SubscriptionStatus::Active.as_str())
        .bind(subscription_id)
        .bind(SubscriptionStatus::Paused.as_str())
        .execute(self.pool.as_ref())
        .await?;

        if result.rows_affected() == 0 {
            return Err(Error::SubscriptionNotFound(subscription_id.to_string()));
        }

        tracing::info!("Subscription resumed: {}", subscription_id);
        Ok(())
    }

    /// Cancel subscription
    pub async fn cancel_subscription(&self, subscription_id: &str, reason: &str) -> Result<()> {
        let result = sqlx::query(
            r#"
            UPDATE subscriptions 
            SET status = $1, cancelled_at = NOW(), cancellation_reason = $2
            WHERE id = $3
            "#
        )
        .bind(SubscriptionStatus::Cancelled.as_str())
        .bind(reason)
        .bind(subscription_id)
        .execute(self.pool.as_ref())
        .await?;

        if result.rows_affected() == 0 {
            return Err(Error::SubscriptionNotFound(subscription_id.to_string()));
        }

        tracing::info!("Subscription cancelled: {} (reason: {})", subscription_id, reason);
        Ok(())
    }

    /// Record payment attempt
    pub async fn record_payment_attempt(
        &self,
        subscription_id: &str,
        amount: &str,
        status: &str,
        error_message: Option<&str>,
    ) -> Result<String> {
        let attempt_id = uuid::Uuid::new_v4().to_string();

        sqlx::query(
            r#"
            INSERT INTO payment_attempts 
            (id, subscription_id, amount, status, error_message, attempted_at)
            VALUES ($1, $2, $3, $4, $5, NOW())
            "#,
        )
        .bind(&attempt_id)
        .bind(subscription_id)
        .bind(amount)
        .bind(status)
        .bind(error_message)
        .execute(self.pool.as_ref())
        .await?;

        Ok(attempt_id)
    }

    /// Get payment attempts for subscription
    pub async fn get_payment_attempts(&self, subscription_id: &str) -> Result<Vec<PaymentAttempt>> {
        let rows = sqlx::query(
            r#"
            SELECT id, subscription_id, amount, status, attempt_number, error_message, attempted_at, completed_at
            FROM payment_attempts
            WHERE subscription_id = $1
            ORDER BY attempted_at DESC
            "#,
        )
        .bind(subscription_id)
        .fetch_all(self.pool.as_ref())
        .await?;

        let mut attempts = Vec::new();
        for row in rows {
            use sqlx::Row;
            attempts.push(PaymentAttempt {
                id: row.get("id"),
                subscription_id: row.get("subscription_id"),
                amount: row.get("amount"),
                status: row.get("status"),
                attempt_number: row.get("attempt_number"),
                error_message: row.get("error_message"),
                attempted_at: row.get("attempted_at"),
                completed_at: row.get("completed_at"),
            });
        }

        Ok(attempts)
    }

    /// Process due payments
    pub async fn process_due_payments(&self) -> Result<usize> {
        let now = Utc::now();
        
        let rows = sqlx::query(
            r#"
            SELECT id, merchant_id, customer_id, amount FROM subscriptions
            WHERE status IN ('active', 'pending_payment')
            AND next_payment_at <= $1
            "#,
        )
        .bind(now)
        .fetch_all(self.pool.as_ref())
        .await?;

        let count = rows.len();

        for row in rows {
            use sqlx::Row;
            let sub_id: String = row.get("id");
            let amount: String = row.get("amount");
            
            // Record payment attempt
            if let Err(e) = self.record_payment_attempt(&sub_id, &amount, "processing", None).await {
                tracing::error!("Failed to record payment attempt for {}: {:?}", sub_id, e);
            }
        }

        tracing::info!("Processed {} due payments", count);
        Ok(count)
    }
}
