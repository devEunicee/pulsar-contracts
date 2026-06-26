//! Data models for subscriptions

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Subscription status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum SubscriptionStatus {
    #[serde(rename = "active")]
    Active,
    #[serde(rename = "paused")]
    Paused,
    #[serde(rename = "pending_payment")]
    PendingPayment,
    #[serde(rename = "past_due")]
    PastDue,
    #[serde(rename = "cancelled")]
    Cancelled,
}

impl SubscriptionStatus {
    pub fn as_str(&self) -> &str {
        match self {
            SubscriptionStatus::Active => "active",
            SubscriptionStatus::Paused => "paused",
            SubscriptionStatus::PendingPayment => "pending_payment",
            SubscriptionStatus::PastDue => "past_due",
            SubscriptionStatus::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for SubscriptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Frequency of subscription billing
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BillingFrequency {
    Daily,
    Weekly,
    BiWeekly,
    Monthly,
    Quarterly,
    Annually,
}

impl BillingFrequency {
    pub fn as_str(&self) -> &str {
        match self {
            BillingFrequency::Daily => "daily",
            BillingFrequency::Weekly => "weekly",
            BillingFrequency::BiWeekly => "biweekly",
            BillingFrequency::Monthly => "monthly",
            BillingFrequency::Quarterly => "quarterly",
            BillingFrequency::Annually => "annually",
        }
    }

    pub fn days(&self) -> i32 {
        match self {
            BillingFrequency::Daily => 1,
            BillingFrequency::Weekly => 7,
            BillingFrequency::BiWeekly => 14,
            BillingFrequency::Monthly => 30,
            BillingFrequency::Quarterly => 90,
            BillingFrequency::Annually => 365,
        }
    }
}

/// Subscription definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub id: String,
    pub merchant_id: String,
    pub customer_id: String,
    pub amount: String, // NUMERIC
    pub currency: String,
    pub frequency: BillingFrequency,
    pub status: SubscriptionStatus,
    pub started_at: DateTime<Utc>,
    pub next_payment_at: DateTime<Utc>,
    pub paused_at: Option<DateTime<Utc>>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub cancellation_reason: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Subscription {
    pub fn new(
        merchant_id: String,
        customer_id: String,
        amount: String,
        currency: String,
        frequency: BillingFrequency,
    ) -> Self {
        let now = Utc::now();
        let next_payment = now + chrono::Duration::days(frequency.days() as i64);

        Self {
            id: Uuid::new_v4().to_string(),
            merchant_id,
            customer_id,
            amount,
            currency,
            frequency,
            status: SubscriptionStatus::Active,
            started_at: now,
            next_payment_at: next_payment,
            paused_at: None,
            cancelled_at: None,
            cancellation_reason: None,
            metadata: serde_json::json!({}),
            created_at: now,
            updated_at: now,
        }
    }
}

/// Payment attempt record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentAttempt {
    pub id: String,
    pub subscription_id: String,
    pub amount: String,
    pub status: String,
    pub attempt_number: i32,
    pub error_message: Option<String>,
    pub attempted_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Invoice record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub id: String,
    pub subscription_id: String,
    pub merchant_id: String,
    pub customer_id: String,
    pub amount: String,
    pub currency: String,
    pub status: String,
    pub invoice_date: DateTime<Utc>,
    pub due_date: DateTime<Utc>,
    pub paid_at: Option<DateTime<Utc>>,
    pub invoice_number: String,
    pub line_items: Vec<InvoiceLineItem>,
    pub created_at: DateTime<Utc>,
}

/// Invoice line item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceLineItem {
    pub description: String,
    pub amount: String,
    pub quantity: i32,
}
