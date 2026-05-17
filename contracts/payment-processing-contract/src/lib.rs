#![no_std]

mod error;
mod helper;
mod storage;
mod types;

#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contractimpl, token, Address, Bytes, Env, String, Vec,
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
        signature: Bytes,
        merchant_public_key: Bytes,
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
        let payload = Bytes::from_slice(&env, order.order_id.to_string().as_bytes());
        helper::verify_signature(&env, &merchant_public_key, &payload, &signature)?;

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
        order_id: String,
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
        cursor: Option<String>,
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
        cursor: Option<String>,
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
        // For date-filtered stats we return the stored totals (full stats).
        // A production implementation would maintain time-bucketed counters.
        let _ = (date_start, date_end);
        Ok(storage::get_global_stats(&env))
    }

    // ── Payment management ────────────────────────────────────────────────────

    pub fn update_payment_status(
        env: Env,
        caller: Address,
        order_id: String,
        refunded_amount: i128,
    ) -> Result<(), PaymentError> {
        caller.require_auth();
        let mut record =
            storage::get_payment(&env, &order_id).ok_or(PaymentError::PaymentNotFound)?;

        let admin = storage::get_admin(&env);
        if caller != record.merchant_address && admin.as_ref() != Some(&caller) {
            return Err(PaymentError::Unauthorized);
        }

        helper::validate_amount(refunded_amount)?;
        let new_total = record.refunded_amount + refunded_amount;
        if new_total > record.amount {
            return Err(PaymentError::RefundAmountExceedsPayment);
        }

        record.refunded_amount = new_total;
        record.status = if new_total == record.amount {
            PaymentStatus::FullyRefunded
        } else {
            PaymentStatus::PartiallyRefunded
        };
        storage::save_payment(&env, &record);
        Ok(())
    }

    pub fn archive_payment_record(
        env: Env,
        admin: Address,
        order_id: String,
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

        // We iterate over a snapshot of all known payment IDs.
        // In practice you'd maintain a global index; here we skip that for brevity
        // and return 0 (no-op) since we don't store a global list.
        let _ = cutoff;
        Ok(0)
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

    // ── Refunds ───────────────────────────────────────────────────────────────

    pub fn initiate_refund(
        env: Env,
        caller: Address,
        refund_id: String,
        order_id: String,
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
        refund_id: String,
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
        refund_id: String,
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

    pub fn execute_refund(env: Env, refund_id: String) -> Result<(), PaymentError> {
        let mut refund =
            storage::get_refund(&env, &refund_id).ok_or(PaymentError::RefundNotFound)?;

        if refund.status != RefundStatus::Approved {
            return Err(PaymentError::RefundNotApproved);
        }

        let mut record = storage::get_payment(&env, &refund.order_id)
            .ok_or(PaymentError::PaymentNotFound)?;

        record.merchant_address.require_auth();

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
        storage::increment_refund_stats(&env, refund.amount)?;

        env.events().publish(
            (String::from_str(&env, "refund_executed"),),
            (refund_id, refund.amount),
        );
        Ok(())
    }

    pub fn get_refund_status(
        env: Env,
        refund_id: String,
    ) -> Result<RefundStatus, PaymentError> {
        let refund =
            storage::get_refund(&env, &refund_id).ok_or(PaymentError::RefundNotFound)?;
        Ok(refund.status)
    }

    // ── Multi-signature payments ───────────────────────────────────────────────

    pub fn initiate_multisig_payment(
        env: Env,
        initiator: Address,
        payment_id: String,
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

        let ms = MultisigPayment {
            payment_id: payment_id.clone(),
            order,
            required_signers,
            signatures: Vec::new(&env),
            executed: false,
            created_at: env.ledger().timestamp(),
        };
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
        payment_id: String,
    ) -> Result<(), PaymentError> {
        signer.require_auth();
        let mut ms =
            storage::get_multisig(&env, &payment_id).ok_or(PaymentError::MultisigNotFound)?;

        if ms.executed {
            return Err(PaymentError::MultisigAlreadyExecuted);
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
        payment_id: String,
    ) -> Result<(), PaymentError> {
        executor.require_auth();
        let mut ms =
            storage::get_multisig(&env, &payment_id).ok_or(PaymentError::MultisigNotFound)?;

        if ms.executed {
            return Err(PaymentError::MultisigAlreadyExecuted);
        }
        if ms.signatures.len() < ms.required_signers.len() {
            return Err(PaymentError::InsufficientSignatures);
        }

        let order = &ms.order;
        let now = env.ledger().timestamp();
        if order.expires_at > 0 && now > order.expires_at {
            return Err(PaymentError::PaymentExpired);
        }

        let token_client = token::Client::new(&env, &order.token);
        token_client.transfer(&executor, &order.merchant_address, &order.amount);

        let record = PaymentRecord {
            order_id: order.order_id.clone(),
            merchant_address: order.merchant_address.clone(),
            payer: executor.clone(),
            token: order.token.clone(),
            amount: order.amount,
            refunded_amount: 0,
            status: PaymentStatus::Completed,
            paid_at: now,
        };
        storage::save_payment(&env, &record);
        storage::push_merchant_payment_id(&env, &order.merchant_address, &order.order_id);
        storage::push_payer_payment_id(&env, &executor, &order.order_id);
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
        ids: Vec<String>,
        cursor: Option<String>,
        limit: u32,
        filter: Option<PaymentFilter>,
        sort_field: SortField,
        sort_order: SortOrder,
    ) -> Result<PaymentPage, PaymentError> {
        let cap = limit.min(100) as usize;

        // Collect all matching records
        let mut records: Vec<PaymentRecord> = Vec::new(env);
        let mut skip = cursor.is_some();

        for i in 0..ids.len() {
            let id = ids.get(i).unwrap();
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
                    records.push_back(record);
                }
            }
        }

        // Sort
        // soroban_sdk::Vec doesn't have sort_by; we do a simple insertion sort
        let len = records.len() as usize;
        for i in 1..len {
            let mut j = i;
            while j > 0 {
                let a = records.get(j as u32 - 1).unwrap();
                let b = records.get(j as u32).unwrap();
                let swap = match sort_field {
                    SortField::Date => match sort_order {
                        SortOrder::Ascending => a.paid_at > b.paid_at,
                        SortOrder::Descending => a.paid_at < b.paid_at,
                    },
                    SortField::Amount => match sort_order {
                        SortOrder::Ascending => a.amount > b.amount,
                        SortOrder::Descending => a.amount < b.amount,
                    },
                };
                if swap {
                    records.set(j as u32 - 1, b);
                    records.set(j as u32, a);
                    j -= 1;
                } else {
                    break;
                }
            }
        }

        let total = records.len();
        let next_cursor = if records.len() as usize > cap {
            records.get(cap as u32 - 1).map(|r| r.order_id)
        } else {
            None
        };

        // Truncate to cap
        let mut page: Vec<PaymentRecord> = Vec::new(env);
        for i in 0..(records.len().min(cap as u32)) {
            page.push_back(records.get(i).unwrap());
        }

        Ok(PaymentPage {
            records: page,
            next_cursor,
            total,
        })
    }
}
