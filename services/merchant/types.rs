/// Merchant management service — types and domain objects.
/// Issue #273: Create Merchant Management Service

/// Mirrors on-chain MerchantCategory; kept separate for off-chain service layer.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum MerchantCategory {
    Retail,
    Food,
    Services,
    Digital,
    Other,
}

/// Full merchant profile as stored off-chain.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Merchant {
    pub address: String,
    pub name: String,
    pub description: String,
    pub contact_info: String,
    pub category: MerchantCategory,
    pub active: bool,
    pub whitelisted: bool,
    pub registered_at: u64,
    pub updated_at: u64,
}

/// Fields allowed in a profile update (all optional).
#[derive(Debug, Default, serde::Deserialize)]
pub struct UpdateMerchantRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub contact_info: Option<String>,
    pub category: Option<MerchantCategory>,
}

/// Aggregated stats for a single merchant.
#[derive(Debug, serde::Serialize)]
pub struct MerchantStats {
    pub address: String,
    pub payment_count: u64,
    pub total_volume: i128,
    pub refund_count: u64,
    pub total_refunded: i128,
}

/// Search / filter parameters for merchant queries.
#[derive(Debug, Default, serde::Deserialize)]
pub struct MerchantFilter {
    pub name_contains: Option<String>,
    pub category: Option<MerchantCategory>,
    pub active: Option<bool>,
    pub whitelisted: Option<bool>,
}

/// Audit log entry for a merchant change.
#[derive(Debug, serde::Serialize)]
pub struct AuditEntry {
    pub merchant_address: String,
    pub action: String,
    pub changed_by: String,
    pub changed_at: u64,
    pub details: Option<String>,
}
