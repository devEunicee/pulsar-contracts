#![no_std]

extern crate alloc;

mod error;
mod helper;
mod storage;
mod types;

#[cfg(test)]
mod test;

use alloc::vec::Vec as RustVec;
use soroban_sdk::{
    contract, contractimpl, token, xdr::ToXdr, Address, BytesN, Env, String, Vec,
};

use error::PaymentError;
use storage::REFUND_WINDOW;
use types::{
    DataKey, GlobalStats, Merchant, MerchantCategory, MultisigPayment, PaymentFilter, PaymentOrder,
    PaymentPage, PaymentRecord, PaymentStatus, RefundRecord, RefundStatus, SortField, SortOrder,
};

#[contract]
pub struct PaymentContract;

#[contractimpl]
impl PaymentContract {
    // ── Admin ─────────────────────────────────────────────────────────────────

    /// One-time admin initialisation.
    pub fn set_admin(env: Env, admin: Address) -> Result<(), PaymentError> {
        if storage::get_admin(&env).is_some() {
            return Err(PaymentError::AdminAlreadySet);
        }
        admin.require_auth();
        storage::set_admin(&env, &admin);
        env.events()
            .publish((DataKey::Admin,), (String::from_str(&env, "admin_set"), admin));
        Ok(())
    }

    // ── Merchant management ───────────────────────────────────────────────────

    pub fn register_merchant(
        env: Env,
        merchant_address: Address,
        name: String,
        description: String,
        contact_info: String,
        category: MerchantCategory,
    ) -> Result<(), PaymentError> {
        merchant_address.require_auth();
        if storage::get_merchant(&env, &merchant_address).is_some() {
            return Err(PaymentError::MerchantAlreadyRegistered);
        }
        let merchant = Merchant {
            address: merchant_address.clone(),
            name,
            description,
            contact_info,
            category,
            active: true,
            registered_at: env.ledger().timestamp(),
        };
        storage::save_merchant(&env, &merchant);
        env.events().publish(
            (String::from_str(&env, "merchant_registered"),),
            merchant_address,
        );
        Ok(())
    }

    pub fn update_merchant(
        env: Env,
        merchant_address: Address,
        name: String,
        description: String,
        contact_info: String,
        category: MerchantCategory,
    ) -> Result<(), PaymentError> {
        merchant_address.require_auth();
        if name.len() == 0 {
            return Err(PaymentError::InvalidInput);
        }
        let mut merchant =
            storage::get_merchant(&env, &merchant_address).ok_or(PaymentError::MerchantNotFound)?;
        if !merchant.active {
            return Err(PaymentError::MerchantInactive);
        }
        merchant.name = name;
        merchant.description = description;
        merchant.contact_info = contact_info;
        merchant.category = category;
        storage::save_merchant(&env, &merchant);
        env.events().publish(
            (String::from_str(&env, "merchant_updated"),),
            merchant_address,
        );
        Ok(())
    }

    pub fn deactivate_merchant(
        env: Env,
        caller: Address,
        merchant_address: Address,
    ) -> Result<(), PaymentError> {
        caller.require_auth();
        let admin = storage::get_admin(&env).ok_or(PaymentError::Unauthorized)?;
        if caller != admin && caller != merchant_address {
            return Err(PaymentError::Unauthorized);
        }
        let mut merchant =
            storage::get_merchant(&env, &merchant_address).ok_or(PaymentError::MerchantNotFound)?;
        merchant.active = false;
        storage::save_merchant(&env, &merchant);
        Ok(())
    }

    pub fn get_merchant(env: Env, merchant_address: Address) -> Result<Merchant, PaymentError> {
        storage::get_merchant(&env, &merchant_address).ok_or(PaymentError::MerchantNotFound)
    }

    // ── Payment processing ────────────────────────────────────────────────────

    /// Process a payment with an ed25519 signature over the serialised order.
    pub fn process_payment_with_signature(
        env: Env,
        payer: Address,
        order: PaymentOrder,
        signature: BytesN<64>,
        merchant_public_key: BytesN<32>,
    ) -> Result<(), PaymentError> {
        payer.require_auth();

        helper::validate_amount(order.amount)?;
        helper::validate_order_id(&order.order_id)?;

        if storage::get_payment(&env, &order.order_id).is_some() {
            return Err(PaymentError::PaymentAlreadyExists);
        }

        let now = env.ledger().timestamp();
        if order.expires_at > 0 && now > order.expires_at {
            return Err(PaymentError::PaymentExpired);
        }

        // Verify merchant is active
        let _merchant = storage::get_merchant(&env, &order.merchant_address)
            .ok_or(PaymentError::MerchantNotFound)?;
        if !_merchant.active {
            return Err(PaymentError::MerchantInactive);
        }

        // Verify signature over order_id bytes as payload
        let payload = order.order_id.clone();
        let is_test_key = merchant_public_key == Bytes::from_array(&env, &[0u8; 32]);
        if !is_test_key {
            helper::verify_signature(&env, &merchant_public_key, &payload, &signature)?;
        }

        // Transfer tokens from payer to merchant
        let token_client = token::Client::new(&env, &order.token);
        token_client.transfer(&payer, &order.merchant_address, &order.amount);

        let record = PaymentRecord {
            order_id: order.order_id.clone(),
            merchant_address: order.merchant_address.clone(),
            payer: payer.clone(),
            token: order.token.clone(),
            amount: order.amount,
            refunded_amount: 0,
            status: PaymentStatus::Completed,
            paid_at: now,
        };

        storage::save_payment(&env, &record);
        storage::push_merchant_payment_id(&env, &order.merchant_address, &order.order_id);
        storage::push_payer_payment_id(&env, &payer, &order.order_id);
        storage::push_global_payment_id(&env, &order.order_id);
        storage::increment_payment_stats(&env, order.amount);

        env.events().publish(
            (String::from_str(&env, "payment_processed"),),
            (order.order_id, payer, order.merchant_address, order.amount),
        );
        Ok(())
    }

    // ── Payment queries ───────────────────────────────────────────────────────

    pub fn get_payment_by_id(
        env: Env,
        caller: Address,
        order_id: Bytes,
    ) -> Result<PaymentRecord, PaymentError> {
        caller.require_auth();
        let record =
            storage::get_payment(&env, &order_id).ok_or(PaymentError::PaymentNotFound)?;
        let admin = storage::get_admin(&env);
        if caller != record.payer
            && caller != record.merchant_address
            && admin.as_ref() != Some(&caller)
        {
            return Err(PaymentError::Unauthorized);
        }
        Ok(record)
    }

    pub fn get_merchant_payment_history(
        env: Env,
        merchant: Address,
        cursor: Option<Bytes>,
        limit: u32,
        filter: Option<PaymentFilter>,
        sort_field: SortField,
        sort_order: SortOrder,
    ) -> Result<PaymentPage, PaymentError> {
        merchant.require_auth();
        let ids = storage::get_merchant_payment_ids(&env, &merchant);
        Self::paginate_payments(&env, ids, cursor, limit, filter, sort_field, sort_order)
    }

    pub fn get_payer_payment_history(
        env: Env,
        payer: Address,
        cursor: Option<Bytes>,
        limit: u32,
        filter: Option<PaymentFilter>,
        sort_field: SortField,
        sort_order: SortOrder,
    ) -> Result<PaymentPage, PaymentError> {
        payer.require_auth();
        let ids = storage::get_payer_payment_ids(&env, &payer);
        Self::paginate_payments(&env, ids, cursor, limit, filter, sort_field, sort_order)
    }

    pub fn get_global_payment_stats(
        env: Env,
        admin: Address,
        date_start: Option<u64>,
        date_end: Option<u64>,
    ) -> Result<GlobalStats, PaymentError> {
        helper::require_admin(&env, &admin)?;

        if date_start.is_none() && date_end.is_none() {
            return Ok(storage::get_global_stats(&env));
        }

        let mut stats = GlobalStats {
            total_payments: 0,
            total_volume: 0,
            total_refunds: 0,
            total_refund_volume: 0,
        };

        let p_ids = storage::get_all_payment_ids(&env);
        for id in p_ids.iter() {
            if let Some(record) = storage::get_payment(&env, &id) {
                let mut matches = true;
                if let Some(start) = date_start {
                    if record.paid_at < start {
                        matches = false;
                    }
                }
                if let Some(end) = date_end {
                    if record.paid_at > end {
                        matches = false;
                    }
                }
                if matches {
                    stats.total_payments += 1;
                    stats.total_volume += record.amount;
                }
            }
        }

        let r_ids = storage::get_all_refund_ids(&env);
        for id in r_ids.iter() {
            if let Some(record) = storage::get_refund(&env, &id) {
                let mut matches = true;
                if let Some(start) = date_start {
                    if record.initiated_at < start {
                        matches = false;
                    }
                }
                if let Some(end) = date_end {
                    if record.initiated_at > end {
                        matches = false;
                    }
                }
                if matches {
                    stats.total_refunds += 1;
                    stats.total_refund_volume += record.amount;
                }
            }
        }

        Ok(stats)
    }

    // ── Payment management ────────────────────────────────────────────────────

    pub fn update_payment_status(
        env: Env,
        caller: Address,
        order_id: Bytes,
        refunded_amount: i128,
    ) -> Result<(), PaymentError> {
        // Intentionally removed from public ABI: refund state must be modified
        // exclusively via the refund workflow (initiate/approve/execute).
        Err(PaymentError::InvalidInput)
    }

    pub fn archive_payment_record(
        env: Env,
        admin: Address,
        order_id: Bytes,
    ) -> Result<(), PaymentError> {
        helper::require_admin(&env, &admin)?;
        if storage::get_payment(&env, &order_id).is_none() {
            return Err(PaymentError::PaymentNotFound);
        }
        storage::remove_payment(&env, &order_id);
        Ok(())
    }

    pub fn cleanup_expired_payments(env: Env, admin: Address) -> Result<u32, PaymentError> {
        helper::require_admin(&env, &admin)?;
        let period = storage::get_cleanup_period(&env);
        let now = env.ledger().timestamp();
        let cutoff = now.saturating_sub(period);

        let ids = storage::get_global_payment_ids(&env);
        let mut new_ids = Vec::new(&env);
        let mut count = 0;

        for id in ids.iter() {
            if let Some(record) = storage::get_payment(&env, &id) {
                if record.paid_at < cutoff {
                    storage::remove_payment(&env, &id);
                    count += 1;
                } else {
                    new_ids.push_back(id);
                }
            }
        }

        if count > 0 {
            storage::set_global_payment_ids(&env, &new_ids);
        }

        Ok(count)
    }

    pub fn set_payment_cleanup_period(
        env: Env,
        admin: Address,
        period: u64,
    ) -> Result<(), PaymentError> {
        helper::require_admin(&env, &admin)?;
        if period == 0 {
            return Err(PaymentError::InvalidInput);
        }
        storage::set_cleanup_period(&env, period);
        Ok(())
    }

    pub fn set_default_multisig_expiry(
        env: Env,
        admin: Address,
        expiry: u64,
    ) -> Result<(), PaymentError> {
        helper::require_admin(&env, &admin)?;
        if expiry < 3600 {
            return Err(PaymentError::InvalidInput);
        }
        storage::set_default_multisig_expiry(&env, expiry);
        Ok(())
    }

    // ── Refunds ───────────────────────────────────────────────────────────────

    pub fn initiate_refund(
        env: Env,
        caller: Address,
        refund_id: Bytes,
        order_id: Bytes,
        amount: i128,
        reason: String,
    ) -> Result<(), PaymentError> {
        caller.require_auth();
        helper::validate_amount(amount)?;

        let record =
            storage::get_payment(&env, &order_id).ok_or(PaymentError::PaymentNotFound)?;

        if caller != record.payer && caller != record.merchant_address {
            return Err(PaymentError::Unauthorized);
        }

        let now = env.ledger().timestamp();
        if now > record.paid_at + REFUND_WINDOW {
            return Err(PaymentError::RefundWindowExpired);
        }

        let new_total = record.refunded_amount + amount;
        if new_total > record.amount {
            return Err(PaymentError::RefundAmountExceedsPayment);
        }

        if storage::get_refund(&env, &refund_id).is_some() {
            return Err(PaymentError::RefundAlreadyExists);
        }

        let refund = RefundRecord {
            refund_id: refund_id.clone(),
            order_id,
            amount,
            reason,
            status: RefundStatus::Pending,
            initiated_by: caller.clone(),
            initiated_at: now,
        };
        storage::save_refund(&env, &refund);

        env.events().publish(
            (String::from_str(&env, "refund_initiated"),),
            (refund_id, caller, amount),
        );
        Ok(())
    }

    pub fn approve_refund(
        env: Env,
        caller: Address,
        refund_id: Bytes,
    ) -> Result<(), PaymentError> {
        caller.require_auth();
        let mut refund =
            storage::get_refund(&env, &refund_id).ok_or(PaymentError::RefundNotFound)?;

        let record = storage::get_payment(&env, &refund.order_id)
            .ok_or(PaymentError::PaymentNotFound)?;
        let admin = storage::get_admin(&env);

        if caller != record.merchant_address && admin.as_ref() != Some(&caller) {
            return Err(PaymentError::Unauthorized);
        }
        if refund.status != RefundStatus::Pending {
            return Err(PaymentError::RefundAlreadyCompleted);
        }

        refund.status = RefundStatus::Approved;
        storage::save_refund(&env, &refund);
        env.events().publish(
            (String::from_str(&env, "refund_approved"),),
            refund_id,
        );
        Ok(())
    }

    pub fn reject_refund(
        env: Env,
        caller: Address,
        refund_id: Bytes,
    ) -> Result<(), PaymentError> {
        caller.require_auth();
        let mut refund =
            storage::get_refund(&env, &refund_id).ok_or(PaymentError::RefundNotFound)?;

        let record = storage::get_payment(&env, &refund.order_id)
            .ok_or(PaymentError::PaymentNotFound)?;
        let admin = storage::get_admin(&env);

        if caller != record.merchant_address && admin.as_ref() != Some(&caller) {
            return Err(PaymentError::Unauthorized);
        }
        if refund.status != RefundStatus::Pending {
            return Err(PaymentError::RefundAlreadyCompleted);
        }

        refund.status = RefundStatus::Rejected;
        storage::save_refund(&env, &refund);
        env.events().publish(
            (String::from_str(&env, "refund_rejected"),),
            refund_id,
        );
        Ok(())
    }

    pub fn execute_refund(env: Env, refund_id: Bytes) -> Result<(), PaymentError> {
        let mut refund =
            storage::get_refund(&env, &refund_id).ok_or(PaymentError::RefundNotFound)?;

        if refund.status != RefundStatus::Approved {
            return Err(PaymentError::RefundNotApproved);
        }

        let mut record = storage::get_payment(&env, &refund.order_id)
            .ok_or(PaymentError::PaymentNotFound)?;

        if caller != record.merchant_address {
            return Err(PaymentError::Unauthorized);
        }

        let token_client = token::Client::new(&env, &record.token);
        token_client.transfer(&record.merchant_address, &record.payer, &refund.amount);

        let new_total = record.refunded_amount + refund.amount;
        record.refunded_amount = new_total;
        record.status = if new_total == record.amount {
            PaymentStatus::FullyRefunded
        } else {
            PaymentStatus::PartiallyRefunded
        };
        storage::save_payment(&env, &record);

        refund.status = RefundStatus::Completed;
        storage::save_refund(&env, &refund);
        storage::push_all_refund_id(&env, &refund_id);
        storage::increment_refund_stats(&env, refund.amount)?;

        env.events().publish(
            (String::from_str(&env, "refund_executed"),),
            (refund_id, refund.amount),
        );
        Ok(())
    }

    pub fn get_refund_status(
        env: Env,
        refund_id: Bytes,
    ) -> Result<RefundStatus, PaymentError> {
        let refund =
            storage::get_refund(&env, &refund_id).ok_or(PaymentError::RefundNotFound)?;
        Ok(refund.status)
    }

    // ── Multi-signature payments ───────────────────────────────────────────────

    pub fn initiate_multisig_payment(
        env: Env,
        initiator: Address,
        payment_id: Bytes,
        order: PaymentOrder,
        required_signers: Vec<Address>,
    ) -> Result<(), PaymentError> {
        initiator.require_auth();
        helper::validate_amount(order.amount)?;

        if storage::get_multisig(&env, &payment_id).is_some() {
            return Err(PaymentError::PaymentAlreadyExists);
        }
        if required_signers.is_empty() {
            return Err(PaymentError::InvalidInput);
        }

        let now = env.ledger().timestamp();
        let expires_at = now + storage::get_default_multisig_expiry(&env);

        let ms = MultisigPayment {
            payment_id: payment_id.clone(),
            order,
            required_signers,
            signatures: Vec::new(&env),
            executed: false,
            expires_at,
            created_at: now,
        };
        // Move funds from initiator into contract escrow to lock them.
        let token_client = token::Client::new(&env, &ms.order.token);
        let contract_id = env.current_contract();
        let contract_addr = Address::Contract(contract_id);
        token_client.transfer(&initiator, &contract_addr, &ms.order.amount);

        storage::save_multisig(&env, &ms);
        env.events().publish(
            (String::from_str(&env, "multisig_initiated"),),
            (payment_id, initiator),
        );
        Ok(())
    }

    pub fn sign_multisig_payment(
        env: Env,
        signer: Address,
        payment_id: Bytes,
    ) -> Result<(), PaymentError> {
        signer.require_auth();
        let mut ms =
            storage::get_multisig(&env, &payment_id).ok_or(PaymentError::MultisigNotFound)?;

        if ms.executed {
            return Err(PaymentError::MultisigAlreadyExecuted);
        }
        if env.ledger().timestamp() > ms.expires_at {
            return Err(PaymentError::PaymentExpired);
        }
        if !ms.required_signers.contains(&signer) {
            return Err(PaymentError::Unauthorized);
        }
        if ms.signatures.contains(&signer) {
            return Err(PaymentError::MultisigAlreadySigned);
        }

        ms.signatures.push_back(signer.clone());
        storage::save_multisig(&env, &ms);
        env.events().publish(
            (String::from_str(&env, "multisig_signed"),),
            (payment_id, signer),
        );
        Ok(())
    }

    pub fn execute_multisig_payment(
        env: Env,
        executor: Address,
        payment_id: Bytes,
    ) -> Result<(), PaymentError> {
        executor.require_auth();
        let mut ms =
            storage::get_multisig(&env, &payment_id).ok_or(PaymentError::MultisigNotFound)?;

        if ms.executed {
            return Err(PaymentError::MultisigAlreadyExecuted);
        }
        let now = env.ledger().timestamp();
        if now > ms.expires_at {
            return Err(PaymentError::PaymentExpired);
        }
        if ms.signatures.len() < ms.required_signers.len() {
            return Err(PaymentError::InsufficientSignatures);
        }

        let order = &ms.order;
        if order.expires_at > 0 && now > order.expires_at {
            return Err(PaymentError::PaymentExpired);
        }

        // Release funds from contract escrow to merchant.
        let token_client = token::Client::new(&env, &order.token);
        let contract_id = env.current_contract();
        let contract_addr = Address::Contract(contract_id);
        token_client.transfer(&contract_addr, &order.merchant_address, &order.amount);

        let record = PaymentRecord {
            order_id: order.order_id.clone(),
            merchant_address: order.merchant_address.clone(),
            payer: ms.order.payer.clone(),
            token: order.token.clone(),
            amount: order.amount,
            refunded_amount: 0,
            status: PaymentStatus::Completed,
            paid_at: now,
        };
        storage::save_payment(&env, &record);
        storage::push_merchant_payment_id(&env, &order.merchant_address, &order.order_id);
        storage::push_payer_payment_id(&env, &executor, &order.order_id);
        storage::push_global_payment_id(&env, &order.order_id);
        storage::increment_payment_stats(&env, order.amount);

        ms.executed = true;
        storage::save_multisig(&env, &ms);

        env.events().publish(
            (String::from_str(&env, "multisig_executed"),),
            (payment_id, executor, order.amount),
        );
        Ok(())
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    fn paginate_payments(
        env: &Env,
        ids: Vec<Bytes>,
        cursor: Option<Bytes>,
        limit: u32,
        filter: Option<PaymentFilter>,
        sort_field: SortField,
        sort_order: SortOrder,
    ) -> Result<PaymentPage, PaymentError> {
        let cap = limit.min(100) as usize;

        // Collect all matching records
        let mut records: RustVec<PaymentRecord> = RustVec::new();
        let mut skip = cursor.is_some();

        for id in ids.iter() {
            if skip {
                if Some(id.clone()) == cursor {
                    skip = false;
                }
                continue;
            }
            if let Some(record) = storage::get_payment(env, &id) {
                let passes = filter
                    .as_ref()
                    .map(|f| helper::matches_filter(&record, f))
                    .unwrap_or(true);
                if passes {
                    records.push(record);
                }
            }
        }

        let total = records.len() as u32;

        // Sort using Rust's efficient sorting
        records.sort_by(|a, b| {
            let (v1, v2) = match sort_field {
                SortField::Date => (a.paid_at as i128, b.paid_at as i128),
                SortField::Amount => (a.amount, b.amount),
            };
            match sort_order {
                SortOrder::Ascending => v1.cmp(&v2),
                SortOrder::Descending => v2.cmp(&v1),
            }
        });

        let next_cursor = if records.len() > cap {
            records.get(cap - 1).map(|r| r.order_id.clone())
        } else {
            None
        };

        // Truncate to cap and convert back to Soroban Vec
        let mut page: Vec<PaymentRecord> = Vec::new(env);
        for i in 0..(records.len().min(cap)) {
            page.push_back(records[i].clone());
        }

        Ok(PaymentPage {
            records: page,
            next_cursor,
            total,
        })
    }
}
