#![cfg(test)]

extern crate alloc;
use alloc::vec;

use soroban_sdk::{
    testutils::{Address as _, Events as _, Ledger},
    token::StellarAssetClient,
    Address, Bytes, BytesN, Env, IntoVal, String, Vec,
};

use ed25519_dalek::{Signer, SigningKey};

use crate::{
    error::PaymentError,
    types::{MerchantCategory, PaymentOrder, PaymentStatus, RefundStatus, SortField, SortOrder},
    PaymentContract, PaymentContractClient,
};

use soroban_sdk::xdr::ToXdr;

// ── Test helpers ──────────────────────────────────────────────────────────────

fn sign_order(env: &Env, order: &PaymentOrder) -> (BytesN<32>, BytesN<64>) {
    let signing_key = SigningKey::from_bytes(&[1u8; 32]);
    let public_key = signing_key.verifying_key();

    let payload = order.clone().to_xdr(env);
    let mut payload_bytes = vec![0u8; payload.len() as usize];
    payload.copy_into_slice(&mut payload_bytes);

    let signature = signing_key.sign(&payload_bytes);

    (
        BytesN::from_array(env, &public_key.to_bytes()),
        BytesN::from_array(env, &signature.to_bytes()),
    )
}

fn setup() -> (Env, PaymentContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PaymentContract);
    let client = PaymentContractClient::new(&env, &contract_id);
    (env, client)
}

fn create_token(env: &Env, admin: &Address) -> Address {
    let token_id = env.register_stellar_asset_contract_v2(admin.clone());
    token_id.address()
}

fn mint(env: &Env, token: &Address, _admin: &Address, to: &Address, amount: i128) {
    StellarAssetClient::new(env, token).mint(to, &amount);
}

fn str(env: &Env, s: &str) -> String {
    String::from_str(env, s)
}

fn bytes(env: &Env, s: &str) -> Bytes {
    Bytes::from_slice(env, s.as_bytes())
}

fn make_order(env: &Env, merchant: &Address, payer: &Address, token: &Address) -> PaymentOrder {
    PaymentOrder {
        order_id: bytes(env, "ORDER_001"),
        merchant_address: merchant.clone(),
        payer: payer.clone(),
        token: token.clone(),
        amount: 1000,
        description: str(env, "Test order"),
        expires_at: 0,
    }
}

#[test]
fn test_payment_payer_mismatch_fails() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let other_payer = Address::generate(&env);
    let token = create_token(&env, &admin);

    client.set_admin(&admin);
    client.register_merchant(
        &merchant,
        &str(&env, "Store"),
        &str(&env, "desc"),
        &str(&env, "c@c.com "),
        &MerchantCategory::Retail,
    );
    mint(&env, &token, &admin, &payer, 5000);

    // Order signed for other_payer but submitted by `payer`.
    let order = make_order(&env, &merchant, &other_payer, &token);
    let (pub_key, sig) = sign_order(&env, &order);

    let result = client.try_process_payment_with_signature(&payer, &order, &sig, &pub_key);
    assert_eq!(result, Err(Ok(PaymentError::InvalidInput)));
}

#[test]
fn test_register_merchant_field_limits() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    client.set_admin(&admin);

    // name: 64 ok, 65 fails
    let m1 = Address::generate(&env);
    let name_ok = "n".repeat(64);
    client.register_merchant(
        &m1,
        &str(&env, &name_ok),
        &str(&env, "desc"),
        &str(&env, "c@c.com"),
        &MerchantCategory::Retail,
    );

    let m2 = Address::generate(&env);
    let name_bad = "n".repeat(65);
    let result = client.try_register_merchant(
        &m2,
        &str(&env, &name_bad),
        &str(&env, "desc"),
        &str(&env, "c@c.com"),
        &MerchantCategory::Retail,
    );
    assert_eq!(result, Err(Ok(PaymentError::InvalidInput)));

    // description: 256 ok, 257 fails
    let m3 = Address::generate(&env);
    let desc_ok = "d".repeat(256);
    client.register_merchant(
        &m3,
        &str(&env, "Store"),
        &str(&env, &desc_ok),
        &str(&env, "c@c.com"),
        &MerchantCategory::Retail,
    );

    let m4 = Address::generate(&env);
    let desc_bad = "d".repeat(257);
    let result = client.try_register_merchant(
        &m4,
        &str(&env, "Store"),
        &str(&env, &desc_bad),
        &str(&env, "c@c.com"),
        &MerchantCategory::Retail,
    );
    assert_eq!(result, Err(Ok(PaymentError::InvalidInput)));

    // contact_info: 128 ok, 129 fails
    let m5 = Address::generate(&env);
    let contact_ok = "c".repeat(128);
    client.register_merchant(
        &m5,
        &str(&env, "Store"),
        &str(&env, "desc"),
        &str(&env, &contact_ok),
        &MerchantCategory::Retail,
    );

    let m6 = Address::generate(&env);
    let contact_bad = "c".repeat(129);
    let result = client.try_register_merchant(
        &m6,
        &str(&env, "Store"),
        &str(&env, "desc"),
        &str(&env, &contact_bad),
        &MerchantCategory::Retail,
    );
    assert_eq!(result, Err(Ok(PaymentError::InvalidInput)));
}

#[test]
fn test_register_contact_info_sanitisation_rejects_control_chars() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    client.set_admin(&admin);
    let merchant = Address::generate(&env);

    // contact_info with control character (0x01)
    let bad_contact = String::from_str(&env, "bad\x01contact");
    let result = client.try_register_merchant(
        &merchant,
        &str(&env, "Store"),
        &str(&env, "desc"),
        &bad_contact,
        &MerchantCategory::Retail,
    );
    assert_eq!(result, Err(Ok(PaymentError::InvalidInput)));
}

// ── Admin tests ───────────────────────────────────────────────────────────────

#[test]
fn test_set_admin_success() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    client.set_admin(&admin);
}

#[test]
fn test_set_admin_twice_fails() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    client.set_admin(&admin);
    let result = client.try_set_admin(&admin);
    assert_eq!(result, Err(Ok(PaymentError::AdminAlreadySet)));
}

// ── Merchant tests ────────────────────────────────────────────────────────────

#[test]
fn test_register_merchant_success() {
    let (env, client) = setup();
    let merchant = Address::generate(&env);
    client.register_merchant(
        &merchant,
        &str(&env, "My Store"),
        &str(&env, "A great store"),
        &str(&env, "contact@store.com"),
        &MerchantCategory::Retail,
        &None,
    );
    let m = client.get_merchant(&merchant);
    assert_eq!(m.name, str(&env, "My Store"));
    assert!(m.active);
}

#[test]
fn test_register_merchant_duplicate_fails() {
    let (env, client) = setup();
    let merchant = Address::generate(&env);
    client.register_merchant(
        &merchant,
        &str(&env, "Store"),
        &str(&env, "desc"),
        &str(&env, "c@c.com "),
        &MerchantCategory::Retail,
        &None,
    );
    let result = client.try_register_merchant(
        &merchant,
        &str(&env, "Store"),
        &str(&env, "desc"),
        &str(&env, "c@c.com "),
        &MerchantCategory::Retail,
        &None,
    );
    assert_eq!(result, Err(Ok(PaymentError::MerchantAlreadyRegistered)));
}

#[test]
fn test_deactivate_merchant() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    client.set_admin(&admin);
    client.register_merchant(
        &merchant,
        &str(&env, "Store"),
        &str(&env, "desc"),
        &str(&env, "c@c.com "),
        &MerchantCategory::Retail,
        &None,
    );
    client.deactivate_merchant(&admin, &merchant);
    let m = client.get_merchant(&merchant);
    assert!(!m.active);
}

#[test]
fn test_reactivate_merchant_success() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = create_token(&env, &admin);

    client.set_admin(&admin);
    client.register_merchant(
        &merchant,
        &str(&env, "Store"),
        &str(&env, "desc"),
        &str(&env, "c@c.com "),
        &MerchantCategory::Retail,
        &None,
    );
    mint(&env, &token, &admin, &payer, 5000);

    // Deactivate
    client.deactivate_merchant(&admin, &merchant);
    let m = client.get_merchant(&merchant);
    assert!(!m.active);

    // Try payment (should fail)
    let order = make_order(&env, &merchant, &payer, &token);
    let (pub_key, sig) = sign_order(&env, &order);
    let result = client.try_process_payment_with_signature(&payer, &order, &sig);
    assert_eq!(result, Err(Ok(PaymentError::MerchantInactive)));

    // Reactivate
    client.reactivate_merchant(&merchant, &merchant);
    let m = client.get_merchant(&merchant);
    assert!(m.active);

    // Process payment
    client.process_payment_with_signature(&payer, &order, &sig);
    let record = client.get_payment_by_id(&payer, &bytes(&env, "ORDER_001"));
    assert_eq!(record.status, PaymentStatus::Completed);
}

// ── Payment tests ─────────────────────────────────────────────────────────────

#[test]
fn test_successful_payment_with_signature() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = create_token(&env, &admin);

    client.set_admin(&admin);
    client.register_merchant(
        &merchant,
        &str(&env, "Store"),
        &str(&env, "desc"),
        &str(&env, "c@c.com "),
        &MerchantCategory::Retail,
        &None,
    );
    mint(&env, &token, &admin, &payer, 5000);

    let order = make_order(&env, &merchant, &payer, &token);
    let (pub_key, sig) = sign_order(&env, &order);

    client.process_payment_with_signature(&payer, &order, &sig);

    let record = client.get_payment_by_id(&payer, &bytes(&env, "ORDER_001"));
    assert_eq!(record.amount, 1000);
    assert_eq!(record.status, PaymentStatus::Completed);
}

#[test]
fn test_global_stats_overflow_fails() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = create_token(&env, &admin);

    client.set_admin(&admin);
    client.register_merchant(
        &merchant,
        &str(&env, "Store"),
        &str(&env, "desc"),
        &str(&env, "c@c.com "),
        &MerchantCategory::Retail,
        &None,
    );
    mint(&env, &token, &admin, &payer, i128::MAX);

    // Process a large payment
    let mut order = make_order(&env, &merchant, &payer, &token);
    order.amount = i128::MAX;
    let (pub_key, sig) = sign_order(&env, &order);
    client.process_payment_with_signature(&payer, &order, &sig);

    // Second payment should overflow total_volume
    order.order_id = bytes(&env, "ORDER_002");
    let (pub_key2, sig2) = sign_order(&env, &order);
    let result = client.try_process_payment_with_signature(&payer, &order, &sig2);
    assert_eq!(result, Err(Ok(PaymentError::ArithmeticError)));
}

#[test]
fn test_duplicate_payment_fails() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = create_token(&env, &admin);

    client.set_admin(&admin);
    client.register_merchant(
        &merchant,
        &str(&env, "Store"),
        &str(&env, "desc"),
        &str(&env, "c@c.com "),
        &MerchantCategory::Retail,
        &None,
    );
    mint(&env, &token, &admin, &payer, 5000);

    let order = make_order(&env, &merchant, &payer, &token);
    let (pub_key, sig) = sign_order(&env, &order);

    client.process_payment_with_signature(&payer, &order, &sig);
    let result = client.try_process_payment_with_signature(&payer, &order, &sig);
    assert_eq!(result, Err(Ok(PaymentError::PaymentAlreadyExists)));
}

#[test]
fn test_payment_expired_fails() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = create_token(&env, &admin);

    client.set_admin(&admin);
    client.register_merchant(
        &merchant,
        &str(&env, "Store"),
        &str(&env, "desc"),
        &str(&env, "c@c.com "),
        &MerchantCategory::Retail,
        &None,
    );
    mint(&env, &token, &admin, &payer, 5000);

    env.ledger().with_mut(|l| l.timestamp = 2000);

    let mut order = make_order(&env, &merchant, &payer, &token);
    order.expires_at = 1000; // already expired

    let (pub_key, sig) = sign_order(&env, &order);
    let result = client.try_process_payment_with_signature(&payer, &order, &sig);
    assert_eq!(result, Err(Ok(PaymentError::PaymentExpired)));
}

#[test]
fn test_signature_over_different_amount_fails() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = create_token(&env, &admin);

    client.set_admin(&admin);
    client.register_merchant(
        &merchant,
        &str(&env, "Store"),
        &str(&env, "desc"),
        &str(&env, "c@c.com"),
        &MerchantCategory::Retail,
        &None,
    );
    mint(&env, &token, &admin, &payer, 5000);

    let mut order = make_order(&env, &merchant, &payer, &token);
    let (pub_key, sig) = sign_order(&env, &order);

    // Change amount after signing
    order.amount = 2000;

    let result = client.try_process_payment_with_signature(&payer, &order, &sig);
    // In Soroban, ed25519_verify panics on failure, which try_... returns as HostError(Crypto, InvalidInput)
    // or just fails the contract call.
    assert!(result.is_err());
}

// ── Refund tests ──────────────────────────────────────────────────────────────

fn setup_paid_order(
    env: &Env,
    client: &PaymentContractClient,
) -> (Address, Address, Address, Address) {
    let admin = Address::generate(env);
    let merchant = Address::generate(env);
    let payer = Address::generate(env);
    let token = create_token(env, &admin);

    client.set_admin(&admin);
    client.register_merchant(
        &merchant,
        &str(env, "Store"),
        &str(env, "desc"),
        &str(env, "c@c.com "),
        &MerchantCategory::Retail,
        &None,
    );
    mint(env, &token, &admin, &payer, 5000);

    let order = make_order(env, &merchant, &payer, &token);
    let (pub_key, sig) = sign_order(env, &order);
    client.process_payment_with_signature(&payer, &order, &sig);

    (admin, merchant, payer, token)
}

#[test]
fn test_successful_refund_flow() {
    let (env, client) = setup();
    let (_admin, merchant, payer, token) = setup_paid_order(&env, &client);

    let token_client = soroban_sdk::token::TokenClient::new(&env, &token);
    let payer_balance_before = token_client.balance(&payer);
    let merchant_balance_before = token_client.balance(&merchant);

    client.initiate_refund(
        &payer,
        &bytes(&env, "REFUND_001"),
        &bytes(&env, "ORDER_001"),
        &500,
        &str(&env, "Customer request "),
    );

    let status = client.get_refund_status(&bytes(&env, "REFUND_001"));
    assert_eq!(status, RefundStatus::Pending);

    client.approve_refund(&merchant, &bytes(&env, "REFUND_001"));
    let status = client.get_refund_status(&bytes(&env, "REFUND_001"));
    assert_eq!(status, RefundStatus::Approved);

    client.execute_refund(&merchant, &bytes(&env, "REFUND_001"));
    let status = client.get_refund_status(&bytes(&env, "REFUND_001"));
    assert_eq!(status, RefundStatus::Completed);

    let record = client.get_payment_by_id(&payer, &bytes(&env, "ORDER_001"));
    assert_eq!(record.refunded_amount, 500);
    assert_eq!(record.status, PaymentStatus::PartiallyRefunded);

    // Balance assertions: payer receives refund, merchant pays it
    assert_eq!(token_client.balance(&payer), payer_balance_before + 500);
    assert_eq!(token_client.balance(&merchant), merchant_balance_before - 500);
}

#[test]
fn test_full_refund_flow_with_balance_assertions() {
    let (env, client) = setup();
    let (_admin, merchant, payer, token) = setup_paid_order(&env, &client);

    let token_client = soroban_sdk::token::TokenClient::new(&env, &token);
    let payer_balance_before = token_client.balance(&payer);
    let merchant_balance_before = token_client.balance(&merchant);

    // Full refund of 1000
    client.initiate_refund(
        &payer,
        &bytes(&env, "REFUND_FULL"),
        &bytes(&env, "ORDER_001"),
        &1000,
        &str(&env, "Full refund"),
    );
    client.approve_refund(&merchant, &bytes(&env, "REFUND_FULL"));
    client.execute_refund(&merchant, &bytes(&env, "REFUND_FULL"));

    let record = client.get_payment_by_id(&payer, &bytes(&env, "ORDER_001"));
    assert_eq!(record.refunded_amount, 1000);
    assert_eq!(record.status, PaymentStatus::FullyRefunded);

    assert_eq!(token_client.balance(&payer), payer_balance_before + 1000);
    assert_eq!(token_client.balance(&merchant), merchant_balance_before - 1000);
}

#[test]
fn test_refund_reason_length_limit() {
    let (env, client) = setup();
    let (_admin, _merchant, payer, _token) = setup_paid_order(&env, &client);

    // 256 bytes: ok
    let reason_ok = "r".repeat(256);
    client.initiate_refund(
        &payer,
        &bytes(&env, "REFUND_OK"),
        &bytes(&env, "ORDER_001"),
        &100,
        &str(&env, &reason_ok),
    );

    // 257 bytes: fails
    let reason_bad = "r".repeat(257);
    let result = client.try_initiate_refund(
        &payer,
        &bytes(&env, "REFUND_BAD"),
        &bytes(&env, "ORDER_001"),
        &100,
        &str(&env, &reason_bad),
    );
    assert_eq!(result, Err(Ok(PaymentError::InvalidInput)));
}

#[test]
fn test_refund_exceeds_payment_fails() {
    let (env, client) = setup();
    let (_admin, _merchant, payer, _token) = setup_paid_order(&env, &client);

    let result = client.try_initiate_refund(
        &payer,
        &bytes(&env, "REFUND_001"),
        &bytes(&env, "ORDER_001"),
        &1500, // more than 1000
        &str(&env, "Too much "),
    );
    assert_eq!(result, Err(Ok(PaymentError::RefundAmountExceedsPayment)));
}

#[test]
fn test_refund_window_expired_fails() {
    let (env, client) = setup();
    let (_admin, _merchant, payer, _token) = setup_paid_order(&env, &client);

    // Advance time past 30-day refund window
    env.ledger().with_mut(|l| l.timestamp = 2_592_001);

    let result = client.try_initiate_refund(
        &payer,
        &bytes(&env, "REFUND_001"),
        &bytes(&env, "ORDER_001"),
        &500,
        &str(&env, "Late"),
    );
    assert_eq!(result, Err(Ok(PaymentError::RefundWindowExpired)));
}

#[test]
fn test_reject_refund() {
    let (env, client) = setup();
    let (_admin, merchant, payer, _token) = setup_paid_order(&env, &client);

    client.initiate_refund(
        &payer,
        &bytes(&env, "REFUND_001"),
        &bytes(&env, "ORDER_001"),
        &500,
        &str(&env, "Request"),
    );
    client.reject_refund(&merchant, &bytes(&env, "REFUND_001"));
    let status = client.get_refund_status(&bytes(&env, "REFUND_001"));
    assert_eq!(status, RefundStatus::Rejected);
}

// ── Payment history tests ─────────────────────────────────────────────────────

#[test]
fn test_get_merchant_payment_history() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = create_token(&env, &admin);

    client.set_admin(&admin);
    client.register_merchant(
        &merchant,
        &str(&env, "Store"),
        &str(&env, "desc"),
        &str(&env, "c@c.com "),
        &MerchantCategory::Retail,
        &None,
    );
    mint(&env, &token, &admin, &payer, 10000);

    for (id, amount) in [("ORDER_001", 100i128), ("ORDER_002", 200), ("ORDER_003", 300)] {
        let order = PaymentOrder {
            order_id: bytes(&env, id),
            merchant_address: merchant.clone(),
            payer: payer.clone(),
            token: token.clone(),
            amount,
            description: str(&env, "desc"),
            expires_at: 0,
        };
        let (pub_key, sig) = sign_order(&env, &order);
        client.process_payment_with_signature(&payer, &order, &sig);
    }

    let page = client.get_merchant_payment_history(
        &merchant,
        &None,
        &10,
        &None,
        &SortField::Amount,
        &SortOrder::Descending,
    );
    assert_eq!(page.total, 3);
    assert_eq!(page.records.get(0).unwrap().amount, 300);
}

#[test]
fn test_execute_refund_unauthorized_fails() {
    let (env, client) = setup();
    let (_admin, merchant, payer, _token) = setup_paid_order(&env, &client);

    client.initiate_refund(&payer, &bytes(&env, "R1"), &bytes(&env, "ORDER_001"), &500, &str(&env, "reason"));
    client.approve_refund(&merchant, &bytes(&env, "R1"));

    let other = Address::generate(&env);
    let result = client.try_execute_refund(&other, &bytes(&env, "R1"));
    assert_eq!(result, Err(Ok(PaymentError::Unauthorized)));
}

// ── Multisig tests ────────────────────────────────────────────────────────────

#[test]
fn test_initiate_multisig_payment_success() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let signer1 = Address::generate(&env);
    let signer2 = Address::generate(&env);
    let token = create_token(&env, &admin);

    client.set_admin(&admin);
    client.register_merchant(
        &merchant,
        &str(&env, "Store"),
        &str(&env, "desc"),
        &str(&env, "c@c.com "),
        &MerchantCategory::Retail,
        &None,
    );
    mint(&env, &token, &admin, &signer1, 5000);

    let order = PaymentOrder {
        order_id: bytes(&env, "MS_001"),
        merchant_address: merchant.clone(),
        payer: signer1.clone(),
        token: token.clone(),
        amount: 1000,
        description: str(&env, "Multisig order "),
        expires_at: 0,
    };

    let mut signers = Vec::new(&env);
    signers.push_back(signer1.clone());
    signers.push_back(signer2.clone());

    client.initiate_multisig_payment(&signer1, &bytes(&env, "MS_001"), &order, &signers);
    client.sign_multisig_payment(&signer1, &bytes(&env, "MS_001"));
    client.sign_multisig_payment(&signer2, &bytes(&env, "MS_001"));
    client.execute_multisig_payment(&signer1, &bytes(&env, "MS_001"));

    let record = client.get_payment_by_id(&signer1, &bytes(&env, "MS_001"));
    assert_eq!(record.amount, 1000);
    assert_eq!(record.status, PaymentStatus::Completed);
}

#[test]
fn test_multisig_insufficient_signatures_fails() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let signer1 = Address::generate(&env);
    let signer2 = Address::generate(&env);
    let token = create_token(&env, &admin);

    client.set_admin(&admin);
    client.register_merchant(
        &merchant,
        &str(&env, "Store"),
        &str(&env, "desc"),
        &str(&env, "c@c.com "),
        &MerchantCategory::Retail,
        &None,
    );
    mint(&env, &token, &admin, &signer1, 5000);

    let order = PaymentOrder {
        order_id: bytes(&env, "MS_002"),
        merchant_address: merchant.clone(),
        payer: signer1.clone(),
        token: token.clone(),
        amount: 1000,
        description: str(&env, "Multisig order "),
        expires_at: 0,
    };

    let mut signers = Vec::new(&env);
    signers.push_back(signer1.clone());
    signers.push_back(signer2.clone());

    client.initiate_multisig_payment(&signer1, &bytes(&env, "MS_002"), &order, &signers);
    client.sign_multisig_payment(&signer1, &bytes(&env, "MS_002"));

    let result = client.try_execute_multisig_payment(&signer1, &bytes(&env, "MS_002"));
    assert_eq!(result, Err(Ok(PaymentError::InsufficientSignatures)));
}

#[test]
fn test_initiate_multisig_duplicate_signer_fails() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let signer1 = Address::generate(&env);
    let token = create_token(&env, &admin);

    client.set_admin(&admin);
    client.register_merchant(
        &merchant,
        &str(&env, "Store"),
        &str(&env, "desc"),
        &str(&env, "c@c.com"),
        &MerchantCategory::Retail,
    );
    mint(&env, &token, &admin, &signer1, 5000);

    let order = PaymentOrder {
        order_id: bytes(&env, "MS_DUP"),
        merchant_address: merchant.clone(),
        payer: signer1.clone(),
        token: token.clone(),
        amount: 1000,
        description: str(&env, "Multisig order"),
        expires_at: 0,
    };

    let mut signers = Vec::new(&env);
    signers.push_back(signer1.clone());
    signers.push_back(signer1.clone()); // Duplicate

    let result = client.try_initiate_multisig_payment(&signer1, &bytes(&env, "MS_DUP"), &order, &signers);
    assert_eq!(result, Err(Ok(PaymentError::InvalidInput)));
}

// ── Admin config tests ────────────────────────────────────────────────────────

// ── Whitelist tests ───────────────────────────────────────────────────────────

#[test]
fn test_whitelist_mode_blocks_unregistered_merchant() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    client.set_admin(&admin);

    client.set_whitelist_mode(&admin, &true);

    let result = client.try_register_merchant(
        &merchant,
        &str(&env, "Store"),
        &str(&env, "desc"),
        &str(&env, "c@c.com"),
        &MerchantCategory::Retail,
        &None,
    );
    assert_eq!(result, Err(Ok(PaymentError::Unauthorized)));
}

#[test]
fn test_whitelist_mode_allows_approved_merchant() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    client.set_admin(&admin);

    client.set_whitelist_mode(&admin, &true);
    client.approve_merchant_registration(&admin, &merchant);

    client.register_merchant(
        &merchant,
        &str(&env, "Store"),
        &str(&env, "desc"),
        &str(&env, "c@c.com"),
        &MerchantCategory::Retail,
        &None,
    );
    let m = client.get_merchant(&merchant);
    assert!(m.active);
}

#[test]
fn test_whitelist_disabled_allows_open_registration() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    client.set_admin(&admin);

    // Whitelist mode off by default
    client.register_merchant(
        &merchant,
        &str(&env, "Store"),
        &str(&env, "desc"),
        &str(&env, "c@c.com"),
        &MerchantCategory::Retail,
        &None,
    );
    let m = client.get_merchant(&merchant);
    assert!(m.active);
}

#[test]
fn test_set_whitelist_mode_non_admin_fails() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let other = Address::generate(&env);
    client.set_admin(&admin);

    let result = client.try_set_whitelist_mode(&other, &true);
    assert_eq!(result, Err(Ok(PaymentError::Unauthorized)));
}



#[test]
fn test_archive_payment_record_removes_from_indexes() {
    let (env, client) = setup();
    let (admin, merchant, payer, _token) = setup_paid_order(&env, &client);

    // Verify payment exists and appears in history
    let page = client.get_merchant_payment_history(
        &merchant,
        &None,
        &10,
        &None,
        &SortField::Date,
        &SortOrder::Ascending,
    );
    assert_eq!(page.total, 1);

    // Archive the payment
    client.archive_payment_record(&admin, &bytes(&env, "ORDER_001"));

    // Payment should no longer be retrievable
    let result = client.try_get_payment_by_id(&payer, &bytes(&env, "ORDER_001"));
    assert_eq!(result, Err(Ok(PaymentError::PaymentNotFound)));

    // Merchant history should be empty
    let page = client.get_merchant_payment_history(
        &merchant,
        &None,
        &10,
        &None,
        &SortField::Date,
        &SortOrder::Ascending,
    );
    assert_eq!(page.total, 0);

    // Payer history should be empty
    let page = client.get_payer_payment_history(
        &payer,
        &None,
        &10,
        &None,
        &SortField::Date,
        &SortOrder::Ascending,
    );
    assert_eq!(page.total, 0);
}

#[test]
fn test_archive_payment_record_non_admin_fails() {
    let (env, client) = setup();
    let (_admin, _merchant, payer, _token) = setup_paid_order(&env, &client);

    let result = client.try_archive_payment_record(&payer, &bytes(&env, "ORDER_001"));
    assert_eq!(result, Err(Ok(PaymentError::Unauthorized)));
}

#[test]
fn test_archive_payment_record_not_found_fails() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    client.set_admin(&admin);

    let result = client.try_archive_payment_record(&admin, &bytes(&env, "NONEXISTENT"));
    assert_eq!(result, Err(Ok(PaymentError::PaymentNotFound)));
}



#[test]
fn test_set_cleanup_period() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    client.set_admin(&admin);
    client.set_payment_cleanup_period(&admin, &86400);
}

#[test]
fn test_get_global_stats_with_filtering() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = create_token(&env, &admin);

    client.set_admin(&admin);
    client.register_merchant(
        &merchant,
        &str(&env, "Store"),
        &str(&env, "desc"),
        &str(&env, "c@c.com"),
        &MerchantCategory::Retail,
        &None,
    );
    mint(&env, &token, &admin, &payer, 10000);

    // Payment 1: t=1000
    env.ledger().with_mut(|l| l.timestamp = 1000);
    let order1 = PaymentOrder {
        order_id: bytes(&env, "ORDER_001"),
        merchant_address: merchant.clone(),
        payer: payer.clone(),
        token: token.clone(),
        amount: 1000,
        description: str(&env, "p1"),
        expires_at: 0,
    };
    let (pk1, sig1) = sign_order(&env, &order1);
    client.process_payment_with_signature(&payer, &order1, &sig1);

    // Payment 2: t=2000
    env.ledger().with_mut(|l| l.timestamp = 2000);
    let order2 = PaymentOrder {
        order_id: bytes(&env, "ORDER_002"),
        merchant_address: merchant.clone(),
        payer: payer.clone(),
        token: token.clone(),
        amount: 2000,
        description: str(&env, "p2"),
        expires_at: 0,
    };
    let (pk2, sig2) = sign_order(&env, &order2);
    client.process_payment_with_signature(&payer, &order2, &sig2);

    // Refund for Payment 1: initiated at t=3000, executed at t=4000
    env.ledger().with_mut(|l| l.timestamp = 3000);
    client.initiate_refund(&payer, &bytes(&env, "R1"), &bytes(&env, "ORDER_001"), &500, &str(&env, "reason"));
    client.approve_refund(&merchant, &bytes(&env, "R1"));
    env.ledger().with_mut(|l| l.timestamp = 4000);
    client.execute_refund(&merchant, &bytes(&env, "R1"));

    // Unfiltered
    let stats = client.get_global_payment_stats(&admin, &None, &None);
    assert_eq!(stats.total_payments, 2);
    assert_eq!(stats.total_volume, 3000);
    assert_eq!(stats.total_refunds, 1);
    assert_eq!(stats.total_refund_volume, 500);

    // Filtered: only t=1000 to t=1500 (Payment 1 only)
    let stats = client.get_global_payment_stats(&admin, &Some(500), &Some(1500));
    assert_eq!(stats.total_payments, 1);
    assert_eq!(stats.total_volume, 1000);
    assert_eq!(stats.total_refunds, 0);

    // Filtered: only t=2500 to t=3500 (Refund 1 only, because initiated_at=3000)
    let stats = client.get_global_payment_stats(&admin, &Some(2500), &Some(3500));
    assert_eq!(stats.total_payments, 0);
    assert_eq!(stats.total_refunds, 1);
    assert_eq!(stats.total_refund_volume, 500);
}

#[test]
fn test_update_payment_status_emits_event() {
    let (env, client) = setup();
    let (_admin, merchant, _payer, _token) = setup_paid_order(&env, &client);

    client.update_payment_status(&merchant, &bytes(&env, "ORDER_001"), &500);

    let events = env.events().all();
    let last_event = events.get(events.len() - 1).unwrap();

    // Check topics
    let topics = last_event.1;
    assert_eq!(topics.len(), 1);
    let topic: String = topics.get(0).unwrap().into_val(&env);
    assert_eq!(topic, str(&env, "payment_status_updated"));

    // Check data
    let (order_id, status, caller): (Bytes, PaymentStatus, Address) = last_event.2.into_val(&env);
    assert_eq!(order_id, bytes(&env, "ORDER_001"));
    assert_eq!(status, PaymentStatus::PartiallyRefunded);
    assert_eq!(caller, merchant);
}

#[test]
fn test_cleanup_expired_payments() {
    let (env, client) = setup();
    let (admin, merchant, payer, token) = setup_paid_order(&env, &client);

    // Default cleanup is 90 days. Set to 1h for test.
    client.set_payment_cleanup_period(&admin, &3600);

    // Both payments exist (one from setup_paid_order)
    assert!(client.try_get_payment_by_id(&payer, &bytes(&env, "ORDER_001")).is_ok());

    // Create another payment
    let order2 = PaymentOrder {
        order_id: bytes(&env, "ORDER_002"),
        merchant_address: merchant.clone(),
        payer: payer.clone(),
        token: token.clone(),
        amount: 500,
        description: str(&env, "desc"),
        expires_at: 0,
    };
    let pub_key = BytesN::from_array(&env, &[0u8; 32]);
    let sig = BytesN::from_array(&env, &[0u8; 64]);
    client.process_payment_with_signature(&payer, &order2, &sig, &pub_key);

    assert!(client.try_get_payment_by_id(&payer, &bytes(&env, "ORDER_002")).is_ok());

    // Fast forward 2h
    env.ledger().set_timestamp(7201);

    // Cleanup should remove both
    let count = client.cleanup_expired_payments(&admin);
    assert_eq!(count, 2);

    // Payments should be gone
    let result = client.try_get_payment_by_id(&payer, &bytes(&env, "ORDER_001"));
    assert_eq!(result, Err(Ok(PaymentError::PaymentNotFound)));
    let result = client.try_get_payment_by_id(&payer, &bytes(&env, "ORDER_002"));
    assert_eq!(result, Err(Ok(PaymentError::PaymentNotFound)));
}

#[test]
fn test_multisig_payment_expiry() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let signer1 = Address::generate(&env);
    let token = create_token(&env, &admin);

    client.set_admin(&admin);
    client.register_merchant(
        &merchant,
        &str(&env, "Store"),
        &str(&env, "desc"),
        &str(&env, "c@c.com "),
        &MerchantCategory::Retail,
        &None,
    );

    mint(&env, &token, &admin, &signer1, 5000);

    let order = make_order(&env, &merchant, &signer1, &token);
    let mut signers = Vec::new(&env);
    signers.push_back(signer1.clone());

    client.initiate_multisig_payment(&signer1, &bytes(&env, "MS_EXPIRY"), &order, &signers);

    // Fast forward 25h (default is 24h)
    env.ledger().set_timestamp(86400 + 3601);

    let result = client.try_sign_multisig_payment(&signer1, &bytes(&env, "MS_EXPIRY"));
    assert_eq!(result, Err(Ok(PaymentError::PaymentExpired)));

    let result = client.try_execute_multisig_payment(&signer1, &bytes(&env, "MS_EXPIRY"));
    assert_eq!(result, Err(Ok(PaymentError::PaymentExpired)));
}

// ── T-003: get_payer_payment_history tests ────────────────────────────────────

fn setup_payer_history(
    env: &Env,
    client: &PaymentContractClient,
    amounts: &[i128],
) -> (Address, Address, Address, Address) {
    let admin = Address::generate(env);
    let merchant = Address::generate(env);
    let payer = Address::generate(env);
    let token = create_token(env, &admin);

    client.set_admin(&admin);
    client.register_merchant(
        &merchant,
        &str(env, "Store"),
        &str(env, "desc"),
        &str(env, "c@c.com"),
        &MerchantCategory::Retail,
        &None,
    );
    let total: i128 = amounts.iter().sum::<i128>() + 1000;
    mint(env, &token, &admin, &payer, total);

    for (i, &amount) in amounts.iter().enumerate() {
        let id = alloc::format!("PAY_{:03}", i);
        let order = PaymentOrder {
            order_id: Bytes::from_slice(env, id.as_bytes()),
            merchant_address: merchant.clone(),
            payer: payer.clone(),
            token: token.clone(),
            amount,
            description: str(env, "desc"),
            expires_at: 0,
        };
        let (_pk, sig) = sign_order(env, &order);
        client.process_payment_with_signature(&payer, &order, &sig);
    }

    (admin, merchant, payer, token)
}

#[test]
fn test_payer_history_no_payments() {
    let (env, client) = setup();
    let payer = Address::generate(&env);
    // Register payer as a merchant so auth works; payer has no payments
    let admin = Address::generate(&env);
    client.set_admin(&admin);

    let page = client.get_payer_payment_history(
        &payer,
        &None,
        &10,
        &None,
        &SortField::Date,
        &SortOrder::Descending,
    );
    assert_eq!(page.total, 0);
    assert_eq!(page.records.len(), 0);
    assert!(page.next_cursor.is_none());
}

#[test]
fn test_payer_history_single_payment() {
    let (env, client) = setup();
    let (_admin, _merchant, payer, _token) = setup_payer_history(&env, &client, &[500]);

    let page = client.get_payer_payment_history(
        &payer,
        &None,
        &10,
        &None,
        &SortField::Date,
        &SortOrder::Ascending,
    );
    assert_eq!(page.total, 1);
    assert_eq!(page.records.get(0).unwrap().amount, 500);
    assert!(page.next_cursor.is_none());
}

#[test]
fn test_payer_history_multiple_payments() {
    let (env, client) = setup();
    let (_admin, _merchant, payer, _token) =
        setup_payer_history(&env, &client, &[100, 200, 300]);

    let page = client.get_payer_payment_history(
        &payer,
        &None,
        &10,
        &None,
        &SortField::Amount,
        &SortOrder::Ascending,
    );
    assert_eq!(page.total, 3);
    assert_eq!(page.records.get(0).unwrap().amount, 100);
    assert_eq!(page.records.get(1).unwrap().amount, 200);
    assert_eq!(page.records.get(2).unwrap().amount, 300);
}

#[test]
fn test_payer_history_filter_date_range() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = create_token(&env, &admin);

    client.set_admin(&admin);
    client.register_merchant(
        &merchant,
        &str(&env, "Store"),
        &str(&env, "desc"),
        &str(&env, "c@c.com"),
        &MerchantCategory::Retail,
        &None,
    );
    mint(&env, &token, &admin, &payer, 10000);

    env.ledger().with_mut(|l| l.timestamp = 1000);
    let o1 = PaymentOrder {
        order_id: bytes(&env, "D_001"),
        merchant_address: merchant.clone(),
        payer: payer.clone(),
        token: token.clone(),
        amount: 100,
        description: str(&env, "d"),
        expires_at: 0,
    };
    let (_pk, sig) = sign_order(&env, &o1);
    client.process_payment_with_signature(&payer, &o1, &sig);

    env.ledger().with_mut(|l| l.timestamp = 5000);
    let o2 = PaymentOrder {
        order_id: bytes(&env, "D_002"),
        merchant_address: merchant.clone(),
        payer: payer.clone(),
        token: token.clone(),
        amount: 200,
        description: str(&env, "d"),
        expires_at: 0,
    };
    let (_pk2, sig2) = sign_order(&env, &o2);
    client.process_payment_with_signature(&payer, &o2, &sig2);

    use crate::types::{PaymentFilter, StatusFilter};
    let filter = PaymentFilter {
        date_start: Some(500),
        date_end: Some(2000),
        amount_min: None,
        amount_max: None,
        token: None,
        status: StatusFilter::Any,
    };
    let page = client.get_payer_payment_history(
        &payer,
        &None,
        &10,
        &Some(filter),
        &SortField::Date,
        &SortOrder::Ascending,
    );
    assert_eq!(page.total, 1);
    assert_eq!(page.records.get(0).unwrap().amount, 100);
}

#[test]
fn test_payer_history_filter_amount_range() {
    let (env, client) = setup();
    let (_admin, _merchant, payer, _token) =
        setup_payer_history(&env, &client, &[50, 150, 500]);

    use crate::types::{PaymentFilter, StatusFilter};
    let filter = PaymentFilter {
        date_start: None,
        date_end: None,
        amount_min: Some(100),
        amount_max: Some(200),
        token: None,
        status: StatusFilter::Any,
    };
    let page = client.get_payer_payment_history(
        &payer,
        &None,
        &10,
        &Some(filter),
        &SortField::Amount,
        &SortOrder::Ascending,
    );
    assert_eq!(page.total, 1);
    assert_eq!(page.records.get(0).unwrap().amount, 150);
}

#[test]
fn test_payer_history_filter_by_token() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let token1 = create_token(&env, &admin);
    let token2 = create_token(&env, &admin);

    client.set_admin(&admin);
    client.register_merchant(
        &merchant,
        &str(&env, "Store"),
        &str(&env, "desc"),
        &str(&env, "c@c.com"),
        &MerchantCategory::Retail,
        &None,
    );
    mint(&env, &token1, &admin, &payer, 5000);
    mint(&env, &token2, &admin, &payer, 5000);

    let o1 = PaymentOrder {
        order_id: bytes(&env, "T_001"),
        merchant_address: merchant.clone(),
        payer: payer.clone(),
        token: token1.clone(),
        amount: 100,
        description: str(&env, "d"),
        expires_at: 0,
    };
    let (_pk, sig) = sign_order(&env, &o1);
    client.process_payment_with_signature(&payer, &o1, &sig);

    let o2 = PaymentOrder {
        order_id: bytes(&env, "T_002"),
        merchant_address: merchant.clone(),
        payer: payer.clone(),
        token: token2.clone(),
        amount: 200,
        description: str(&env, "d"),
        expires_at: 0,
    };
    let (_pk2, sig2) = sign_order(&env, &o2);
    client.process_payment_with_signature(&payer, &o2, &sig2);

    use crate::types::{PaymentFilter, StatusFilter};
    let filter = PaymentFilter {
        date_start: None,
        date_end: None,
        amount_min: None,
        amount_max: None,
        token: Some(token2.clone()),
        status: StatusFilter::Any,
    };
    let page = client.get_payer_payment_history(
        &payer,
        &None,
        &10,
        &Some(filter),
        &SortField::Date,
        &SortOrder::Ascending,
    );
    assert_eq!(page.total, 1);
    assert_eq!(page.records.get(0).unwrap().token, token2);
}

#[test]
fn test_payer_history_filter_by_status() {
    let (env, client) = setup();
    let (_admin, merchant, payer, _token) =
        setup_payer_history(&env, &client, &[300, 400]);

    // Initiate + approve + execute a partial refund on PAY_000
    client.initiate_refund(
        &payer,
        &bytes(&env, "RF_001"),
        &bytes(&env, "PAY_000"),
        &100,
        &str(&env, "partial"),
    );
    client.approve_refund(&merchant, &bytes(&env, "RF_001"));
    client.execute_refund(&bytes(&env, "RF_001"));

    use crate::types::{PaymentFilter, StatusFilter};
    let filter = PaymentFilter {
        date_start: None,
        date_end: None,
        amount_min: None,
        amount_max: None,
        token: None,
        status: StatusFilter::PartiallyRefunded,
    };
    let page = client.get_payer_payment_history(
        &payer,
        &None,
        &10,
        &Some(filter),
        &SortField::Date,
        &SortOrder::Ascending,
    );
    assert_eq!(page.total, 1);
    assert_eq!(page.records.get(0).unwrap().status, PaymentStatus::PartiallyRefunded);
}

#[test]
fn test_payer_history_sort_by_amount_ascending() {
    let (env, client) = setup();
    let (_admin, _merchant, payer, _token) =
        setup_payer_history(&env, &client, &[300, 100, 200]);

    let page = client.get_payer_payment_history(
        &payer,
        &None,
        &10,
        &None,
        &SortField::Amount,
        &SortOrder::Ascending,
    );
    assert_eq!(page.records.get(0).unwrap().amount, 100);
    assert_eq!(page.records.get(2).unwrap().amount, 300);
}

#[test]
fn test_payer_history_sort_by_amount_descending() {
    let (env, client) = setup();
    let (_admin, _merchant, payer, _token) =
        setup_payer_history(&env, &client, &[300, 100, 200]);

    let page = client.get_payer_payment_history(
        &payer,
        &None,
        &10,
        &None,
        &SortField::Amount,
        &SortOrder::Descending,
    );
    assert_eq!(page.records.get(0).unwrap().amount, 300);
    assert_eq!(page.records.get(2).unwrap().amount, 100);
}

#[test]
fn test_payer_history_sort_by_date_ascending() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = create_token(&env, &admin);

    client.set_admin(&admin);
    client.register_merchant(
        &merchant,
        &str(&env, "Store"),
        &str(&env, "desc"),
        &str(&env, "c@c.com"),
        &MerchantCategory::Retail,
        &None,
    );
    mint(&env, &token, &admin, &payer, 10000);

    for (i, ts) in [(0usize, 3000u64), (1, 1000), (2, 2000)] {
        env.ledger().with_mut(|l| l.timestamp = ts);
        let id = alloc::format!("SD_{:03}", i);
        let order = PaymentOrder {
            order_id: Bytes::from_slice(&env, id.as_bytes()),
            merchant_address: merchant.clone(),
            payer: payer.clone(),
            token: token.clone(),
            amount: 100,
            description: str(&env, "d"),
            expires_at: 0,
        };
        let (_pk, sig) = sign_order(&env, &order);
        client.process_payment_with_signature(&payer, &order, &sig);
    }

    let page = client.get_payer_payment_history(
        &payer,
        &None,
        &10,
        &None,
        &SortField::Date,
        &SortOrder::Ascending,
    );
    assert_eq!(page.records.get(0).unwrap().paid_at, 1000);
    assert_eq!(page.records.get(2).unwrap().paid_at, 3000);
}

#[test]
fn test_payer_history_sort_by_date_descending() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = create_token(&env, &admin);

    client.set_admin(&admin);
    client.register_merchant(
        &merchant,
        &str(&env, "Store"),
        &str(&env, "desc"),
        &str(&env, "c@c.com"),
        &MerchantCategory::Retail,
        &None,
    );
    mint(&env, &token, &admin, &payer, 10000);

    for (i, ts) in [(0usize, 1000u64), (1, 3000), (2, 2000)] {
        env.ledger().with_mut(|l| l.timestamp = ts);
        let id = alloc::format!("DD_{:03}", i);
        let order = PaymentOrder {
            order_id: Bytes::from_slice(&env, id.as_bytes()),
            merchant_address: merchant.clone(),
            payer: payer.clone(),
            token: token.clone(),
            amount: 100,
            description: str(&env, "d"),
            expires_at: 0,
        };
        let (_pk, sig) = sign_order(&env, &order);
        client.process_payment_with_signature(&payer, &order, &sig);
    }

    let page = client.get_payer_payment_history(
        &payer,
        &None,
        &10,
        &None,
        &SortField::Date,
        &SortOrder::Descending,
    );
    assert_eq!(page.records.get(0).unwrap().paid_at, 3000);
    assert_eq!(page.records.get(2).unwrap().paid_at, 1000);
}

#[test]
fn test_payer_history_pagination() {
    let (env, client) = setup();
    let (_admin, _merchant, payer, _token) =
        setup_payer_history(&env, &client, &[100, 200, 300, 400, 500]);

    // Page 1: limit 2
    let page1 = client.get_payer_payment_history(
        &payer,
        &None,
        &2,
        &None,
        &SortField::Amount,
        &SortOrder::Ascending,
    );
    assert_eq!(page1.records.len(), 2);
    assert!(page1.next_cursor.is_some());

    // Page 2: use cursor from page 1
    let page2 = client.get_payer_payment_history(
        &payer,
        &page1.next_cursor,
        &2,
        &None,
        &SortField::Amount,
        &SortOrder::Ascending,
    );
    assert_eq!(page2.records.len(), 2);

    // Amounts on page 2 must be greater than those on page 1
    let max_p1 = page1.records.get(1).unwrap().amount;
    let min_p2 = page2.records.get(0).unwrap().amount;
    assert!(min_p2 > max_p1);
}
