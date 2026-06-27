//! Configuration for batch processing service

use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchProcessingConfig {
    /// Maximum number of items to process in a single batch
    pub batch_size: usize,
    /// Database connection string
    pub database_url: String,
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Initial retry delay in seconds
    pub retry_delay_secs: u64,
    /// Enable archival job
    pub enable_archival: bool,
    /// Archival job cron schedule (default: daily at 2 AM)
    pub archival_schedule: String,
    /// Enable cleanup job
    pub enable_cleanup: bool,
    /// Cleanup job cron schedule (default: daily at 3 AM)
    pub cleanup_schedule: String,
    /// Enable stats aggregation job
    pub enable_stats_aggregation: bool,
    /// Stats aggregation job cron schedule (default: hourly)
    pub stats_aggregation_schedule: String,
    /// Records older than this (in days) will be archived
    pub archival_retention_days: i32,
    /// Jobs older than this (in days) will be cleaned up
    pub cleanup_retention_days: i32,
}

impl Default for BatchProcessingConfig {
    fn default() -> Self {
        Self {
            batch_size: 1000,
            database_url: "postgresql://localhost/batch_processing".to_string(),
            max_retries: 3,
            retry_delay_secs: 60,
            enable_archival: true,
            archival_schedule: "0 2 * * *".to_string(), // 2 AM daily
            enable_cleanup: true,
            cleanup_schedule: "0 3 * * *".to_string(), // 3 AM daily
            enable_stats_aggregation: true,
            stats_aggregation_schedule: "0 * * * *".to_string(), // Every hour
            archival_retention_days: 90,
            cleanup_retention_days: 30,
        }
    }
}

impl BatchProcessingConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();
        
        if let Ok(batch_size) = std::env::var("BATCH_SIZE") {
            config.batch_size = batch_size.parse().unwrap_or(config.batch_size);
        }
        if let Ok(db_url) = std::env::var("DATABASE_URL") {
            config.database_url = db_url;
        }
        if let Ok(max_retries) = std::env::var("MAX_RETRIES") {
            config.max_retries = max_retries.parse().unwrap_or(config.max_retries);
        }
        if let Ok(retry_delay) = std::env::var("RETRY_DELAY_SECS") {
            config.retry_delay_secs = retry_delay.parse().unwrap_or(config.retry_delay_secs);
        }
        if let Ok(retention) = std::env::var("ARCHIVAL_RETENTION_DAYS") {
            config.archival_retention_days = retention.parse().unwrap_or(config.archival_retention_days);
        }
        
        config
    }

    /// Get retry delay for attempt number
    pub fn get_retry_delay(&self, attempt: u32) -> Duration {
        let base_delay = self.retry_delay_secs as u64;
        // Exponential backoff: delay * 2^attempt
        Duration::from_secs(base_delay * 2_u64.pow(attempt))
    }
}
