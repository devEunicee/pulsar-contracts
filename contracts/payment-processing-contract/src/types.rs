use soroban_sdk::{contracttype, Address, Bytes, String, Vec};

// ── Merchant ──────────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MerchantCategory {
    Retail,
    Food,
    Services,
    Digital,
    Other,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Merchant {
    pub address: Address,
    pub name: String,
    pub description: String,
    pub contact_info: String,
    pub category: MerchantCategory,
    pub active: bool,
    pub registered_at: u64,
}

// ── Payment ───────────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PaymentStatus {
    Completed,
    PartiallyRefunded,
    FullyRefunded,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaymentOrder {
    pub order_id: Bytes,
    pub merchant_address: Address,
    pub payer: Address,
    pub token: Address,
    pub amount: i128,
    pub description: String,
    pub expires_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaymentRecord {
    pub order_id: Bytes,
    pub merchant_address: Address,
    pub payer: Address,
    pub token: Address,
    pub amount: i128,
    pub refunded_amount: i128,
    pub status: PaymentStatus,
    pub paid_at: u64,
}

// ── Refund ────────────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RefundStatus {
    Pending,
    Approved,
    Rejected,
    Completed,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RefundRecord {
    pub refund_id: Bytes,
    pub order_id: Bytes,
    pub amount: i128,
    pub reason: String,
    pub status: RefundStatus,
    pub initiated_by: Address,
    pub initiated_at: u64,
}

// ── Multisig ──────────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MultisigPayment {
    pub payment_id: Bytes,
    pub order: PaymentOrder,
    pub required_signers: Vec<Address>,
    pub signatures: Vec<Address>,
    pub executed: bool,
    pub expires_at: u64,
    pub created_at: u64,
}

// ── Query helpers ─────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SortField {
    Date,
    Amount,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StatusFilter {
    Any,
    Completed,
    PartiallyRefunded,
    FullyRefunded,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaymentFilter {
    pub date_start: Option<u64>,
    pub date_end: Option<u64>,
    pub amount_min: Option<i128>,
    pub amount_max: Option<i128>,
    pub token: Option<Address>,
    pub status: StatusFilter,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaymentPage {
    pub records: Vec<PaymentRecord>,
    pub next_cursor: Option<Bytes>,
    pub total: u32,
}

// ── Global stats ──────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GlobalStats {
    pub total_payments: u64,
    pub total_volume: i128,
    pub total_refunds: u64,
    pub total_refund_volume: i128,
}

// ── Storage keys ──────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    Merchant(Address),
    Payment(Bytes),
    MerchantPayments(Address),
    PayerPayments(Address),
    Refund(Bytes),
    Multisig(Bytes),
    CleanupPeriod,
    DefaultMultisigExpiry,
    GlobalPaymentIndex,
    GlobalStats,
}
