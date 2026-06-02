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

/// Validate that `admin` is not the zero/burn address.
pub fn validate_admin_address(env: &Env, admin: &Address) -> Result<(), PaymentError> {
    // The Soroban SDK does not expose a dedicated zero/burn address validation API
    // for `Address`. This is a best-effort guard against a zero-address
    // representation when the SDK serialization exposes it.
    let admin_xdr = admin.clone().to_xdr(env);
    let all_zero = admin_xdr.iter().all(|&b| b == 0);
    if all_zero {
        return Err(PaymentError::InvalidInput);
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

/// Validate merchant string fields: name, description, contact_info
pub fn validate_merchant_fields(
    name: &String,
    description: &String,
    contact_info: &String,
) -> Result<(), PaymentError> {
    // name <= 64 bytes
    let name_bytes = name.to_string().as_bytes();
    if name_bytes.len() > 64 {
        return Err(PaymentError::InvalidInput);
    }

    // description <= 256 bytes
    let desc_bytes = description.to_string().as_bytes();
    if desc_bytes.len() > 256 {
        return Err(PaymentError::InvalidInput);
    }

    // contact_info <= 128 bytes and printable ASCII only
    let contact_bytes = contact_info.to_string().as_bytes();
    if contact_bytes.len() > 128 {
        return Err(PaymentError::InvalidInput);
    }
    for &b in contact_bytes.iter() {
        if b < 0x20 || b > 0x7E {
            return Err(PaymentError::InvalidInput);
        }
    }

    Ok(())
}

/// Validate that `order_id` is non-empty.
pub fn validate_order_id(order_id: &Bytes) -> Result<(), PaymentError> {
    // Enforce non-empty, max 64 bytes
    let len = order_id.len();
    if len == 0 || len > 64 {
        return Err(PaymentError::InvalidInput);
    }
    // character check is omitted for Bytes as it's harder in no_std without String
    Ok(())
}

/// Verify an ed25519 signature over `payload` using `public_key`.
pub fn verify_signature(
    env: &Env,
    public_key: &BytesN<32>,
    payload: &Bytes,
    signature: &BytesN<64>,
) -> Result<(), PaymentError> {
    let pk: BytesN<32> = public_key
        .clone()
        .try_into()
        .map_err(|_| PaymentError::InvalidInput)?;
    let sig: BytesN<64> = signature
        .clone()
        .try_into()
        .map_err(|_| PaymentError::InvalidInput)?;

    env.crypto().ed25519_verify(&pk, payload, &sig);
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
    if let Some(ref tokens) = filter.tokens {
        // Empty list → no filter (match all). Non-empty → token must be in list.
        if !tokens.is_empty() && !tokens.contains(&record.token) {
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
