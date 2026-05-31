#![no_std]

extern crate alloc;

mod error;
mod helper;
mod storage;
mod types;

#[cfg(test)]
mod test;
#[cfg(test)]
mod repro_tests;

use alloc::vec::Vec as RustVec;
use soroban_sdk::{
    contract, contractimpl, token, xdr::ToXdr, Address, Bytes, BytesN, Env, String, Vec,
};

use error::PaymentError;
use storage::REFUND_WINDOW;
use types::{
    DataKey, GlobalStats, Merchant, MerchantCategory, MultisigPayment, PaymentFilter, PaymentOrder,
    PaymentPage, PaymentRecord, PaymentStatus, RefundRecord, RefundStatus, SortField, SortOrder,
    SubscriptionPlan, SubscriptionState, SubscriptionStatus,
};

#[contract]
pub struct PaymentContract;

#[contractimpl]
impl PaymentContract {
    // ── Admin ─────────────────────────────────────────────────────────────────

    /// One-time admin initialisation. Stores the first admin address.
    pub fn set_admin(env: Env, admins: Vec<Address>, threshold: u32) -> Result<(), PaymentError> {
        if storage::get_admin_config(&env).is_some() || storage::get_admin(&env).is_some() {
            return Err(PaymentError::AdminAlreadySet);
        }
        if admins.is_empty() {
            return Err(PaymentError::InvalidInput);
        }
        let first = admins.get(0).unwrap();
        helper::validate_admin_address(&env, &first)?;
        first.require_auth();
        // Store the first admin for single-admin helpers; also store full config.
        storage::set_admin(&env, &first);
        storage::set_admin_config(
            &env,
            &types::AdminConfig {
                admins,
                threshold,
            },
        );
        storage::set_contract_version(&env, 1);
        env.events()
            .publish((DataKey::Admin,), (String::from_str(&env, "admin_set"), first));
        Ok(())
    }

    pub fn upgrade(env: Env, admin: Address, new_wasm_hash: BytesN<32>) -> Result<(), PaymentError> {
        helper::require_admin(&env, &admin)?;
        env.deployer().update_current_contract_wasm(new_wasm_hash);
        Ok(())
    }

    pub fn get_version(env: Env) -> u32 {
        storage::get_contract_version(&env)
    }

    // ── Merchant management ───────────────────────────────────────────────────

    pub fn register_merchant(
        env: Env,
        merchant_address: Address,
        name: String,
        description: String,
        contact_info: String,
        category: MerchantCategory,
        signing_public_key: Option<BytesN<32>>,
    ) -> Result<(), PaymentError> {
        merchant_address.require_auth();
        if storage::get_merchant(&env, &merchant_address).is_some() {
            return Err(PaymentError::MerchantAlreadyRegistered);
        }
        if storage::is_whitelist_enabled(&env)
            && !storage::is_whitelisted(&env, &merchant_address)
        {
            return Err(PaymentError::Unauthorized);
        }
        helper::validate_merchant_fields(&name, &description, &contact_info)?;
        let merchant = Merchant {
            address: merchant_address.clone(),
            name,
            description,
            contact_info,
            category,
            active: true,
            registered_at: env.ledger().timestamp(),
            signing_public_key,
        };
        storage::save_merchant(&env, &merchant);
        env.events().publish(
            (String::from_str(&env, "merchant_registered"),),
            merchant_address,
        );
        Ok(())
    }

    pub fn set_whitelist_mode(
        env: Env,
        admins: Vec<Address>,
        enabled: bool,
    ) -> Result<(), PaymentError> {
        helper::require_multi_admin(&env, admins)?;
        storage::set_whitelist_enabled(&env, enabled);
        Ok(())
    }

    pub fn approve_merchant_registration(
        env: Env,
        admins: Vec<Address>,
        merchant_address: Address,
    ) -> Result<(), PaymentError> {
        helper::require_multi_admin(&env, admins)?;
        storage::set_whitelisted(&env, &merchant_address, true);
        Ok(())
    }

    pub fn deactivate_merchant(
        env: Env,
        merchant_address: Address,
        admin_authorizers: Option<Vec<Address>>,
    ) -> Result<(), PaymentError> {
        if let Some(admins) = admin_authorizers {
            helper::require_multi_admin(&env, admins)?;
        } else {
            merchant_address.require_auth();
        }
        let mut merchant =
            storage::get_merchant(&env, &merchant_address).ok_or(PaymentError::MerchantNotFound)?;
        merchant.active = false;
        storage::save_merchant(&env, &merchant);
        env.events().publish(
            (String::from_str(&env, "merchant_deactivated"),),
            merchant_address,
        );
        Ok(())
    }

    pub fn get_merchant(env: Env, merchant_address: Address) -> Result<Merchant, PaymentError> {
        storage::get_merchant(&env, &merchant_address).ok_or(PaymentError::MerchantNotFound)
    }

    // ── Payment processing ────────────────────────────────────────────────────

    pub fn process_payment_with_signature(
        env: Env,
        payer: Address,
        order: PaymentOrder,
        signature: BytesN<64>,
        merchant_public_key: BytesN<32>,
    ) -> Result<(), PaymentError> {
        payer.require_auth();
        if order.payer != payer {
            return Err(PaymentError::InvalidInput);
        }
        helper::validate_amount(order.amount)?;
        helper::validate_order_id(&order.order_id)?;
        if storage::get_payment(&env, &order.order_id).is_some() {
            return Err(PaymentError::PaymentAlreadyExists);
        }
        let now = env.ledger().timestamp();
        if order.expires_at > 0 && now > order.expires_at {
            return Err(PaymentError::PaymentExpired);
        }
        let merchant = storage::get_merchant(&env, &order.merchant_address)
            .ok_or(PaymentError::MerchantNotFound)?;
        if !merchant.active {
            return Err(PaymentError::MerchantInactive);
        }
        let stored_key = merchant
            .signing_public_key
            .unwrap_or_else(|| BytesN::from_array(&env, &[0u8; 32]));
        let zero_key = BytesN::from_array(&env, &[0u8; 32]);
        if stored_key != zero_key {
            let payload = order.clone().to_xdr(&env);
            helper::verify_signature(&env, &stored_key, &payload, &signature)?;
        }
        let record = PaymentRecord {
            order_id: order.order_id.clone(),
            merchant_address: order.merchant_address.clone(),
            payer: payer.clone(),
            token: order.token.clone(),
            amount: order.amount,
            refunded_amount: 0,
            pending_refund_amount: 0,
            status: PaymentStatus::Completed,
            paid_at: now,
            description: order.description.clone(),
        };
        storage::save_payment(&env, &record);
        storage::push_merchant_payment_id(&env, &order.merchant_address, &order.order_id);
        storage::push_payer_payment_id(&env, &payer, &order.order_id);
        storage::push_global_payment_id(&env, &order.order_id);
        storage::increment_payment_stats(&env, order.amount)?;
        let token_client = token::Client::new(&env, &order.token);
        token_client.transfer(&payer, &order.merchant_address, &order.amount);
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
        let record = storage::get_payment(&env, &order_id).ok_or(PaymentError::PaymentNotFound)?;
        let is_admin = if let Some(config) = storage::get_admin_config(&env) {
            config.admins.contains(&caller)
        } else if let Some(admin) = storage::get_admin(&env) {
            admin == caller
        } else {
            false
        };
        if caller != record.payer && caller != record.merchant_address && !is_admin {
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
        helper::require_merchant(&env, &merchant, &merchant)?;
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
        admins: Vec<Address>,
        date_start: Option<u64>,
        date_end: Option<u64>,
    ) -> Result<GlobalStats, PaymentError> {
        helper::require_multi_admin(&env, admins)?;
        if date_start.is_none() && date_end.is_none() {
            return Ok(storage::get_global_stats(&env));
        }
        let mut stats = GlobalStats {
            total_payments: 0,
            total_volume: 0,
            total_refunds: 0,
            total_refund_volume: 0,
        };
        for id in storage::get_global_payment_ids(&env).iter() {
            if let Some(record) = storage::get_payment(&env, &id) {
                if helper::in_date_range(record.paid_at, date_start, date_end) {
                    stats.total_payments += 1;
                    stats.total_volume = stats
                        .total_volume
                        .checked_add(record.amount)
                        .ok_or(PaymentError::ArithmeticError)?;
                }
            }
        }
        for id in storage::get_all_refund_ids(&env).iter() {
            if let Some(record) = storage::get_refund(&env, &id) {
                if helper::in_date_range(record.initiated_at, date_start, date_end) {
                    stats.total_refunds += 1;
                    stats.total_refund_volume = stats
                        .total_refund_volume
                        .checked_add(record.amount)
                        .ok_or(PaymentError::ArithmeticError)?;
                }
            }
        }
        Ok(stats)
    }

    // ── Payment management ────────────────────────────────────────────────────

    pub fn update_payment_status(
        env: Env,
        _caller: Address,
        _order_id: Bytes,
        _refunded_amount: i128,
    ) -> Result<(), PaymentError> {
        // Intentionally disabled: refund state is managed exclusively via the
        // initiate/approve/execute refund workflow.
        Err(PaymentError::InvalidInput)
    }

    pub fn archive_payment_record(
        env: Env,
        admins: Vec<Address>,
        order_id: Bytes,
    ) -> Result<(), PaymentError> {
        helper::require_multi_admin(&env, admins)?;
        let record = storage::get_payment(&env, &order_id).ok_or(PaymentError::PaymentNotFound)?;
        storage::remove_payment(&env, &order_id);
        storage::remove_merchant_payment_id(&env, &record.merchant_address, &order_id);
        storage::remove_payer_payment_id(&env, &record.payer, &order_id);
        storage::remove_global_payment_id(&env, &order_id);
        Ok(())
    }

    pub fn cleanup_expired_payments(env: Env, admins: Vec<Address>) -> Result<u32, PaymentError> {
        helper::require_multi_admin(&env, admins)?;
        let period = storage::get_cleanup_period(&env);
        let now = env.ledger().timestamp();
        let cutoff = now.saturating_sub(period);
        let ids = storage::get_global_payment_ids(&env);
        let mut new_ids = Vec::new(&env);
        let mut count = 0u32;
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
        admins: Vec<Address>,
        period: u64,
    ) -> Result<(), PaymentError> {
        helper::require_multi_admin(&env, admins)?;
        if period == 0 {
            return Err(PaymentError::InvalidInput);
        }
        storage::set_cleanup_period(&env, period);
        Ok(())
    }

    pub fn set_default_multisig_expiry(
        env: Env,
        admins: Vec<Address>,
        expiry: u64,
    ) -> Result<(), PaymentError> {
        helper::require_multi_admin(&env, admins)?;
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
        if reason.len() > 256 {
            return Err(PaymentError::InvalidInput);
        }
        let mut record =
            storage::get_payment(&env, &order_id).ok_or(PaymentError::PaymentNotFound)?;
        if caller != record.payer && caller != record.merchant_address {
            return Err(PaymentError::Unauthorized);
        }
        let now = env.ledger().timestamp();
        if now > record.paid_at + REFUND_WINDOW {
            return Err(PaymentError::RefundWindowExpired);
        }
        let new_total = record.refunded_amount + record.pending_refund_amount + amount;
        if new_total > record.amount {
            return Err(PaymentError::RefundAmountExceedsPayment);
        }
        if storage::get_refund(&env, &refund_id).is_some() {
            return Err(PaymentError::RefundAlreadyExists);
        }
        let refund = RefundRecord {
            refund_id: refund_id.clone(),
            order_id: order_id.clone(),
            amount,
            reason,
            status: RefundStatus::Pending,
            initiated_by: caller.clone(),
            initiated_at: now,
        };
        storage::save_refund(&env, &refund);
        record.pending_refund_amount += amount;
        storage::save_payment(&env, &record);
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
        admin_authorizers: Option<Vec<Address>>,
    ) -> Result<(), PaymentError> {
        caller.require_auth();
        let mut refund =
            storage::get_refund(&env, &refund_id).ok_or(PaymentError::RefundNotFound)?;
        let record =
            storage::get_payment(&env, &refund.order_id).ok_or(PaymentError::PaymentNotFound)?;
        let is_authorized = if let Some(admins) = admin_authorizers {
            helper::require_multi_admin(&env, admins).is_ok()
        } else {
            helper::require_merchant(&env, &caller, &record.merchant_address).is_ok()
        };
        if !is_authorized {
            return Err(PaymentError::Unauthorized);
        }
        if refund.status != RefundStatus::Pending {
            return Err(PaymentError::RefundAlreadyCompleted);
        }
        refund.status = RefundStatus::Approved;
        storage::save_refund(&env, &refund);
        env.events()
            .publish((String::from_str(&env, "refund_approved"),), refund_id);
        Ok(())
    }

    pub fn reject_refund(
        env: Env,
        caller: Address,
        refund_id: Bytes,
        admin_authorizers: Option<Vec<Address>>,
    ) -> Result<(), PaymentError> {
        caller.require_auth();
        let mut refund =
            storage::get_refund(&env, &refund_id).ok_or(PaymentError::RefundNotFound)?;
        let record =
            storage::get_payment(&env, &refund.order_id).ok_or(PaymentError::PaymentNotFound)?;
        let is_authorized = if let Some(admins) = admin_authorizers {
            helper::require_multi_admin(&env, admins).is_ok()
        } else {
            helper::require_merchant(&env, &caller, &record.merchant_address).is_ok()
        };
        if !is_authorized {
            return Err(PaymentError::Unauthorized);
        }
        if refund.status != RefundStatus::Pending {
            return Err(PaymentError::RefundAlreadyCompleted);
        }
        refund.status = RefundStatus::Rejected;
        storage::save_refund(&env, &refund);
        let mut record = storage::get_payment(&env, &refund.order_id)
            .ok_or(PaymentError::PaymentNotFound)?;
        record.pending_refund_amount = record.pending_refund_amount.saturating_sub(refund.amount);
        storage::save_payment(&env, &record);
        env.events()
            .publish((String::from_str(&env, "refund_rejected"),), refund_id);
        Ok(())
    }

    pub fn execute_refund(env: Env, caller: Address, refund_id: Bytes) -> Result<(), PaymentError> {
        caller.require_auth();
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
        let new_total = record.refunded_amount + refund.amount;
        record.refunded_amount = new_total;
        record.pending_refund_amount = record.pending_refund_amount.saturating_sub(refund.amount);
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
        let token_client = token::Client::new(&env, &record.token);
        token_client.transfer(&record.merchant_address, &record.payer, &refund.amount);
        env.events().publish(
            (String::from_str(&env, "refund_executed"),),
            (refund_id, refund.amount),
        );
        Ok(())
    }

    pub fn get_refund_status(env: Env, refund_id: Bytes) -> Result<RefundStatus, PaymentError> {
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
        if required_signers.is_empty() || required_signers.len() > storage::MAX_SIGNERS {
            return Err(PaymentError::InvalidInput);
        }
        let merchant = storage::get_merchant(&env, &order.merchant_address)
            .ok_or(PaymentError::MerchantNotFound)?;
        if !merchant.active {
            return Err(PaymentError::MerchantInactive);
        }
        let mut unique_signers = Vec::new(&env);
        for signer in required_signers.iter() {
            if unique_signers.contains(&signer) {
                return Err(PaymentError::InvalidInput);
            }
            unique_signers.push_back(signer);
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
        let token_client = token::Client::new(&env, &ms.order.token);
        let contract_addr = env.current_contract_address();
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
        let token_client = token::Client::new(&env, &order.token);
        let contract_addr = env.current_contract_address();
        token_client.transfer(&contract_addr, &order.merchant_address, &order.amount);
        let record = PaymentRecord {
            order_id: order.order_id.clone(),
            merchant_address: order.merchant_address.clone(),
            payer: ms.order.payer.clone(),
            token: order.token.clone(),
            amount: order.amount,
            refunded_amount: 0,
            pending_refund_amount: 0,
            status: PaymentStatus::Completed,
            paid_at: now,
            description: order.description.clone(),
        };
        storage::save_payment(&env, &record);
        storage::push_merchant_payment_id(&env, &order.merchant_address, &order.order_id);
        storage::push_payer_payment_id(&env, &executor, &order.order_id);
        storage::push_global_payment_id(&env, &order.order_id);
        storage::increment_payment_stats(&env, order.amount)?;
        ms.executed = true;
        storage::save_multisig(&env, &ms);
        env.events().publish(
            (String::from_str(&env, "multisig_executed"),),
            (payment_id, executor, order.amount),
        );
        Ok(())
    }

    // ── Subscriptions ─────────────────────────────────────────────────────────
    //
    // IMPORTANT — Off-chain scheduler requirement:
    // Soroban contracts cannot autonomously schedule execution. An off-chain
    // scheduler service MUST call `process_subscription_payment` at each
    // interval boundary (i.e. when `now >= last_charged_at + plan.interval`).
    // The contract enforces correctness and idempotency; the scheduler is
    // responsible for timely invocation.

    /// Create a new subscription for a payer–merchant pair.
    ///
    /// The payer authorises this call. The subscription starts in `Active`
    /// state with `last_charged_at = 0` (no payment yet). The first payment
    /// is collected immediately by calling `process_subscription_payment`.
    pub fn create_subscription(
        env: Env,
        payer: Address,
        merchant: Address,
        subscription_id: Bytes,
        plan: SubscriptionPlan,
    ) -> Result<(), PaymentError> {
        payer.require_auth();
        if plan.interval == 0 {
            return Err(PaymentError::InvalidInput);
        }
        helper::validate_amount(plan.amount)?;
        let m = storage::get_merchant(&env, &merchant).ok_or(PaymentError::MerchantNotFound)?;
        if !m.active {
            return Err(PaymentError::MerchantInactive);
        }
        if storage::get_subscription(&env, &subscription_id).is_some() {
            return Err(PaymentError::SubscriptionAlreadyExists);
        }
        let sub = SubscriptionState {
            subscription_id: subscription_id.clone(),
            payer,
            merchant,
            plan,
            status: SubscriptionStatus::Active,
            created_at: env.ledger().timestamp(),
            last_charged_at: 0,
        };
        storage::save_subscription(&env, &sub);
        env.events().publish(
            (String::from_str(&env, "subscription_created"),),
            subscription_id,
        );
        Ok(())
    }

    /// Cancel an active subscription. Only the payer may cancel.
    pub fn cancel_subscription(
        env: Env,
        payer: Address,
        subscription_id: Bytes,
    ) -> Result<(), PaymentError> {
        payer.require_auth();
        let mut sub = storage::get_subscription(&env, &subscription_id)
            .ok_or(PaymentError::SubscriptionNotFound)?;
        if sub.payer != payer {
            return Err(PaymentError::Unauthorized);
        }
        if sub.status != SubscriptionStatus::Active {
            return Err(PaymentError::SubscriptionNotActive);
        }
        sub.status = SubscriptionStatus::Cancelled;
        storage::save_subscription(&env, &sub);
        env.events().publish(
            (String::from_str(&env, "subscription_cancelled"),),
            subscription_id,
        );
        Ok(())
    }

    /// Execute a scheduled recurring payment for an active subscription.
    ///
    /// # Off-chain scheduler requirement
    /// This function MUST be called by an off-chain scheduler service once the
    /// interval has elapsed (`now >= last_charged_at + plan.interval`). The
    /// contract validates the interval guard to prevent double-charging within
    /// the same window. The scheduler must track `last_charged_at` (available
    /// via `get_subscription`) and invoke this function at the correct time.
    ///
    /// # Idempotency
    /// If called before the interval has elapsed, the call fails with
    /// `SubscriptionIntervalNotElapsed`, making it safe to retry.
    pub fn process_subscription_payment(
        env: Env,
        caller: Address,
        subscription_id: Bytes,
    ) -> Result<(), PaymentError> {
        caller.require_auth();
        let mut sub = storage::get_subscription(&env, &subscription_id)
            .ok_or(PaymentError::SubscriptionNotFound)?;
        if sub.status != SubscriptionStatus::Active {
            return Err(PaymentError::SubscriptionNotActive);
        }
        let now = env.ledger().timestamp();
        // Interval guard: prevent double-charging within the same window.
        // For the very first payment (last_charged_at == 0) the guard is skipped.
        if sub.last_charged_at > 0 && now < sub.last_charged_at + sub.plan.interval {
            return Err(PaymentError::SubscriptionIntervalNotElapsed);
        }
        let m = storage::get_merchant(&env, &sub.merchant)
            .ok_or(PaymentError::MerchantNotFound)?;
        if !m.active {
            return Err(PaymentError::MerchantInactive);
        }
        // Transfer recurring payment from payer to merchant.
        let token_client = token::Client::new(&env, &sub.plan.token);
        token_client.transfer(&sub.payer, &sub.merchant, &sub.plan.amount);
        sub.last_charged_at = now;
        storage::save_subscription(&env, &sub);
        env.events().publish(
            (String::from_str(&env, "subscription_payment_processed"),),
            (subscription_id, sub.plan.amount, now),
        );
        Ok(())
    }

    /// Retrieve the current state of a subscription.
    pub fn get_subscription(
        env: Env,
        subscription_id: Bytes,
    ) -> Result<SubscriptionState, PaymentError> {
        storage::get_subscription(&env, &subscription_id)
            .ok_or(PaymentError::SubscriptionNotFound)
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
