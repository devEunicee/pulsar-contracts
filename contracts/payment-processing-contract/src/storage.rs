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

const CHUNK_SIZE: u32 = 100;

pub fn get_merchant_payment_ids(env: &Env, merchant: &Address) -> Vec<Bytes> {
    let count = env
        .storage()
        .persistent()
        .get(&DataKey::MerchantPaymentCount(merchant.clone()))
        .unwrap_or(0u32);
    let mut all_ids = Vec::new(env);
    if count == 0 {
        return all_ids;
    }
    let num_chunks = (count + CHUNK_SIZE - 1) / CHUNK_SIZE;
    for i in 0..num_chunks {
        let chunk: Vec<Bytes> = env
            .storage()
            .persistent()
            .get(&DataKey::MerchantPaymentChunk(merchant.clone(), i))
            .unwrap_or_else(|| Vec::new(env));
        for id in chunk.iter() {
            all_ids.push_back(id);
        }
    }
    all_ids
}

pub fn push_merchant_payment_id(env: &Env, merchant: &Address, order_id: &Bytes) {
    let count = env
        .storage()
        .persistent()
        .get(&DataKey::MerchantPaymentCount(merchant.clone()))
        .unwrap_or(0u32);
    let chunk_index = count / CHUNK_SIZE;
    let mut chunk: Vec<Bytes> = env
        .storage()
        .persistent()
        .get(&DataKey::MerchantPaymentChunk(merchant.clone(), chunk_index))
        .unwrap_or_else(|| Vec::new(env));
    chunk.push_back(order_id.clone());
    env.storage()
        .persistent()
        .set(
            &DataKey::MerchantPaymentChunk(merchant.clone(), chunk_index),
            &chunk,
        );
    env.storage()
        .persistent()
        .set(&DataKey::MerchantPaymentCount(merchant.clone()), &(count + 1));
}

pub fn get_payer_payment_ids(env: &Env, payer: &Address) -> Vec<Bytes> {
    let count = env
        .storage()
        .persistent()
        .get(&DataKey::PayerPaymentCount(payer.clone()))
        .unwrap_or(0u32);
    let mut all_ids = Vec::new(env);
    if count == 0 {
        return all_ids;
    }
    let num_chunks = (count + CHUNK_SIZE - 1) / CHUNK_SIZE;
    for i in 0..num_chunks {
        let chunk: Vec<Bytes> = env
            .storage()
            .persistent()
            .get(&DataKey::PayerPaymentChunk(payer.clone(), i))
            .unwrap_or_else(|| Vec::new(env));
        for id in chunk.iter() {
            all_ids.push_back(id);
        }
    }
    all_ids
}

pub fn push_payer_payment_id(env: &Env, payer: &Address, order_id: &Bytes) {
    let count = env
        .storage()
        .persistent()
        .get(&DataKey::PayerPaymentCount(payer.clone()))
        .unwrap_or(0u32);
    let chunk_index = count / CHUNK_SIZE;
    let mut chunk: Vec<Bytes> = env
        .storage()
        .persistent()
        .get(&DataKey::PayerPaymentChunk(payer.clone(), chunk_index))
        .unwrap_or_else(|| Vec::new(env));
    chunk.push_back(order_id.clone());
    env.storage()
        .persistent()
        .set(
            &DataKey::PayerPaymentChunk(payer.clone(), chunk_index),
            &chunk,
        );
    env.storage()
        .persistent()
        .set(&DataKey::PayerPaymentCount(payer.clone()), &(count + 1));
}

pub fn get_global_payment_ids(env: &Env) -> Vec<Bytes> {
    let count = env
        .storage()
        .persistent()
        .get(&DataKey::GlobalPaymentCount)
        .unwrap_or(0u32);
    let mut all_ids = Vec::new(env);
    if count == 0 {
        return all_ids;
    }
    let num_chunks = (count + CHUNK_SIZE - 1) / CHUNK_SIZE;
    for i in 0..num_chunks {
        let chunk: Vec<Bytes> = env
            .storage()
            .persistent()
            .get(&DataKey::GlobalPaymentChunk(i))
            .unwrap_or_else(|| Vec::new(env));
        for id in chunk.iter() {
            all_ids.push_back(id);
        }
    }
    all_ids
}

pub fn push_global_payment_id(env: &Env, order_id: &Bytes) {
    let count = env
        .storage()
        .persistent()
        .get(&DataKey::GlobalPaymentCount)
        .unwrap_or(0u32);
    let chunk_index = count / CHUNK_SIZE;
    let mut chunk: Vec<Bytes> = env
        .storage()
        .persistent()
        .get(&DataKey::GlobalPaymentChunk(chunk_index))
        .unwrap_or_else(|| Vec::new(env));
    chunk.push_back(order_id.clone());
    env.storage()
        .persistent()
        .set(&DataKey::GlobalPaymentChunk(chunk_index), &chunk);
    env.storage()
        .persistent()
        .set(&DataKey::GlobalPaymentCount, &(count + 1));
}

pub fn set_global_payment_ids(env: &Env, ids: &Vec<Bytes>) {
    let count = ids.len();
    env.storage()
        .persistent()
        .set(&DataKey::GlobalPaymentCount, &count);
    let num_chunks = (count + CHUNK_SIZE - 1) / CHUNK_SIZE;
    for i in 0..num_chunks {
        let mut chunk = Vec::new(env);
        let start = i * CHUNK_SIZE;
        let end = (start + CHUNK_SIZE).min(count);
        for j in start..end {
            chunk.push_back(ids.get(j).unwrap());
        }
        env.storage()
            .persistent()
            .set(&DataKey::GlobalPaymentChunk(i), &chunk);
    }
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
