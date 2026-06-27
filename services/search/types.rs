/// Advanced search & filtering types — Issue #275

use serde::{Deserialize, Serialize};

// ── Sort helpers ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortOrder {
    Ascending,
    Descending,
}

// ── Payment search ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentSortField {
    Date,
    Amount,
    MerchantAddress,
    PayerAddress,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentStatusFilter {
    Any,
    Completed,
    PartiallyRefunded,
    FullyRefunded,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct PaymentSearchQuery {
    // text
    pub merchant_address: Option<String>,
    pub payer_address: Option<String>,
    pub token_address: Option<String>,
    // amount range
    pub amount_min: Option<i128>,
    pub amount_max: Option<i128>,
    // date range
    pub date_start: Option<u64>,
    pub date_end: Option<u64>,
    // status (supports multiple values)
    pub statuses: Option<Vec<PaymentStatusFilter>>,
    // sort
    pub sort_field: Option<PaymentSortField>,
    pub sort_order: Option<SortOrder>,
    // pagination
    pub cursor: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PaymentRecord {
    pub order_id: String,
    pub merchant_address: String,
    pub payer_address: String,
    pub token_address: String,
    pub amount: i128,
    pub refunded_amount: i128,
    pub status: String,
    pub paid_at: u64,
}

#[derive(Debug, Serialize)]
pub struct PaymentPage {
    pub records: Vec<PaymentRecord>,
    pub next_cursor: Option<String>,
    pub total: u64,
}

// ── Merchant search ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MerchantSortField {
    Name,
    RegisteredAt,
    Category,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct MerchantSearchQuery {
    pub name_contains: Option<String>,
    pub category: Option<String>,
    pub active: Option<bool>,
    pub whitelisted: Option<bool>,
    pub sort_field: Option<MerchantSortField>,
    pub sort_order: Option<SortOrder>,
    pub cursor: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MerchantRecord {
    pub address: String,
    pub name: String,
    pub category: String,
    pub active: bool,
    pub whitelisted: bool,
    pub registered_at: u64,
}

#[derive(Debug, Serialize)]
pub struct MerchantPage {
    pub records: Vec<MerchantRecord>,
    pub next_cursor: Option<String>,
    pub total: u64,
}

// ── Refund search ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RefundSortField {
    InitiatedAt,
    Amount,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct RefundSearchQuery {
    pub order_id: Option<String>,
    pub initiated_by: Option<String>,
    pub statuses: Option<Vec<String>>,
    pub date_start: Option<u64>,
    pub date_end: Option<u64>,
    pub amount_min: Option<i128>,
    pub amount_max: Option<i128>,
    pub sort_field: Option<RefundSortField>,
    pub sort_order: Option<SortOrder>,
    pub cursor: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RefundRecord {
    pub refund_id: String,
    pub order_id: String,
    pub amount: i128,
    pub status: String,
    pub initiated_by: String,
    pub initiated_at: u64,
}

#[derive(Debug, Serialize)]
pub struct RefundPage {
    pub records: Vec<RefundRecord>,
    pub next_cursor: Option<String>,
    pub total: u64,
}
