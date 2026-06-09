#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::StellarAssetClient,
    Address, BytesN, Env, String, Vec,
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
    let contract_id = env.register(PaymentContract, ());
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

fn make_order(env: &Env, merchant: &Address, payer: &Address, token: &Address) -> PaymentOrder {
    PaymentOrder {
        order_id: str(env, "ORDER_001"),
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
        &str(&env, "c@c.com"),
        &MerchantCategory::Retail,
    );
    let result = client.try_register_merchant(
        &merchant,
        &str(&env, "Store"),
        &str(&env, "desc"),
        &str(&env, "c@c.com"),
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
        &str(&env, "c@c.com"),
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
        &str(&env, "c@c.com"),
        &MerchantCategory::Retail,
    );
    mint(&env, &token, &admin, &payer, 5000);

    let order = make_order(&env, &merchant, &payer, &token);
    // Use a dummy 32-byte public key and 64-byte signature (mock_all_auths bypasses crypto)
    let pub_key = BytesN::from_array(&env, &[0u8; 32]);
    let sig = BytesN::from_array(&env, &[0u8; 64]);

    client.process_payment_with_signature(&payer, &order, &sig, &pub_key);

    let record = client.get_payment_by_id(&payer, &str(&env, "ORDER_001"));
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
        &str(&env, "c@c.com"),
        &MerchantCategory::Retail,
    );
    mint(&env, &token, &admin, &payer, 5000);

    let order = make_order(&env, &merchant, &payer, &token);
    let pub_key = BytesN::from_array(&env, &[0u8; 32]);
    let sig = BytesN::from_array(&env, &[0u8; 64]);

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
        &str(&env, "c@c.com"),
        &MerchantCategory::Retail,
    );
    mint(&env, &token, &admin, &payer, 5000);

    env.ledger().with_mut(|l| l.timestamp = 2000);

    let mut order = make_order(&env, &merchant, &payer, &token);
    order.expires_at = 1000; // already expired

    let pub_key = BytesN::from_array(&env, &[0u8; 32]);
    let sig = BytesN::from_array(&env, &[0u8; 64]);
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
        &str(env, "c@c.com"),
        &MerchantCategory::Retail,
    );
    mint(env, &token, &admin, &payer, 5000);

    let order = make_order(env, &merchant, &payer, &token);
    let pub_key = BytesN::from_array(env, &[0u8; 32]);
    let sig = BytesN::from_array(env, &[0u8; 64]);
    client.process_payment_with_signature(&payer, &order, &sig, &pub_key);

    (admin, merchant, payer, token)
}

#[test]
fn test_successful_refund_flow() {
    let (env, client) = setup();
    let (_admin, merchant, payer, _token) = setup_paid_order(&env, &client);

    client.initiate_refund(
        &payer,
        &str(&env, "REFUND_001"),
        &str(&env, "ORDER_001"),
        &500,
        &str(&env, "Customer request"),
    );

    let status = client.get_refund_status(&str(&env, "REFUND_001"));
    assert_eq!(status, RefundStatus::Pending);

    client.approve_refund(&merchant, &str(&env, "REFUND_001"));
    let status = client.get_refund_status(&str(&env, "REFUND_001"));
    assert_eq!(status, RefundStatus::Approved);

    client.execute_refund(&str(&env, "REFUND_001"));
    let status = client.get_refund_status(&str(&env, "REFUND_001"));
    assert_eq!(status, RefundStatus::Completed);

    let record = client.get_payment_by_id(&payer, &str(&env, "ORDER_001"));
    assert_eq!(record.refunded_amount, 500);
    assert_eq!(record.status, PaymentStatus::PartiallyRefunded);
}

#[test]
fn test_refund_exceeds_payment_fails() {
    let (env, client) = setup();
    let (_admin, _merchant, payer, _token) = setup_paid_order(&env, &client);

    let result = client.try_initiate_refund(
        &payer,
        &str(&env, "REFUND_001"),
        &str(&env, "ORDER_001"),
        &1500, // more than 1000
        &str(&env, "Too much"),
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
        &str(&env, "REFUND_001"),
        &str(&env, "ORDER_001"),
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
        &str(&env, "REFUND_001"),
        &str(&env, "ORDER_001"),
        &500,
        &str(&env, "Request"),
    );
    client.reject_refund(&merchant, &str(&env, "REFUND_001"));
    let status = client.get_refund_status(&str(&env, "REFUND_001"));
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
        &str(&env, "c@c.com"),
        &MerchantCategory::Retail,
    );
    mint(&env, &token, &admin, &payer, 10000);

    let pub_key = BytesN::from_array(&env, &[0u8; 32]);
    let sig = BytesN::from_array(&env, &[0u8; 64]);

    for (id, amount) in [("ORDER_001", 100i128), ("ORDER_002", 200), ("ORDER_003", 300)] {
        let order = PaymentOrder {
            order_id: str(&env, id),
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
        &str(&env, "c@c.com"),
        &MerchantCategory::Retail,
    );
    mint(&env, &token, &admin, &signer1, 5000);

    let order = PaymentOrder {
        order_id: str(&env, "MS_001"),
        merchant_address: merchant.clone(),
        payer: signer1.clone(),
        token: token.clone(),
        amount: 1000,
        description: str(&env, "Multisig order"),
        expires_at: 0,
    };

    let mut signers = Vec::new(&env);
    signers.push_back(signer1.clone());
    signers.push_back(signer2.clone());

    client.initiate_multisig_payment(&signer1, &str(&env, "MS_001"), &order, &signers);
    client.sign_multisig_payment(&signer1, &str(&env, "MS_001"));
    client.sign_multisig_payment(&signer2, &str(&env, "MS_001"));
    client.execute_multisig_payment(&signer1, &str(&env, "MS_001"));

    let record = client.get_payment_by_id(&signer1, &str(&env, "MS_001"));
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
        &str(&env, "c@c.com"),
        &MerchantCategory::Retail,
    );
    mint(&env, &token, &admin, &signer1, 5000);

    let order = PaymentOrder {
        order_id: str(&env, "MS_002"),
        merchant_address: merchant.clone(),
        payer: signer1.clone(),
        token: token.clone(),
        amount: 1000,
        description: str(&env, "Multisig order"),
        expires_at: 0,
    };

    let mut signers = Vec::new(&env);
    signers.push_back(signer1.clone());
    signers.push_back(signer2.clone());

    client.initiate_multisig_payment(&signer1, &str(&env, "MS_002"), &order, &signers);
    client.sign_multisig_payment(&signer1, &str(&env, "MS_002")); // only 1 of 2

    let result = client.try_execute_multisig_payment(&signer1, &str(&env, "MS_002"));
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
