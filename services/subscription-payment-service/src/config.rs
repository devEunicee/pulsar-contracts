//! Configuration for subscription payment service

use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionConfig {
    /// Database connection string
    pub database_url: String,
    /// Maximum retry attempts for failed payments
    pub max_payment_retries: u32,
    /// Initial retry delay in seconds
    pub retry_delay_secs: u64,
    /// Enable automatic payment processing
    pub enable_auto_payments: bool,
    /// Payment processing interval in seconds
    pub payment_interval_secs: u64,
    /// Enable invoice generation
    pub enable_invoice_generation: bool,
    /// Enable event emission
    pub enable_events: bool,
    /// Number of days to keep payment history
    pub payment_history_retention_days: i32,
    /// Grace period for late payments (in hours)
    pub late_payment_grace_hours: i32,
}

impl Default for SubscriptionConfig {
    fn default() -> Self {
        Self {
            database_url: "postgresql://localhost/subscriptions".to_string(),
            max_payment_retries: 3,
            retry_delay_secs: 300, // 5 minutes
            enable_auto_payments: true,
            payment_interval_secs: 3600, // 1 hour
            enable_invoice_generation: true,
            enable_events: true,
            payment_history_retention_days: 365,
            late_payment_grace_hours: 24,
        }
    }
}

impl SubscriptionConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();
        
        if let Ok(db_url) = std::env::var("DATABASE_URL") {
            config.database_url = db_url;
        }
        if let Ok(max_retries) = std::env::var("MAX_PAYMENT_RETRIES") {
            config.max_payment_retries = max_retries.parse().unwrap_or(config.max_payment_retries);
        }
        if let Ok(retry_delay) = std::env::var("RETRY_DELAY_SECS") {
            config.retry_delay_secs = retry_delay.parse().unwrap_or(config.retry_delay_secs);
        }
        
        config
    }

    /// Get retry delay for attempt number
    pub fn get_retry_delay(&self, attempt: u32) -> Duration {
        let base_delay = self.retry_delay_secs;
        // Exponential backoff: delay * 2^attempt
        Duration::from_secs(base_delay * 2_u64.pow(attempt))
    }
}
