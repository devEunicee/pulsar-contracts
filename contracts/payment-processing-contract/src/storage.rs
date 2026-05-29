use soroban_sdk::{Address, Bytes, Env, String, Vec};

use crate::error::PaymentError;
use crate::types::{DataKey, GlobalStats, Merchant, MultisigPayment, PaymentRecord, RefundRecord};

// ── Admin ─────────────────────────────────────────────────────────────────────

pub fn get_admin(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::Admin)
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
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
    env.storage()
        .persistent()
        .get(&DataKey::Merchant(address.clone()))
}

pub fn save_merchant(env: &Env, merchant: &Merchant) {
    env.storage()
        .persistent()
        .set(&DataKey::Merchant(merchant.address.clone()), merchant);
}

// ── Payment ───────────────────────────────────────────────────────────────────

pub fn get_payment(env: &Env, order_id: &Bytes) -> Option<PaymentRecord> {
    env.storage()
        .persistent()
        .get(&DataKey::Payment(order_id.clone()))
}

pub fn save_payment(env: &Env, record: &PaymentRecord) {
    env.storage()
        .persistent()
        .set(&DataKey::Payment(record.order_id.clone()), record);
}

pub fn remove_payment(env: &Env, order_id: &Bytes) {
    env.storage()
        .persistent()
        .remove(&DataKey::Payment(order_id.clone()));
}

// ── Payment index lists ───────────────────────────────────────────────────────

pub fn get_merchant_payment_ids(env: &Env, merchant: &Address) -> Vec<Bytes> {
    env.storage()
        .persistent()
        .get(&DataKey::MerchantPayments(merchant.clone()))
        .unwrap_or_else(|| Vec::new(env))
}

pub fn push_merchant_payment_id(env: &Env, merchant: &Address, order_id: &Bytes) {
    let mut ids = get_merchant_payment_ids(env, merchant);
    ids.push_back(order_id.clone());
    env.storage()
        .persistent()
        .set(&DataKey::MerchantPayments(merchant.clone()), &ids);
}

pub fn get_payer_payment_ids(env: &Env, payer: &Address) -> Vec<Bytes> {
    env.storage()
        .persistent()
        .get(&DataKey::PayerPayments(payer.clone()))
        .unwrap_or_else(|| Vec::new(env))
}

pub fn push_payer_payment_id(env: &Env, payer: &Address, order_id: &Bytes) {
    let mut ids = get_payer_payment_ids(env, payer);
    ids.push_back(order_id.clone());
    env.storage()
        .persistent()
        .set(&DataKey::PayerPayments(payer.clone()), &ids);
}

pub fn get_global_payment_ids(env: &Env) -> Vec<Bytes> {
    env.storage()
        .persistent()
        .get(&DataKey::GlobalPaymentIndex)
        .unwrap_or_else(|| Vec::new(env))
}

pub fn push_global_payment_id(env: &Env, order_id: &Bytes) {
    let mut ids = get_global_payment_ids(env);
    ids.push_back(order_id.clone());
    env.storage()
        .persistent()
        .set(&DataKey::GlobalPaymentIndex, &ids);
}

pub fn set_global_payment_ids(env: &Env, ids: &Vec<Bytes>) {
    env.storage()
        .persistent()
        .set(&DataKey::GlobalPaymentIndex, ids);
}

// ── Refund ────────────────────────────────────────────────────────────────────

pub fn get_refund(env: &Env, refund_id: &Bytes) -> Option<RefundRecord> {
    env.storage()
        .persistent()
        .get(&DataKey::Refund(refund_id.clone()))
}

pub fn save_refund(env: &Env, refund: &RefundRecord) {
    env.storage()
        .persistent()
        .set(&DataKey::Refund(refund.refund_id.clone()), refund);
}

pub fn get_all_refund_ids(env: &Env) -> Vec<Bytes> {
    env.storage()
        .persistent()
        .get(&DataKey::AllRefunds)
        .unwrap_or_else(|| Vec::new(env))
}

pub fn push_all_refund_id(env: &Env, refund_id: &Bytes) {
    let mut ids = get_all_refund_ids(env);
    ids.push_back(refund_id.clone());
    env.storage()
        .persistent()
        .set(&DataKey::AllRefunds, &ids);
}

// ── Multisig ──────────────────────────────────────────────────────────────────

pub fn get_multisig(env: &Env, payment_id: &Bytes) -> Option<MultisigPayment> {
    env.storage()
        .persistent()
        .get(&DataKey::Multisig(payment_id.clone()))
}

pub fn save_multisig(env: &Env, ms: &MultisigPayment) {
    env.storage()
        .persistent()
        .set(&DataKey::Multisig(ms.payment_id.clone()), ms);
}

// ── Config ────────────────────────────────────────────────────────────────────

/// Default cleanup period: 90 days in seconds
pub const DEFAULT_CLEANUP_PERIOD: u64 = 7_776_000;
/// Refund window: 30 days in seconds
pub const REFUND_WINDOW: u64 = 2_592_000;
/// Default multisig expiry: 24 hours in seconds
pub const DEFAULT_MULTISIG_EXPIRY: u64 = 86_400;

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

pub fn increment_payment_stats(env: &Env, amount: i128) {
    let mut stats = get_global_stats(env);
    stats.total_payments += 1;
    stats.total_volume += amount;
    save_global_stats(env, &stats);
}

pub fn increment_refund_stats(env: &Env, amount: i128) -> Result<(), PaymentError> {
    let mut stats = get_global_stats(env);
    stats.total_refunds += 1;
    stats.total_refund_volume += amount;
    save_global_stats(env, &stats);
    Ok(())
}
