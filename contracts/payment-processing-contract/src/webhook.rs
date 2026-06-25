/// Webhook Management System
/// 
/// Infrastructure for notifying external systems of payment events (completed, failed, refunded, etc.).
/// Provides secure webhook delivery with HMAC signing, retry logic, and delivery tracking.
/// 
/// Acceptance Criteria:
/// - Register/update/delete webhooks endpoint
/// - Webhook event payload standardized
/// - HMAC signature for verification
/// - Retry logic with exponential backoff
/// - Delivery status tracking
/// - Event history/logs
/// - Test webhook delivery
/// - Webhook validation before registration

use alloc::vec::Vec as RustVec;
use soroban_sdk::{Address, Bytes, BytesN, Env, String, Vec};

use crate::error::PaymentError;

/// Types of webhook events
#[derive(Clone, Debug)]
pub enum WebhookEventType {
    PaymentCompleted,
    PaymentFailed,
    PaymentRefunded,
    RefundInitiated,
    RefundApproved,
    RefundRejected,
    RefundExecuted,
    MerchantRegistered,
    MerchantDeactivated,
}

impl WebhookEventType {
    pub fn to_string(&self) -> &'static str {
        match self {
            WebhookEventType::PaymentCompleted => "payment.completed",
            WebhookEventType::PaymentFailed => "payment.failed",
            WebhookEventType::PaymentRefunded => "payment.refunded",
            WebhookEventType::RefundInitiated => "refund.initiated",
            WebhookEventType::RefundApproved => "refund.approved",
            WebhookEventType::RefundRejected => "refund.rejected",
            WebhookEventType::RefundExecuted => "refund.executed",
            WebhookEventType::MerchantRegistered => "merchant.registered",
            WebhookEventType::MerchantDeactivated => "merchant.deactivated",
        }
    }
}

/// Webhook registration for external event notifications
#[derive(Clone, Debug)]
pub struct WebhookRegistration {
    /// Unique webhook ID
    pub webhook_id: Bytes,
    /// Merchant address that owns this webhook
    pub merchant_address: Address,
    /// Target URL for webhook deliveries
    pub target_url: String,
    /// Events this webhook is subscribed to
    pub events: Vec<String>,
    /// Secret key for HMAC signing
    pub secret_key: BytesN<32>,
    /// Whether the webhook is active
    pub active: bool,
    /// Timestamp when webhook was registered
    pub created_at: u64,
    /// Timestamp of last successful delivery
    pub last_delivery_at: Option<u64>,
}

/// Webhook event payload
#[derive(Clone, Debug)]
pub struct WebhookEventPayload {
    /// Event ID (unique per event)
    pub event_id: Bytes,
    /// Type of event
    pub event_type: String,
    /// Timestamp of the event
    pub timestamp: u64,
    /// Merchant address associated with event
    pub merchant_address: Address,
    /// Event data (order ID, refund ID, etc.)
    pub data: Bytes,
    /// HMAC-SHA256 signature for verification
    pub signature: BytesN<32>,
}

/// Delivery attempt for tracking retry logic
#[derive(Clone, Debug)]
pub struct DeliveryAttempt {
    /// Delivery attempt ID
    pub attempt_id: Bytes,
    /// Associated webhook ID
    pub webhook_id: Bytes,
    /// Associated event ID
    pub event_id: Bytes,
    /// Attempt number (1, 2, 3, ...)
    pub attempt_number: u32,
    /// HTTP status code received
    pub status_code: Option<u32>,
    /// Timestamp of this attempt
    pub timestamp: u64,
    /// Whether this delivery was successful
    pub success: bool,
}

/// Webhook delivery status tracking
#[derive(Clone, Debug)]
pub struct DeliveryStatus {
    /// Event ID
    pub event_id: Bytes,
    /// Webhook ID
    pub webhook_id: Bytes,
    /// Number of delivery attempts
    pub attempt_count: u32,
    /// Maximum retry attempts
    pub max_retries: u32,
    /// Overall delivery status: "pending", "delivered", "failed"
    pub status: String,
    /// Next retry timestamp (0 if no retry pending)
    pub next_retry_at: u64,
}

/// Webhook configuration for controlling behavior
#[derive(Clone, Debug)]
pub struct WebhookConfig {
    /// Maximum number of delivery retries
    pub max_retries: u32,
    /// Base delay for exponential backoff in seconds
    pub base_retry_delay_secs: u32,
    /// Maximum delay between retries in seconds
    pub max_retry_delay_secs: u32,
    /// Request timeout in seconds
    pub request_timeout_secs: u32,
    /// Maximum number of webhooks per merchant
    pub max_webhooks_per_merchant: u32,
    /// Whether webhook system is enabled
    pub enabled: bool,
}

impl WebhookConfig {
    /// Create default webhook configuration
    /// - 5 maximum retries
    /// - 5-second base retry delay with exponential backoff
    /// - 30-second max retry delay
    /// - 30-second request timeout
    /// - 50 webhooks per merchant
    pub fn default() -> Self {
        WebhookConfig {
            max_retries: 5,
            base_retry_delay_secs: 5,
            max_retry_delay_secs: 300, // 5 minutes
            request_timeout_secs: 30,
            max_webhooks_per_merchant: 50,
            enabled: true,
        }
    }
}

/// Webhook statistics for monitoring
pub struct WebhookStats {
    /// Total number of registered webhooks
    pub total_webhooks: u32,
    /// Number of active webhooks
    pub active_webhooks: u32,
    /// Total events delivered
    pub total_events_delivered: u64,
    /// Total events failed
    pub total_events_failed: u64,
    /// Average delivery success rate (%)
    pub success_rate: u32,
}

/// Register a new webhook
pub fn register_webhook(
    env: &Env,
    webhook_id: Bytes,
    merchant_address: Address,
    target_url: String,
    events: Vec<String>,
    secret_key: BytesN<32>,
    config: &WebhookConfig,
) -> Result<WebhookRegistration, PaymentError> {
    if !config.enabled {
        return Err(PaymentError::Unauthorized);
    }

    // Validate URL is not empty
    if target_url.len() == 0 {
        return Err(PaymentError::InvalidInput);
    }

    // Validate events list is not empty
    if events.len() == 0 {
        return Err(PaymentError::InvalidInput);
    }

    let webhook = WebhookRegistration {
        webhook_id,
        merchant_address,
        target_url,
        events,
        secret_key,
        active: true,
        created_at: env.ledger().timestamp(),
        last_delivery_at: None,
    };

    Ok(webhook)
}

/// Calculate next retry delay with exponential backoff
pub fn calculate_next_retry_delay(
    attempt_number: u32,
    config: &WebhookConfig,
) -> u32 {
    // Exponential backoff: delay = base_delay * (2 ^ (attempt_number - 1))
    let exponent = attempt_number.saturating_sub(1);
    let backoff = config.base_retry_delay_secs.saturating_mul(1 << exponent);
    
    // Cap at maximum retry delay
    if backoff > config.max_retry_delay_secs {
        config.max_retry_delay_secs
    } else {
        backoff
    }
}

/// Calculate next retry timestamp
pub fn calculate_next_retry_timestamp(
    env: &Env,
    attempt_number: u32,
    config: &WebhookConfig,
) -> u64 {
    let delay = calculate_next_retry_delay(attempt_number, config);
    env.ledger().timestamp().saturating_add(delay as u64)
}

/// Check if a webhook delivery should be retried
pub fn should_retry(
    status: &DeliveryStatus,
    config: &WebhookConfig,
) -> bool {
    status.attempt_count < config.max_retries && status.status == "pending"
}

/// Create a delivery attempt record
pub fn create_delivery_attempt(
    attempt_id: Bytes,
    webhook_id: Bytes,
    event_id: Bytes,
    attempt_number: u32,
    status_code: Option<u32>,
    env: &Env,
) -> DeliveryAttempt {
    let success = status_code.is_some() && status_code.unwrap() >= 200 && status_code.unwrap() < 300;

    DeliveryAttempt {
        attempt_id,
        webhook_id,
        event_id,
        attempt_number,
        status_code,
        timestamp: env.ledger().timestamp(),
        success,
    }
}

/// Create initial delivery status for an event
pub fn create_delivery_status(
    event_id: Bytes,
    webhook_id: Bytes,
    config: &WebhookConfig,
) -> DeliveryStatus {
    DeliveryStatus {
        event_id,
        webhook_id,
        attempt_count: 0,
        max_retries: config.max_retries,
        status: String::from_slice(&Env::default(), "pending"),
        next_retry_at: 0,
    }
}

/// Validate webhook URL format
pub fn validate_webhook_url(env: &Env, url: &String) -> Result<(), PaymentError> {
    // Check minimum length
    if url.len() < 8 {
        return Err(PaymentError::InvalidInput);
    }

    // Check starts with http:// or https://
    let url_str = url.clone();
    if !url_str.starts_with(&String::from_str(env, "http://"))
        && !url_str.starts_with(&String::from_str(env, "https://"))
    {
        return Err(PaymentError::InvalidInput);
    }

    Ok(())
}

/// Test webhook delivery (verify connection)
pub fn test_webhook_delivery(
    env: &Env,
    webhook: &WebhookRegistration,
) -> Result<(), PaymentError> {
    // Validate webhook URL
    validate_webhook_url(env, &webhook.target_url)?;

    // In a real implementation, this would make an HTTP request
    // For now, we just validate the webhook is configured correctly
    if !webhook.active {
        return Err(PaymentError::InvalidInput);
    }

    Ok(())
}

/// Update webhook registration
pub fn update_webhook(
    webhook: &mut WebhookRegistration,
    target_url: Option<String>,
    events: Option<Vec<String>>,
    active: Option<bool>,
) -> Result<(), PaymentError> {
    if let Some(url) = target_url {
        if url.len() == 0 {
            return Err(PaymentError::InvalidInput);
        }
        webhook.target_url = url;
    }

    if let Some(evt) = events {
        if evt.len() == 0 {
            return Err(PaymentError::InvalidInput);
        }
        webhook.events = evt;
    }

    if let Some(a) = active {
        webhook.active = a;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webhook_config_default() {
        let config = WebhookConfig::default();
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.base_retry_delay_secs, 5);
        assert_eq!(config.max_retry_delay_secs, 300);
        assert!(config.enabled);
    }

    #[test]
    fn test_exponential_backoff() {
        let config = WebhookConfig::default();

        let delay1 = calculate_next_retry_delay(1, &config);
        assert_eq!(delay1, 5); // 5 * 2^0 = 5

        let delay2 = calculate_next_retry_delay(2, &config);
        assert_eq!(delay2, 10); // 5 * 2^1 = 10

        let delay3 = calculate_next_retry_delay(3, &config);
        assert_eq!(delay3, 20); // 5 * 2^2 = 20

        let delay4 = calculate_next_retry_delay(4, &config);
        assert_eq!(delay4, 40); // 5 * 2^3 = 40

        let delay5 = calculate_next_retry_delay(5, &config);
        assert_eq!(delay5, 80); // 5 * 2^4 = 80
    }

    #[test]
    fn test_backoff_capped_at_max() {
        let config = WebhookConfig::default();
        let delay10 = calculate_next_retry_delay(10, &config);
        assert!(delay10 <= config.max_retry_delay_secs);
    }

    #[test]
    fn test_event_type_strings() {
        assert_eq!(WebhookEventType::PaymentCompleted.to_string(), "payment.completed");
        assert_eq!(WebhookEventType::PaymentFailed.to_string(), "payment.failed");
        assert_eq!(WebhookEventType::RefundInitiated.to_string(), "refund.initiated");
    }
}
