use soroban_sdk::{Address, Bytes, Env, Vec};

use crate::error::PaymentError;
use crate::types::{
    AdminConfig, DataKey, GlobalStats, Merchant, MultisigPayment, PaymentRecord, RefundRecord,
};

// ── TTL constants ─────────────────────────────────────────────────────────────

/// ~1 year in ledgers (5-second ledger close time).
pub const TTL_LEDGERS: u32 = 6_307_200;
/// Refresh TTL when remaining lifetime drops below ~6 months.
pub const TTL_THRESHOLD: u32 = TTL_LEDGERS / 2;

// ── Admin ─────────────────────────────────────────────────────────────────────

pub fn get_admin(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::Admin)
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

pub fn get_admin_config(env: &Env) -> Option<AdminConfig> {
    env.storage().instance().get(&DataKey::AdminConfig)
}

pub fn set_admin_config(env: &Env, config: &AdminConfig) {
    env.storage().instance().set(&DataKey::AdminConfig, config);
}

// ── Contract version ──────────────────────────────────────────────────────────

pub fn get_contract_version(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&DataKey::ContractVersion)
        .unwrap_or(0)
}

pub fn set_contract_version(env: &Env, version: u32) {
    env.storage()
        .instance()
        .set(&DataKey::ContractVersion, &version);
}

// ── Merchant ──────────────────────────────────────────────────────────────────

pub fn get_merchant(env: &Env, address: &Address) -> Option<Merchant> {
    let key = DataKey::Merchant(address.clone());
    let result = env.storage().persistent().get(&key);
    if result.is_some() {
        env.storage()
            .persistent()
            .extend_ttl(&key, TTL_THRESHOLD, TTL_LEDGERS);
    }
    result
}

pub fn save_merchant(env: &Env, merchant: &Merchant) {
    let key = DataKey::Merchant(merchant.address.clone());
    env.storage().persistent().set(&key, merchant);
    env.storage()
        .persistent()
        .extend_ttl(&key, TTL_THRESHOLD, TTL_LEDGERS);
}

// ── Payment ───────────────────────────────────────────────────────────────────

pub fn get_payment(env: &Env, order_id: &Bytes) -> Option<PaymentRecord> {
    let key = DataKey::Payment(order_id.clone());
    let result = env.storage().persistent().get(&key);
    if result.is_some() {
        env.storage()
            .persistent()
            .extend_ttl(&key, TTL_THRESHOLD, TTL_LEDGERS);
    }
    result
}

pub fn save_payment(env: &Env, record: &PaymentRecord) {
    let key = DataKey::Payment(record.order_id.clone());
    env.storage().persistent().set(&key, record);
    env.storage()
        .persistent()
        .extend_ttl(&key, TTL_THRESHOLD, TTL_LEDGERS);
}

pub fn remove_payment(env: &Env, order_id: &Bytes) {
    env.storage()
        .persistent()
        .remove(&DataKey::Payment(order_id.clone()));
}

// ── Payment index lists ───────────────────────────────────────────────────────

const CHUNK_SIZE: u32 = 100;

pub fn get_merchant_payment_ids(env: &Env, merchant: &Address) -> Vec<Bytes> {
    let key = DataKey::MerchantPayments(merchant.clone());
    let result: Option<Vec<Bytes>> = env.storage().persistent().get(&key);
    if result.is_some() {
        env.storage()
            .persistent()
            .extend_ttl(&key, TTL_THRESHOLD, TTL_LEDGERS);
    }
    result.unwrap_or_else(|| Vec::new(env))
}

pub fn push_merchant_payment_id(env: &Env, merchant: &Address, order_id: &Bytes) {
    let mut ids = get_merchant_payment_ids(env, merchant);
    ids.push_back(order_id.clone());
    let key = DataKey::MerchantPayments(merchant.clone());
    env.storage().persistent().set(&key, &ids);
    env.storage()
        .persistent()
        .extend_ttl(&key, TTL_THRESHOLD, TTL_LEDGERS);
}

pub fn get_payer_payment_ids(env: &Env, payer: &Address) -> Vec<Bytes> {
    let key = DataKey::PayerPayments(payer.clone());
    let result: Option<Vec<Bytes>> = env.storage().persistent().get(&key);
    if result.is_some() {
        env.storage()
            .persistent()
            .extend_ttl(&key, TTL_THRESHOLD, TTL_LEDGERS);
    }
    result.unwrap_or_else(|| Vec::new(env))
}

pub fn push_payer_payment_id(env: &Env, payer: &Address, order_id: &Bytes) {
    let mut ids = get_payer_payment_ids(env, payer);
    ids.push_back(order_id.clone());
    let key = DataKey::PayerPayments(payer.clone());
    env.storage().persistent().set(&key, &ids);
    env.storage()
        .persistent()
        .extend_ttl(&key, TTL_THRESHOLD, TTL_LEDGERS);
}

pub fn get_global_payment_ids(env: &Env) -> Vec<Bytes> {
    let key = DataKey::GlobalPaymentIndex;
    let result: Option<Vec<Bytes>> = env.storage().persistent().get(&key);
    if result.is_some() {
        env.storage()
            .persistent()
            .extend_ttl(&key, TTL_THRESHOLD, TTL_LEDGERS);
    }
    result.unwrap_or_else(|| Vec::new(env))
}

pub fn push_global_payment_id(env: &Env, order_id: &Bytes) {
    let mut ids = get_global_payment_ids(env);
    ids.push_back(order_id.clone());
    let key = DataKey::GlobalPaymentIndex;
    env.storage().persistent().set(&key, &ids);
    env.storage()
        .persistent()
        .extend_ttl(&key, TTL_THRESHOLD, TTL_LEDGERS);
}

pub fn set_global_payment_ids(env: &Env, ids: &Vec<Bytes>) {
    let key = DataKey::GlobalPaymentIndex;
    env.storage().persistent().set(&key, ids);
    env.storage()
        .persistent()
        .extend_ttl(&key, TTL_THRESHOLD, TTL_LEDGERS);
}

pub fn remove_merchant_payment_id(env: &Env, merchant: &Address, order_id: &Bytes) {
    let ids = get_merchant_payment_ids(env, merchant);
    let mut new_ids = Vec::new(env);
    for id in ids.iter() {
        if id != *order_id {
            new_ids.push_back(id);
        }
    }
    env.storage()
        .persistent()
        .set(&DataKey::MerchantPayments(merchant.clone()), &new_ids);
}

pub fn remove_payer_payment_id(env: &Env, payer: &Address, order_id: &Bytes) {
    let ids = get_payer_payment_ids(env, payer);
    let mut new_ids = Vec::new(env);
    for id in ids.iter() {
        if id != *order_id {
            new_ids.push_back(id);
        }
    }
    env.storage()
        .persistent()
        .set(&DataKey::PayerPayments(payer.clone()), &new_ids);
}

pub fn remove_global_payment_id(env: &Env, order_id: &Bytes) {
    let ids = get_global_payment_ids(env);
    let mut new_ids = Vec::new(env);
    for id in ids.iter() {
        if id != *order_id {
            new_ids.push_back(id);
        }
    }
    set_global_payment_ids(env, &new_ids);
}

// ── Refund ────────────────────────────────────────────────────────────────────

pub fn get_refund(env: &Env, refund_id: &Bytes) -> Option<RefundRecord> {
    let key = DataKey::Refund(refund_id.clone());
    let result = env.storage().persistent().get(&key);
    if result.is_some() {
        env.storage()
            .persistent()
            .extend_ttl(&key, TTL_THRESHOLD, TTL_LEDGERS);
    }
    result
}

pub fn save_refund(env: &Env, refund: &RefundRecord) {
    let key = DataKey::Refund(refund.refund_id.clone());
    env.storage().persistent().set(&key, refund);
    env.storage()
        .persistent()
        .extend_ttl(&key, TTL_THRESHOLD, TTL_LEDGERS);
}

pub fn get_all_refund_ids(env: &Env) -> Vec<Bytes> {
    let key = DataKey::AllRefunds;
    let result: Option<Vec<Bytes>> = env.storage().persistent().get(&key);
    if result.is_some() {
        env.storage()
            .persistent()
            .extend_ttl(&key, TTL_THRESHOLD, TTL_LEDGERS);
    }
    result.unwrap_or_else(|| Vec::new(env))
}

pub fn push_all_refund_id(env: &Env, refund_id: &Bytes) {
    let mut ids = get_all_refund_ids(env);
    ids.push_back(refund_id.clone());
    let key = DataKey::AllRefunds;
    env.storage().persistent().set(&key, &ids);
    env.storage()
        .persistent()
        .extend_ttl(&key, TTL_THRESHOLD, TTL_LEDGERS);
}

// ── Multisig ──────────────────────────────────────────────────────────────────

pub fn get_multisig(env: &Env, payment_id: &Bytes) -> Option<MultisigPayment> {
    let key = DataKey::Multisig(payment_id.clone());
    let result = env.storage().persistent().get(&key);
    if result.is_some() {
        env.storage()
            .persistent()
            .extend_ttl(&key, TTL_THRESHOLD, TTL_LEDGERS);
    }
    result
}

pub fn save_multisig(env: &Env, ms: &MultisigPayment) {
    let key = DataKey::Multisig(ms.payment_id.clone());
    env.storage().persistent().set(&key, ms);
    env.storage()
        .persistent()
        .extend_ttl(&key, TTL_THRESHOLD, TTL_LEDGERS);
}

// ── Whitelist ─────────────────────────────────────────────────────────────────

pub fn is_whitelist_enabled(env: &Env) -> bool {
    env.storage()
        .instance()
        .get(&DataKey::WhitelistEnabled)
        .unwrap_or(false)
}

pub fn set_whitelist_enabled(env: &Env, enabled: bool) {
    env.storage()
        .instance()
        .set(&DataKey::WhitelistEnabled, &enabled);
}

pub fn is_whitelisted(env: &Env, address: &Address) -> bool {
    env.storage()
        .persistent()
        .get(&DataKey::Whitelist(address.clone()))
        .unwrap_or(false)
}

pub fn set_whitelisted(env: &Env, address: &Address, approved: bool) {
    env.storage()
        .persistent()
        .set(&DataKey::Whitelist(address.clone()), &approved);
}

// ── Config ────────────────────────────────────────────────────────────────────

/// Default cleanup period: 90 days in seconds
pub const DEFAULT_CLEANUP_PERIOD: u64 = 7_776_000;
/// Refund window: 30 days in seconds
pub const REFUND_WINDOW: u64 = 2_592_000;
/// Default multisig expiry: 24 hours in seconds
pub const DEFAULT_MULTISIG_EXPIRY: u64 = 86_400;
/// Maximum number of signers for a multisig payment
pub const MAX_SIGNERS: u32 = 10;

pub fn get_cleanup_period(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&DataKey::CleanupPeriod)
        .unwrap_or(DEFAULT_CLEANUP_PERIOD)
}

pub fn set_cleanup_period(env: &Env, period: u64) {
    env.storage()
        .instance()
        .set(&DataKey::CleanupPeriod, &period);
}

pub fn get_default_multisig_expiry(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&DataKey::DefaultMultisigExpiry)
        .unwrap_or(DEFAULT_MULTISIG_EXPIRY)
}

pub fn set_default_multisig_expiry(env: &Env, expiry: u64) {
    env.storage()
        .instance()
        .set(&DataKey::DefaultMultisigExpiry, &expiry);
}

// ── Global stats ──────────────────────────────────────────────────────────────

pub fn get_global_stats(env: &Env) -> GlobalStats {
    env.storage()
        .instance()
        .get(&DataKey::GlobalStats)
        .unwrap_or(GlobalStats {
            total_payments: 0,
            total_volume: 0,
            total_refunds: 0,
            total_refund_volume: 0,
        })
}

pub fn save_global_stats(env: &Env, stats: &GlobalStats) {
    env.storage().instance().set(&DataKey::GlobalStats, stats);
}

pub fn increment_payment_stats(env: &Env, amount: i128) -> Result<(), PaymentError> {
    let mut stats = get_global_stats(env);
    stats.total_payments += 1;
    stats.total_volume = stats
        .total_volume
        .checked_add(amount)
        .ok_or(PaymentError::ArithmeticError)?;
    save_global_stats(env, &stats);
    Ok(())
}

pub fn increment_refund_stats(env: &Env, amount: i128) -> Result<(), PaymentError> {
    let mut stats = get_global_stats(env);
    stats.total_refunds += 1;
    stats.total_refund_volume = stats
        .total_refund_volume
        .checked_add(amount)
        .ok_or(PaymentError::ArithmeticError)?;
    save_global_stats(env, &stats);
    Ok(())
}
