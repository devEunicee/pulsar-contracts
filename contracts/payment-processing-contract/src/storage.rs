use soroban_sdk::{Address, Env, String, Vec};

use crate::error::PaymentError;
use crate::types::{DataKey, GlobalStats, Merchant, MultisigPayment, PaymentRecord, RefundRecord};

// ── Admin ─────────────────────────────────────────────────────────────────────

pub fn get_admin(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::Admin)
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
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

pub fn get_payment(env: &Env, order_id: &String) -> Option<PaymentRecord> {
    env.storage()
        .persistent()
        .get(&DataKey::Payment(order_id.clone()))
}

pub fn save_payment(env: &Env, record: &PaymentRecord) {
    env.storage()
        .persistent()
        .set(&DataKey::Payment(record.order_id.clone()), record);
}

pub fn remove_payment(env: &Env, order_id: &String) {
    env.storage()
        .persistent()
        .remove(&DataKey::Payment(order_id.clone()));
}

// ── Payment index lists ───────────────────────────────────────────────────────

pub fn get_merchant_payment_ids(env: &Env, merchant: &Address) -> Vec<String> {
    env.storage()
        .persistent()
        .get(&DataKey::MerchantPayments(merchant.clone()))
        .unwrap_or_else(|| Vec::new(env))
}

pub fn push_merchant_payment_id(env: &Env, merchant: &Address, order_id: &String) {
    let mut ids = get_merchant_payment_ids(env, merchant);
    ids.push_back(order_id.clone());
    env.storage()
        .persistent()
        .set(&DataKey::MerchantPayments(merchant.clone()), &ids);
}

pub fn get_payer_payment_ids(env: &Env, payer: &Address) -> Vec<String> {
    env.storage()
        .persistent()
        .get(&DataKey::PayerPayments(payer.clone()))
        .unwrap_or_else(|| Vec::new(env))
}

pub fn push_payer_payment_id(env: &Env, payer: &Address, order_id: &String) {
    let mut ids = get_payer_payment_ids(env, payer);
    ids.push_back(order_id.clone());
    env.storage()
        .persistent()
        .set(&DataKey::PayerPayments(payer.clone()), &ids);
}

pub fn get_all_payment_ids(env: &Env) -> Vec<String> {
    env.storage()
        .persistent()
        .get(&DataKey::AllPayments)
        .unwrap_or_else(|| Vec::new(env))
}

pub fn push_all_payment_id(env: &Env, order_id: &String) {
    let mut ids = get_all_payment_ids(env);
    ids.push_back(order_id.clone());
    env.storage()
        .persistent()
        .set(&DataKey::AllPayments, &ids);
}

// ── Refund ────────────────────────────────────────────────────────────────────

pub fn get_refund(env: &Env, refund_id: &String) -> Option<RefundRecord> {
    env.storage()
        .persistent()
        .get(&DataKey::Refund(refund_id.clone()))
}

pub fn save_refund(env: &Env, refund: &RefundRecord) {
    env.storage()
        .persistent()
        .set(&DataKey::Refund(refund.refund_id.clone()), refund);
}

pub fn get_all_refund_ids(env: &Env) -> Vec<String> {
    env.storage()
        .persistent()
        .get(&DataKey::AllRefunds)
        .unwrap_or_else(|| Vec::new(env))
}

pub fn push_all_refund_id(env: &Env, refund_id: &String) {
    let mut ids = get_all_refund_ids(env);
    ids.push_back(refund_id.clone());
    env.storage()
        .persistent()
        .set(&DataKey::AllRefunds, &ids);
}

// ── Multisig ──────────────────────────────────────────────────────────────────

pub fn get_multisig(env: &Env, payment_id: &String) -> Option<MultisigPayment> {
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
