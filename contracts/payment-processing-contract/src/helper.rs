use soroban_sdk::{Address, Bytes, BytesN, Env, String};

use crate::error::PaymentError;
use crate::storage;
use crate::types::{PaymentFilter, PaymentRecord, PaymentStatus, StatusFilter};

/// Require that `caller` is the contract admin.
pub fn require_admin(env: &Env, caller: &Address) -> Result<(), PaymentError> {
    caller.require_auth();
    let admin = storage::get_admin(env).ok_or(PaymentError::Unauthorized)?;
    if admin != *caller {
        return Err(PaymentError::Unauthorized);
    }
    Ok(())
}

/// Require that `caller` is the registered merchant at `merchant_address`.
pub fn require_merchant(
    env: &Env,
    caller: &Address,
    merchant_address: &Address,
) -> Result<(), PaymentError> {
    caller.require_auth();
    if caller != merchant_address {
        return Err(PaymentError::Unauthorized);
    }
    let m = storage::get_merchant(env, merchant_address).ok_or(PaymentError::MerchantNotFound)?;
    if !m.active {
        return Err(PaymentError::MerchantInactive);
    }
    Ok(())
}

/// Validate that `amount` is positive.
pub fn validate_amount(amount: i128) -> Result<(), PaymentError> {
    if amount <= 0 {
        return Err(PaymentError::InvalidAmount);
    }
    Ok(())
}

/// Validate that `order_id` is non-empty.
pub fn validate_order_id(order_id: &String) -> Result<(), PaymentError> {
    if order_id.len() == 0 {
        return Err(PaymentError::InvalidInput);
    }
    Ok(())
}

/// Verify an ed25519 signature over `payload` using `public_key`.
pub fn verify_signature(
    env: &Env,
    public_key: &BytesN<32>,
    payload: &Bytes,
    signature: &BytesN<64>,
) -> Result<(), PaymentError> {
    env.crypto()
        .ed25519_verify(public_key, payload, signature);
    Ok(())
}

/// Apply a filter to a payment record; returns true if the record passes.
pub fn matches_filter(record: &PaymentRecord, filter: &PaymentFilter) -> bool {
    if let Some(start) = filter.date_start {
        if record.paid_at < start {
            return false;
        }
    }
    if let Some(end) = filter.date_end {
        if record.paid_at > end {
            return false;
        }
    }
    if let Some(min) = filter.amount_min {
        if record.amount < min {
            return false;
        }
    }
    if let Some(max) = filter.amount_max {
        if record.amount > max {
            return false;
        }
    }
    if let Some(ref token) = filter.token {
        if record.token != *token {
            return false;
        }
    }
    match &filter.status {
        StatusFilter::Any => {}
        StatusFilter::Completed => {
            if record.status != PaymentStatus::Completed {
                return false;
            }
        }
        StatusFilter::PartiallyRefunded => {
            if record.status != PaymentStatus::PartiallyRefunded {
                return false;
            }
        }
        StatusFilter::FullyRefunded => {
            if record.status != PaymentStatus::FullyRefunded {
                return false;
            }
        }
    }
    true
}
