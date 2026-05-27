#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Events as _, Ledger},
    token::StellarAssetClient,
    Address, Bytes, Env, IntoVal, String, Vec,
};

use crate::{
    error::PaymentError,
    types::{MerchantCategory, PaymentOrder, PaymentStatus, RefundStatus, SortField, SortOrder},
    PaymentContract, PaymentContractClient,
};

// ── Test helpers ──────────────────────────────────────────────────────────────

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

fn mint(env: &Env, token: &Address, admin: &Address, to: &Address, amount: i128) {
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
    );
    let result = client.try_register_merchant(
        &merchant,
        &str(&env, "Store"),
        &str(&env, "desc"),
        &str(&env, "c@c.com "),
        &MerchantCategory::Retail,
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
    );
    client.deactivate_merchant(&admin, &merchant);
    let m = client.get_merchant(&merchant);
    assert!(!m.active);
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
    );
    mint(&env, &token, &admin, &payer, 5000);

    let order = make_order(&env, &merchant, &payer, &token);
    // Use a dummy 32-byte public key and 64-byte signature (mock_all_auths bypasses crypto)
    let pub_key = Bytes::from_array(&env, &[0u8; 32]);
    let sig = Bytes::from_array(&env, &[0u8; 64]);

    client.process_payment_with_signature(&payer, &order, &sig, &pub_key);

    let record = client.get_payment_by_id(&payer, &bytes(&env, "ORDER_001"));
    assert_eq!(record.amount, 1000);
    assert_eq!(record.status, PaymentStatus::Completed);
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
    );
    mint(&env, &token, &admin, &payer, 5000);

    let order = make_order(&env, &merchant, &payer, &token);
    let pub_key = Bytes::from_array(&env, &[0u8; 32]);
    let sig = Bytes::from_array(&env, &[0u8; 64]);

    client.process_payment_with_signature(&payer, &order, &sig, &pub_key);
    let result = client.try_process_payment_with_signature(&payer, &order, &sig, &pub_key);
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
    );
    mint(&env, &token, &admin, &payer, 5000);

    env.ledger().with_mut(|l| l.timestamp = 2000);

    let mut order = make_order(&env, &merchant, &payer, &token);
    order.expires_at = 1000; // already expired

    let pub_key = Bytes::from_array(&env, &[0u8; 32]);
    let sig = Bytes::from_array(&env, &[0u8; 64]);
    let result = client.try_process_payment_with_signature(&payer, &order, &sig, &pub_key);
    assert_eq!(result, Err(Ok(PaymentError::PaymentExpired)));
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
    );
    mint(env, &token, &admin, &payer, 5000);

    let order = make_order(env, &merchant, &payer, &token);
    let pub_key = Bytes::from_array(env, &[0u8; 32]);
    let sig = Bytes::from_array(env, &[0u8; 64]);
    client.process_payment_with_signature(&payer, &order, &sig, &pub_key);

    (admin, merchant, payer, token)
}

#[test]
fn test_successful_refund_flow() {
    let (env, client) = setup();
    let (admin, merchant, payer, _token) = setup_paid_order(&env, &client);

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

    client.execute_refund(&bytes(&env, "REFUND_001"));
    let status = client.get_refund_status(&bytes(&env, "REFUND_001"));
    assert_eq!(status, RefundStatus::Completed);

    let record = client.get_payment_by_id(&payer, &bytes(&env, "ORDER_001"));
    assert_eq!(record.refunded_amount, 500);
    assert_eq!(record.status, PaymentStatus::PartiallyRefunded);
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
    );
    mint(&env, &token, &admin, &payer, 10000);

    let pub_key = Bytes::from_array(&env, &[0u8; 32]);
    let sig = Bytes::from_array(&env, &[0u8; 64]);

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
        client.process_payment_with_signature(&payer, &order, &sig, &pub_key);
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

// ── Admin config tests ────────────────────────────────────────────────────────

#[test]
fn test_set_cleanup_period() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    client.set_admin(&admin);
    client.set_payment_cleanup_period(&admin, &86400);
}

#[test]
fn test_get_global_stats() {
    let (env, client) = setup();
    let (admin, _merchant, _payer, _token) = setup_paid_order(&env, &client);
    let stats = client.get_global_payment_stats(&admin, &None, &None);
    assert_eq!(stats.total_payments, 1);
    assert_eq!(stats.total_volume, 1000);
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
    let pub_key = Bytes::from_array(&env, &[0u8; 32]);
    let sig = Bytes::from_array(&env, &[0u8; 64]);
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
    );

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
