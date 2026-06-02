#![cfg(test)]

extern crate alloc;
use alloc::vec;

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::StellarAssetClient,
    Address, Bytes, BytesN, Env, String, Vec,
};

use ed25519_dalek::{Signer, SigningKey};

use crate::{
    error::PaymentError,
    types::{
        MerchantCategory, PaymentFilter, PaymentOrder, PaymentStatus, RefundStatus, SortField,
        SortOrder, StatusFilter, SubscriptionPlan,
    },
    PaymentContract, PaymentContractClient,
};

use soroban_sdk::xdr::ToXdr;

// ── Test helpers ──────────────────────────────────────────────────────────────

fn sign_order(env: &Env, order: &PaymentOrder) -> (BytesN<32>, BytesN<64>) {
    let signing_key = SigningKey::from_bytes(&[1u8; 32]);
    let public_key = signing_key.verifying_key();
    let payload = order.order_id.clone();
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
    env.register_stellar_asset_contract_v2(admin.clone()).address()
}

fn mint(env: &Env, token: &Address, to: &Address, amount: i128) {
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

fn zero_key(env: &Env) -> BytesN<32> {
    BytesN::from_array(env, &[0u8; 32])
}

fn zero_sig(env: &Env) -> BytesN<64> {
    BytesN::from_array(env, &[0u8; 64])
}

fn admins(env: &Env, admin: &Address) -> Vec<Address> {
    vec![env, admin.clone()]
}

// ── Admin tests ───────────────────────────────────────────────────────────────

#[test]
fn test_set_admin_success() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    client.set_admin(&admins(&env, &admin), &1);
}

#[test]
fn test_set_admin_twice_fails() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    client.set_admin(&admins(&env, &admin), &1);
    let result = client.try_set_admin(&admins(&env, &admin), &1);
    assert_eq!(result, Err(Ok(PaymentError::AdminAlreadySet)));
}

#[test]
fn test_get_version_after_set_admin() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    client.set_admin(&admins(&env, &admin), &1);
    assert_eq!(client.get_version(), 1);
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
        &str(&env, "c@c.com"),
        &MerchantCategory::Retail,
        &None,
    );
    let result = client.try_register_merchant(
        &merchant,
        &str(&env, "Store"),
        &str(&env, "desc"),
        &str(&env, "c@c.com"),
        &MerchantCategory::Retail,
        &None,
    );
    assert_eq!(result, Err(Ok(PaymentError::MerchantAlreadyRegistered)));
}

#[test]
fn test_register_merchant_field_limits() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    client.set_admin(&admins(&env, &admin), &1);

    let m1 = Address::generate(&env);
    client.register_merchant(&m1, &str(&env, &"n".repeat(64)), &str(&env, "desc"), &str(&env, "c@c.com"), &MerchantCategory::Retail, &None);

    let m2 = Address::generate(&env);
    assert_eq!(
        client.try_register_merchant(&m2, &str(&env, &"n".repeat(65)), &str(&env, "desc"), &str(&env, "c@c.com"), &MerchantCategory::Retail, &None),
        Err(Ok(PaymentError::InvalidInput))
    );

    let m3 = Address::generate(&env);
    client.register_merchant(&m3, &str(&env, "Store"), &str(&env, &"d".repeat(256)), &str(&env, "c@c.com"), &MerchantCategory::Retail, &None);

    let m4 = Address::generate(&env);
    assert_eq!(
        client.try_register_merchant(&m4, &str(&env, "Store"), &str(&env, &"d".repeat(257)), &str(&env, "c@c.com"), &MerchantCategory::Retail, &None),
        Err(Ok(PaymentError::InvalidInput))
    );

    let m5 = Address::generate(&env);
    client.register_merchant(&m5, &str(&env, "Store"), &str(&env, "desc"), &str(&env, &"c".repeat(128)), &MerchantCategory::Retail, &None);

    let m6 = Address::generate(&env);
    assert_eq!(
        client.try_register_merchant(&m6, &str(&env, "Store"), &str(&env, "desc"), &str(&env, &"c".repeat(129)), &MerchantCategory::Retail, &None),
        Err(Ok(PaymentError::InvalidInput))
    );
}

#[test]
fn test_register_contact_info_sanitisation_rejects_control_chars() {
    let (env, client) = setup();
    let merchant = Address::generate(&env);
    let bad_contact = String::from_str(&env, "bad\x01contact");
    assert_eq!(
        client.try_register_merchant(&merchant, &str(&env, "Store"), &str(&env, "desc"), &bad_contact, &MerchantCategory::Retail, &None),
        Err(Ok(PaymentError::InvalidInput))
    );
}

#[test]
fn test_deactivate_merchant() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    client.set_admin(&admins(&env, &admin), &1);
    client.register_merchant(&merchant, &str(&env, "Store"), &str(&env, "desc"), &str(&env, "c@c.com"), &MerchantCategory::Retail, &None);
    client.deactivate_merchant(&merchant, &Some(admins(&env, &admin)));
    assert!(!client.get_merchant(&merchant).active);
}

// ── Payment tests ─────────────────────────────────────────────────────────────

#[test]
fn test_payment_payer_mismatch_fails() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let other_payer = Address::generate(&env);
    let token = create_token(&env, &admin);
    client.set_admin(&admins(&env, &admin), &1);
    client.register_merchant(&merchant, &str(&env, "Store"), &str(&env, "desc"), &str(&env, "c@c.com"), &MerchantCategory::Retail, &None);
    mint(&env, &token, &payer, 5000);
    let order = make_order(&env, &merchant, &other_payer, &token);
    let (_pk, sig) = sign_order(&env, &order);
    assert_eq!(
        client.try_process_payment_with_signature(&payer, &order, &sig, &zero_key(&env)),
        Err(Ok(PaymentError::InvalidInput))
    );
}

#[test]
fn test_successful_payment_with_signature() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = create_token(&env, &admin);
    client.set_admin(&admins(&env, &admin), &1);
    client.register_merchant(&merchant, &str(&env, "Store"), &str(&env, "desc"), &str(&env, "c@c.com"), &MerchantCategory::Retail, &None);
    mint(&env, &token, &payer, 5000);
    let order = make_order(&env, &merchant, &payer, &token);
    let (_pk, sig) = sign_order(&env, &order);
    client.process_payment_with_signature(&payer, &order, &sig, &zero_key(&env));
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
    client.set_admin(&admins(&env, &admin), &1);
    client.register_merchant(&merchant, &str(&env, "Store"), &str(&env, "desc"), &str(&env, "c@c.com"), &MerchantCategory::Retail, &None);
    mint(&env, &token, &payer, 5000);
    let order = make_order(&env, &merchant, &payer, &token);
    let (_pk, sig) = sign_order(&env, &order);
    client.process_payment_with_signature(&payer, &order, &sig, &zero_key(&env));
    assert_eq!(
        client.try_process_payment_with_signature(&payer, &order, &sig, &zero_key(&env)),
        Err(Ok(PaymentError::PaymentAlreadyExists))
    );
}

#[test]
fn test_payment_expired_fails() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = create_token(&env, &admin);
    client.set_admin(&admins(&env, &admin), &1);
    client.register_merchant(&merchant, &str(&env, "Store"), &str(&env, "desc"), &str(&env, "c@c.com"), &MerchantCategory::Retail, &None);
    mint(&env, &token, &payer, 5000);
    env.ledger().with_mut(|l| l.timestamp = 2000);
    let mut order = make_order(&env, &merchant, &payer, &token);
    order.expires_at = 1000;
    let (_pk, sig) = sign_order(&env, &order);
    assert_eq!(
        client.try_process_payment_with_signature(&payer, &order, &sig, &zero_key(&env)),
        Err(Ok(PaymentError::PaymentExpired))
    );
}

#[test]
fn test_global_stats_overflow_fails() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = create_token(&env, &admin);
    client.set_admin(&admins(&env, &admin), &1);
    client.register_merchant(&merchant, &str(&env, "Store"), &str(&env, "desc"), &str(&env, "c@c.com"), &MerchantCategory::Retail, &None);
    mint(&env, &token, &payer, i128::MAX);
    let mut order = make_order(&env, &merchant, &payer, &token);
    order.amount = i128::MAX;
    let (_pk, sig) = sign_order(&env, &order);
    client.process_payment_with_signature(&payer, &order, &sig, &zero_key(&env));
    order.order_id = bytes(&env, "ORDER_002");
    let (_pk2, sig2) = sign_order(&env, &order);
    assert_eq!(
        client.try_process_payment_with_signature(&payer, &order, &sig2, &zero_key(&env)),
        Err(Ok(PaymentError::ArithmeticError))
    );
}

// ── Refund tests ──────────────────────────────────────────────────────────────

fn setup_paid_order(env: &Env, client: &PaymentContractClient) -> (Address, Address, Address, Address) {
    let admin = Address::generate(env);
    let merchant = Address::generate(env);
    let payer = Address::generate(env);
    let token = create_token(env, &admin);
    client.set_admin(&admins(env, &admin), &1);
    client.register_merchant(&merchant, &str(env, "Store"), &str(env, "desc"), &str(env, "c@c.com"), &MerchantCategory::Retail, &None);
    mint(env, &token, &payer, 5000);
    let order = make_order(env, &merchant, &payer, &token);
    let (_pk, sig) = sign_order(env, &order);
    client.process_payment_with_signature(&payer, &order, &sig, &zero_key(env));
    (admin, merchant, payer, token)
}

#[test]
fn test_successful_refund_flow() {
    let (env, client) = setup();
    let (_admin, merchant, payer, token) = setup_paid_order(&env, &client);
    let token_client = soroban_sdk::token::TokenClient::new(&env, &token);
    let payer_before = token_client.balance(&payer);
    let merchant_before = token_client.balance(&merchant);
    client.initiate_refund(&payer, &bytes(&env, "REFUND_001"), &bytes(&env, "ORDER_001"), &500, &str(&env, "Customer request"));
    assert_eq!(client.get_refund_status(&bytes(&env, "REFUND_001")), RefundStatus::Pending);
    client.approve_refund(&merchant, &bytes(&env, "REFUND_001"), &None);
    assert_eq!(client.get_refund_status(&bytes(&env, "REFUND_001")), RefundStatus::Approved);
    client.execute_refund(&merchant, &bytes(&env, "REFUND_001"));
    assert_eq!(client.get_refund_status(&bytes(&env, "REFUND_001")), RefundStatus::Completed);
    let record = client.get_payment_by_id(&payer, &bytes(&env, "ORDER_001"));
    assert_eq!(record.refunded_amount, 500);
    assert_eq!(record.status, PaymentStatus::PartiallyRefunded);
    assert_eq!(token_client.balance(&payer), payer_before + 500);
    assert_eq!(token_client.balance(&merchant), merchant_before - 500);
}

#[test]
fn test_full_refund_flow_with_balance_assertions() {
    let (env, client) = setup();
    let (_admin, merchant, payer, token) = setup_paid_order(&env, &client);
    let token_client = soroban_sdk::token::TokenClient::new(&env, &token);
    let payer_before = token_client.balance(&payer);
    let merchant_before = token_client.balance(&merchant);
    client.initiate_refund(&payer, &bytes(&env, "REFUND_FULL"), &bytes(&env, "ORDER_001"), &1000, &str(&env, "Full refund"));
    client.approve_refund(&merchant, &bytes(&env, "REFUND_FULL"), &None);
    client.execute_refund(&merchant, &bytes(&env, "REFUND_FULL"));
    let record = client.get_payment_by_id(&payer, &bytes(&env, "ORDER_001"));
    assert_eq!(record.refunded_amount, 1000);
    assert_eq!(record.status, PaymentStatus::FullyRefunded);
    assert_eq!(token_client.balance(&payer), payer_before + 1000);
    assert_eq!(token_client.balance(&merchant), merchant_before - 1000);
}

#[test]
fn test_refund_reason_length_limit() {
    let (env, client) = setup();
    let (_admin, _merchant, payer, _token) = setup_paid_order(&env, &client);
    client.initiate_refund(&payer, &bytes(&env, "REFUND_OK"), &bytes(&env, "ORDER_001"), &100, &str(&env, &"r".repeat(256)));
    assert_eq!(
        client.try_initiate_refund(&payer, &bytes(&env, "REFUND_BAD"), &bytes(&env, "ORDER_001"), &100, &str(&env, &"r".repeat(257))),
        Err(Ok(PaymentError::InvalidInput))
    );
}

#[test]
fn test_approve_refund_unauthorized_fails() {
    let (env, client) = setup();
    let (_admin, _merchant, payer, _token) = setup_paid_order(&env, &client);
    let stranger = Address::generate(&env);
    client.initiate_refund(&payer, &bytes(&env, "REFUND_001"), &bytes(&env, "ORDER_001"), &500, &str(&env, "Customer request"));
    assert_eq!(
        client.try_approve_refund(&stranger, &bytes(&env, "REFUND_001"), &None),
        Err(Ok(PaymentError::Unauthorized))
    );
}

#[test]
fn test_initiate_refund_unauthorized_fails() {
    let (env, client) = setup();
    let (_admin, _merchant, payer, _token) = setup_paid_order(&env, &client);
    let stranger = Address::generate(&env);

    let result = client.try_initiate_refund(
        &stranger,
        &bytes(&env, "REFUND_001"),
        &bytes(&env, "ORDER_001"),
        &500,
        &str(&env, "Unauthorized refund attempt"),
    );

    assert_eq!(result, Err(Ok(PaymentError::Unauthorized)));
}

#[test]
fn test_refund_exceeds_payment_fails() {
    let (env, client) = setup();
    let (_admin, _merchant, payer, _token) = setup_paid_order(&env, &client);
    assert_eq!(
        client.try_initiate_refund(&payer, &bytes(&env, "REFUND_001"), &bytes(&env, "ORDER_001"), &1500, &str(&env, "Too much")),
        Err(Ok(PaymentError::RefundAmountExceedsPayment))
    );
}

#[test]
fn test_refund_window_expired_fails() {
    let (env, client) = setup();
    let (_admin, _merchant, payer, _token) = setup_paid_order(&env, &client);
    env.ledger().with_mut(|l| l.timestamp = 2_592_001);
    assert_eq!(
        client.try_initiate_refund(&payer, &bytes(&env, "REFUND_001"), &bytes(&env, "ORDER_001"), &500, &str(&env, "Late")),
        Err(Ok(PaymentError::RefundWindowExpired))
    );
}

#[test]
fn test_reject_refund() {
    let (env, client) = setup();
    let (_admin, merchant, payer, _token) = setup_paid_order(&env, &client);
    client.initiate_refund(&payer, &bytes(&env, "REFUND_001"), &bytes(&env, "ORDER_001"), &500, &str(&env, "Request"));
    client.reject_refund(&merchant, &bytes(&env, "REFUND_001"), &None);
    assert_eq!(client.get_refund_status(&bytes(&env, "REFUND_001")), RefundStatus::Rejected);
}

#[test]
fn test_execute_refund_unauthorized_fails() {
    let (env, client) = setup();
    let (_admin, merchant, payer, _token) = setup_paid_order(&env, &client);
    client.initiate_refund(&payer, &bytes(&env, "R1"), &bytes(&env, "ORDER_001"), &500, &str(&env, "reason"));
    client.approve_refund(&merchant, &bytes(&env, "R1"), &None);
    let other = Address::generate(&env);
    assert_eq!(
        client.try_execute_refund(&other, &bytes(&env, "R1")),
        Err(Ok(PaymentError::Unauthorized))
    );
}

#[test]
fn test_concurrent_refunds_within_limit_both_succeed() {
    let (env, client) = setup();
    let (_admin, merchant, payer, _token) = setup_paid_order(&env, &client);
    client.initiate_refund(&payer, &bytes(&env, "R_CONC_1"), &bytes(&env, "ORDER_001"), &600, &str(&env, "first"));
    client.initiate_refund(&payer, &bytes(&env, "R_CONC_2"), &bytes(&env, "ORDER_001"), &400, &str(&env, "second"));
    client.approve_refund(&merchant, &bytes(&env, "R_CONC_1"), &None);
    client.execute_refund(&merchant, &bytes(&env, "R_CONC_1"));
    client.approve_refund(&merchant, &bytes(&env, "R_CONC_2"), &None);
    client.execute_refund(&merchant, &bytes(&env, "R_CONC_2"));
    let record = client.get_payment_by_id(&payer, &bytes(&env, "ORDER_001"));
    assert_eq!(record.refunded_amount, 1000);
    assert_eq!(record.status, PaymentStatus::FullyRefunded);
}

#[test]
fn test_concurrent_refunds_exceeding_limit_second_rejected() {
    let (env, client) = setup();
    let (_admin, _merchant, payer, _token) = setup_paid_order(&env, &client);
    client.initiate_refund(&payer, &bytes(&env, "R_RACE_1"), &bytes(&env, "ORDER_001"), &700, &str(&env, "first"));
    assert_eq!(
        client.try_initiate_refund(&payer, &bytes(&env, "R_RACE_2"), &bytes(&env, "ORDER_001"), &400, &str(&env, "second")),
        Err(Ok(PaymentError::RefundAmountExceedsPayment))
    );
}

// ── Payment history tests ─────────────────────────────────────────────────────

#[test]
fn test_get_merchant_payment_history() {
    let (env, client) = setup();
    let (_admin, merchant, payer, token) = setup_paid_order(&env, &client);
    mint(&env, &token, &payer, 10000);
    for (id, amount) in [("ORDER_002", 200i128), ("ORDER_003", 300)] {
        let order = PaymentOrder {
            order_id: bytes(&env, id),
            merchant_address: merchant.clone(),
            payer: payer.clone(),
            token: token.clone(),
            amount,
            description: str(&env, "desc"),
            expires_at: 0,
        };
        let (_pk, sig) = sign_order(&env, &order);
        client.process_payment_with_signature(&payer, &order, &sig, &zero_key(&env));
    }
    let page = client.get_merchant_payment_history(&merchant, &None, &10, &None, &SortField::Amount, &SortOrder::Descending);
    assert_eq!(page.total, 3);
    assert_eq!(page.records.get(0).unwrap().amount, 300);
}

#[test]
fn test_large_payment_index_growth() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = create_token(&env, &admin);
    client.set_admin(&admins(&env, &admin), &1);
    client.register_merchant(&merchant, &str(&env, "Store"), &str(&env, "desc"), &str(&env, "c@c.com"), &MerchantCategory::Retail, &None);
    mint(&env, &token, &payer, 1_000_000);
    for i in 0..250 {
        let order = PaymentOrder {
            order_id: bytes(&env, &alloc::format!("ORDER_{:04}", i)),
            merchant_address: merchant.clone(),
            payer: payer.clone(),
            token: token.clone(),
            amount: 1,
            description: str(&env, "Test"),
            expires_at: 0,
        };
        client.process_payment_with_signature(&payer, &order, &zero_sig(&env), &zero_key(&env));
    }
    let history = client.get_merchant_payment_history(&merchant, &None, &10, &None, &SortField::Date, &SortOrder::Ascending);
    assert_eq!(history.total, 250);
    assert_eq!(history.records.len(), 10);
}

fn setup_payer_history(env: &Env, client: &PaymentContractClient, amounts: &[i128]) -> (Address, Address, Address, Address) {
    let admin = Address::generate(env);
    let merchant = Address::generate(env);
    let payer = Address::generate(env);
    let token = create_token(env, &admin);
    client.set_admin(&admins(env, &admin), &1);
    client.register_merchant(&merchant, &str(env, "Store"), &str(env, "desc"), &str(env, "c@c.com"), &MerchantCategory::Retail, &None);
    let total: i128 = amounts.iter().sum::<i128>() + 1000;
    mint(env, &token, &payer, total);
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
        client.process_payment_with_signature(&payer, &order, &sig, &zero_key(env));
    }
    (admin, merchant, payer, token)
}

#[test]
fn test_payer_history_no_payments() {
    let (env, client) = setup();
    let payer = Address::generate(&env);
    let page = client.get_payer_payment_history(&payer, &None, &10, &None, &SortField::Date, &SortOrder::Descending);
    assert_eq!(page.total, 0);
    assert!(page.next_cursor.is_none());
}

#[test]
fn test_payer_history_single_payment() {
    let (env, client) = setup();
    let (_admin, _merchant, payer, _token) = setup_payer_history(&env, &client, &[500]);
    let page = client.get_payer_payment_history(&payer, &None, &10, &None, &SortField::Date, &SortOrder::Ascending);
    assert_eq!(page.total, 1);
    assert_eq!(page.records.get(0).unwrap().amount, 500);
}

#[test]
fn test_payer_history_multiple_payments() {
    let (env, client) = setup();
    let (_admin, _merchant, payer, _token) = setup_payer_history(&env, &client, &[100, 200, 300]);
    let page = client.get_payer_payment_history(&payer, &None, &10, &None, &SortField::Amount, &SortOrder::Ascending);
    assert_eq!(page.total, 3);
    assert_eq!(page.records.get(0).unwrap().amount, 100);
    assert_eq!(page.records.get(2).unwrap().amount, 300);
}

#[test]
fn test_payer_history_filter_date_range() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = create_token(&env, &admin);
    client.set_admin(&admins(&env, &admin), &1);
    client.register_merchant(&merchant, &str(&env, "Store"), &str(&env, "desc"), &str(&env, "c@c.com"), &MerchantCategory::Retail, &None);
    mint(&env, &token, &payer, 10000);
    env.ledger().with_mut(|l| l.timestamp = 1000);
    let o1 = PaymentOrder { order_id: bytes(&env, "D_001"), merchant_address: merchant.clone(), payer: payer.clone(), token: token.clone(), amount: 100, description: str(&env, "d"), expires_at: 0 };
    let (_pk, sig) = sign_order(&env, &o1);
    client.process_payment_with_signature(&payer, &o1, &sig, &zero_key(&env));
    env.ledger().with_mut(|l| l.timestamp = 5000);
    let o2 = PaymentOrder { order_id: bytes(&env, "D_002"), merchant_address: merchant.clone(), payer: payer.clone(), token: token.clone(), amount: 200, description: str(&env, "d"), expires_at: 0 };
    let (_pk2, sig2) = sign_order(&env, &o2);
    client.process_payment_with_signature(&payer, &o2, &sig2, &zero_key(&env));
    let filter = PaymentFilter { date_start: Some(500), date_end: Some(2000), amount_min: None, amount_max: None, token: None, status: StatusFilter::Any };
    let page = client.get_payer_payment_history(&payer, &None, &10, &Some(filter), &SortField::Date, &SortOrder::Ascending);
    assert_eq!(page.total, 1);
    assert_eq!(page.records.get(0).unwrap().amount, 100);
}

#[test]
fn test_payer_history_filter_amount_range() {
    let (env, client) = setup();
    let (_admin, _merchant, payer, _token) = setup_payer_history(&env, &client, &[50, 150, 500]);
    let filter = PaymentFilter { date_start: None, date_end: None, amount_min: Some(100), amount_max: Some(200), token: None, status: StatusFilter::Any };
    let page = client.get_payer_payment_history(&payer, &None, &10, &Some(filter), &SortField::Amount, &SortOrder::Ascending);
    assert_eq!(page.total, 1);
    assert_eq!(page.records.get(0).unwrap().amount, 150);
}

#[test]
fn test_payer_history_pagination() {
    let (env, client) = setup();
    let (_admin, _merchant, payer, _token) = setup_payer_history(&env, &client, &[100, 200, 300, 400, 500]);
    let page1 = client.get_payer_payment_history(&payer, &None, &2, &None, &SortField::Amount, &SortOrder::Ascending);
    assert_eq!(page1.records.len(), 2);
    assert!(page1.next_cursor.is_some());
    let page2 = client.get_payer_payment_history(&payer, &page1.next_cursor, &2, &None, &SortField::Amount, &SortOrder::Ascending);
    assert_eq!(page2.records.len(), 2);
    assert!(page2.records.get(0).unwrap().amount > page1.records.get(1).unwrap().amount);
}

// ── Whitelist tests ───────────────────────────────────────────────────────────

#[test]
fn test_whitelist_mode_blocks_unregistered_merchant() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    client.set_admin(&admins(&env, &admin), &1);
    client.set_whitelist_mode(&admins(&env, &admin), &true);
    assert_eq!(
        client.try_register_merchant(&merchant, &str(&env, "Store"), &str(&env, "desc"), &str(&env, "c@c.com"), &MerchantCategory::Retail, &None),
        Err(Ok(PaymentError::Unauthorized))
    );
}

#[test]
fn test_whitelist_mode_allows_approved_merchant() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    client.set_admin(&admins(&env, &admin), &1);
    client.set_whitelist_mode(&admins(&env, &admin), &true);
    client.approve_merchant_registration(&admins(&env, &admin), &merchant);
    client.register_merchant(&merchant, &str(&env, "Store"), &str(&env, "desc"), &str(&env, "c@c.com"), &MerchantCategory::Retail, &None);
    assert!(client.get_merchant(&merchant).active);
}

#[test]
fn test_set_whitelist_mode_non_admin_fails() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let other = Address::generate(&env);
    client.set_admin(&admins(&env, &admin), &1);
    assert_eq!(
        client.try_set_whitelist_mode(&admins(&env, &other), &true),
        Err(Ok(PaymentError::Unauthorized))
    );
}

// ── Archive / cleanup tests ───────────────────────────────────────────────────

#[test]
fn test_archive_payment_record_removes_from_indexes() {
    let (env, client) = setup();
    let (admin, merchant, payer, _token) = setup_paid_order(&env, &client);
    let page = client.get_merchant_payment_history(&merchant, &None, &10, &None, &SortField::Date, &SortOrder::Ascending);
    assert_eq!(page.total, 1);
    client.archive_payment_record(&admins(&env, &admin), &bytes(&env, "ORDER_001"));
    assert_eq!(client.try_get_payment_by_id(&payer, &bytes(&env, "ORDER_001")), Err(Ok(PaymentError::PaymentNotFound)));
    assert_eq!(client.get_merchant_payment_history(&merchant, &None, &10, &None, &SortField::Date, &SortOrder::Ascending).total, 0);
    assert_eq!(client.get_payer_payment_history(&payer, &None, &10, &None, &SortField::Date, &SortOrder::Ascending).total, 0);
}

#[test]
fn test_archive_payment_record_non_admin_fails() {
    let (env, client) = setup();
    let (_admin, _merchant, payer, _token) = setup_paid_order(&env, &client);
    assert_eq!(
        client.try_archive_payment_record(&admins(&env, &payer), &bytes(&env, "ORDER_001")),
        Err(Ok(PaymentError::Unauthorized))
    );
}

#[test]
fn test_archive_payment_record_not_found_fails() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    client.set_admin(&admins(&env, &admin), &1);
    assert_eq!(
        client.try_archive_payment_record(&admins(&env, &admin), &bytes(&env, "NONEXISTENT")),
        Err(Ok(PaymentError::PaymentNotFound))
    );
}

#[test]
fn test_set_cleanup_period() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    client.set_admin(&admins(&env, &admin), &1);
    client.set_payment_cleanup_period(&admins(&env, &admin), &86400);
}

#[test]
fn test_set_cleanup_period_zero_fails() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    client.set_admin(&admins(&env, &admin), &1);
    assert_eq!(
        client.try_set_payment_cleanup_period(&admins(&env, &admin), &0),
        Err(Ok(PaymentError::InvalidInput))
    );
}

#[test]
fn test_set_cleanup_period_valid_is_persisted() {
    let (env, client) = setup();
    let (admin, _merchant, payer, _token) = setup_paid_order(&env, &client);
    client.set_payment_cleanup_period(&admins(&env, &admin), &1);
    env.ledger().with_mut(|l| l.timestamp = 100);
    let count = client.cleanup_expired_payments(&admins(&env, &admin));
    assert_eq!(count, 1);
    assert_eq!(client.try_get_payment_by_id(&payer, &bytes(&env, "ORDER_001")), Err(Ok(PaymentError::PaymentNotFound)));
}

#[test]
fn test_cleanup_expired_payments() {
    let (env, client) = setup();
    let (admin, merchant, payer, token) = setup_paid_order(&env, &client);
    client.set_payment_cleanup_period(&admins(&env, &admin), &3600);
    let order2 = PaymentOrder { order_id: bytes(&env, "ORDER_002"), merchant_address: merchant.clone(), payer: payer.clone(), token: token.clone(), amount: 500, description: str(&env, "desc"), expires_at: 0 };
    client.process_payment_with_signature(&payer, &order2, &zero_sig(&env), &zero_key(&env));
    env.ledger().with_mut(|l| l.timestamp = 7201);
    let count = client.cleanup_expired_payments(&admins(&env, &admin));
    assert_eq!(count, 2);
    assert_eq!(client.try_get_payment_by_id(&payer, &bytes(&env, "ORDER_001")), Err(Ok(PaymentError::PaymentNotFound)));
    assert_eq!(client.try_get_payment_by_id(&payer, &bytes(&env, "ORDER_002")), Err(Ok(PaymentError::PaymentNotFound)));
}

#[test]
fn test_cleanup_no_expired_returns_zero() {
    let (env, client) = setup();
    let (admin, _merchant, _payer, _token) = setup_paid_order(&env, &client);
    assert_eq!(client.cleanup_expired_payments(&admins(&env, &admin)), 0);
}

#[test]
fn test_cleanup_non_admin_unauthorized() {
    let (env, client) = setup();
    let (_admin, _merchant, _payer, _token) = setup_paid_order(&env, &client);
    let non_admin = Address::generate(&env);
    assert_eq!(
        client.try_cleanup_expired_payments(&admins(&env, &non_admin)),
        Err(Ok(PaymentError::Unauthorized))
    );
}

// ── Global stats tests ────────────────────────────────────────────────────────

#[test]
fn test_get_global_payment_stats() {
    let (env, client) = setup();
    env.ledger().with_mut(|l| l.timestamp = 1000);
    let (admin, merchant, payer, token) = setup_paid_order(&env, &client);
    env.ledger().with_mut(|l| l.timestamp = 2000);
    let order2 = PaymentOrder { order_id: bytes(&env, "ORDER_002"), merchant_address: merchant.clone(), payer: payer.clone(), token: token.clone(), amount: 2000, description: str(&env, "p2"), expires_at: 0 };
    let (_pk2, sig2) = sign_order(&env, &order2);
    client.process_payment_with_signature(&payer, &order2, &sig2, &zero_key(&env));
    env.ledger().with_mut(|l| l.timestamp = 3000);
    client.initiate_refund(&payer, &bytes(&env, "R1"), &bytes(&env, "ORDER_001"), &500, &str(&env, "reason"));
    client.approve_refund(&merchant, &bytes(&env, "R1"), &None);
    env.ledger().with_mut(|l| l.timestamp = 4000);
    client.execute_refund(&merchant, &bytes(&env, "R1"));
    let stats = client.get_global_payment_stats(&admins(&env, &admin), &None, &None);
    assert_eq!(stats.total_payments, 2);
    assert_eq!(stats.total_volume, 3000);
    assert_eq!(stats.total_refunds, 1);
    assert_eq!(stats.total_refund_volume, 500);
    let stats = client.get_global_payment_stats(&admins(&env, &admin), &Some(500), &Some(1500));
    assert_eq!(stats.total_payments, 1);
    assert_eq!(stats.total_volume, 1000);
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
    client.set_admin(&admins(&env, &admin), &1);
    client.register_merchant(&merchant, &str(&env, "Store"), &str(&env, "desc"), &str(&env, "c@c.com"), &MerchantCategory::Retail, &None);
    mint(&env, &token, &signer1, 5000);
    let order = PaymentOrder { order_id: bytes(&env, "MS_001"), merchant_address: merchant.clone(), payer: signer1.clone(), token: token.clone(), amount: 1000, description: str(&env, "Multisig order"), expires_at: 0 };
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
    client.set_admin(&admins(&env, &admin), &1);
    client.register_merchant(&merchant, &str(&env, "Store"), &str(&env, "desc"), &str(&env, "c@c.com"), &MerchantCategory::Retail, &None);
    mint(&env, &token, &signer1, 5000);
    let order = PaymentOrder { order_id: bytes(&env, "MS_002"), merchant_address: merchant.clone(), payer: signer1.clone(), token: token.clone(), amount: 1000, description: str(&env, "Multisig order"), expires_at: 0 };
    let mut signers = Vec::new(&env);
    signers.push_back(signer1.clone());
    signers.push_back(signer2.clone());
    client.initiate_multisig_payment(&signer1, &bytes(&env, "MS_002"), &order, &signers);
    client.sign_multisig_payment(&signer1, &bytes(&env, "MS_002"));
    assert_eq!(
        client.try_execute_multisig_payment(&signer1, &bytes(&env, "MS_002")),
        Err(Ok(PaymentError::InsufficientSignatures))
    );
}

#[test]
fn test_initiate_multisig_duplicate_signer_fails() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let signer1 = Address::generate(&env);
    let token = create_token(&env, &admin);
    client.set_admin(&admins(&env, &admin), &1);
    client.register_merchant(&merchant, &str(&env, "Store"), &str(&env, "desc"), &str(&env, "c@c.com"), &MerchantCategory::Retail, &None);
    mint(&env, &token, &signer1, 5000);
    let order = PaymentOrder { order_id: bytes(&env, "MS_DUP"), merchant_address: merchant.clone(), payer: signer1.clone(), token: token.clone(), amount: 1000, description: str(&env, "Multisig order"), expires_at: 0 };
    let mut signers = Vec::new(&env);
    signers.push_back(signer1.clone());
    signers.push_back(signer1.clone());
    assert_eq!(
        client.try_initiate_multisig_payment(&signer1, &bytes(&env, "MS_DUP"), &order, &signers),
        Err(Ok(PaymentError::InvalidInput))
    );
}

#[test]
fn test_multisig_payment_expiry() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let signer1 = Address::generate(&env);
    let token = create_token(&env, &admin);
    client.set_admin(&admins(&env, &admin), &1);
    client.register_merchant(&merchant, &str(&env, "Store"), &str(&env, "desc"), &str(&env, "c@c.com"), &MerchantCategory::Retail, &None);
    mint(&env, &token, &signer1, 5000);
    let order = make_order(&env, &merchant, &signer1, &token);
    let mut signers = Vec::new(&env);
    signers.push_back(signer1.clone());
    client.initiate_multisig_payment(&signer1, &bytes(&env, "MS_EXPIRY"), &order, &signers);
    env.ledger().with_mut(|l| l.timestamp = 86400 + 3601);
    assert_eq!(client.try_sign_multisig_payment(&signer1, &bytes(&env, "MS_EXPIRY")), Err(Ok(PaymentError::PaymentExpired)));
    assert_eq!(client.try_execute_multisig_payment(&signer1, &bytes(&env, "MS_EXPIRY")), Err(Ok(PaymentError::PaymentExpired)));
}

// ── Subscription tests ────────────────────────────────────────────────────────

fn make_plan(env: &Env, token: &Address) -> SubscriptionPlan {
    SubscriptionPlan {
        interval: 2_592_000, // 30 days
        amount: 500,
        token: token.clone(),
    }
}

#[test]
fn test_create_subscription_success() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = create_token(&env, &admin);
    client.set_admin(&admins(&env, &admin), &1);
    client.register_merchant(&merchant, &str(&env, "Store"), &str(&env, "desc"), &str(&env, "c@c.com"), &MerchantCategory::Retail, &None);
    let sub_id = bytes(&env, "SUB_001");
    client.create_subscription(&payer, &merchant, &sub_id, &make_plan(&env, &token));
    let sub = client.get_subscription(&sub_id);
    assert_eq!(sub.payer, payer);
    assert_eq!(sub.merchant, merchant);
    assert_eq!(sub.last_charged_at, 0);
    assert_eq!(sub.status, crate::types::SubscriptionStatus::Active);
}

#[test]
fn test_create_subscription_duplicate_fails() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = create_token(&env, &admin);
    client.set_admin(&admins(&env, &admin), &1);
    client.register_merchant(&merchant, &str(&env, "Store"), &str(&env, "desc"), &str(&env, "c@c.com"), &MerchantCategory::Retail, &None);
    let sub_id = bytes(&env, "SUB_001");
    client.create_subscription(&payer, &merchant, &sub_id, &make_plan(&env, &token));
    assert_eq!(
        client.try_create_subscription(&payer, &merchant, &sub_id, &make_plan(&env, &token)),
        Err(Ok(PaymentError::SubscriptionAlreadyExists))
    );
}

#[test]
fn test_create_subscription_zero_interval_fails() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = create_token(&env, &admin);
    client.set_admin(&admins(&env, &admin), &1);
    client.register_merchant(&merchant, &str(&env, "Store"), &str(&env, "desc"), &str(&env, "c@c.com"), &MerchantCategory::Retail, &None);
    let bad_plan = SubscriptionPlan { interval: 0, amount: 500, token: token.clone() };
    assert_eq!(
        client.try_create_subscription(&payer, &merchant, &bytes(&env, "SUB_BAD"), &bad_plan),
        Err(Ok(PaymentError::InvalidInput))
    );
}

#[test]
fn test_create_subscription_inactive_merchant_fails() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = create_token(&env, &admin);
    client.set_admin(&admins(&env, &admin), &1);
    client.register_merchant(&merchant, &str(&env, "Store"), &str(&env, "desc"), &str(&env, "c@c.com"), &MerchantCategory::Retail, &None);
    client.deactivate_merchant(&merchant, &None);
    assert_eq!(
        client.try_create_subscription(&payer, &merchant, &bytes(&env, "SUB_001"), &make_plan(&env, &token)),
        Err(Ok(PaymentError::MerchantInactive))
    );
}

#[test]
fn test_cancel_subscription_success() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = create_token(&env, &admin);
    client.set_admin(&admins(&env, &admin), &1);
    client.register_merchant(&merchant, &str(&env, "Store"), &str(&env, "desc"), &str(&env, "c@c.com"), &MerchantCategory::Retail, &None);
    let sub_id = bytes(&env, "SUB_001");
    client.create_subscription(&payer, &merchant, &sub_id, &make_plan(&env, &token));
    client.cancel_subscription(&payer, &sub_id);
    assert_eq!(client.get_subscription(&sub_id).status, crate::types::SubscriptionStatus::Cancelled);
}

#[test]
fn test_cancel_subscription_wrong_payer_fails() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let other = Address::generate(&env);
    let token = create_token(&env, &admin);
    client.set_admin(&admins(&env, &admin), &1);
    client.register_merchant(&merchant, &str(&env, "Store"), &str(&env, "desc"), &str(&env, "c@c.com"), &MerchantCategory::Retail, &None);
    let sub_id = bytes(&env, "SUB_001");
    client.create_subscription(&payer, &merchant, &sub_id, &make_plan(&env, &token));
    assert_eq!(
        client.try_cancel_subscription(&other, &sub_id),
        Err(Ok(PaymentError::Unauthorized))
    );
}

#[test]
fn test_cancel_already_cancelled_fails() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = create_token(&env, &admin);
    client.set_admin(&admins(&env, &admin), &1);
    client.register_merchant(&merchant, &str(&env, "Store"), &str(&env, "desc"), &str(&env, "c@c.com"), &MerchantCategory::Retail, &None);
    let sub_id = bytes(&env, "SUB_001");
    client.create_subscription(&payer, &merchant, &sub_id, &make_plan(&env, &token));
    client.cancel_subscription(&payer, &sub_id);
    assert_eq!(
        client.try_cancel_subscription(&payer, &sub_id),
        Err(Ok(PaymentError::SubscriptionNotActive))
    );
}

#[test]
fn test_process_subscription_payment_first_charge() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = create_token(&env, &admin);
    client.set_admin(&admins(&env, &admin), &1);
    client.register_merchant(&merchant, &str(&env, "Store"), &str(&env, "desc"), &str(&env, "c@c.com"), &MerchantCategory::Retail, &None);
    mint(&env, &token, &payer, 5000);
    let token_client = soroban_sdk::token::TokenClient::new(&env, &token);
    let payer_before = token_client.balance(&payer);
    let merchant_before = token_client.balance(&merchant);
    let sub_id = bytes(&env, "SUB_001");
    client.create_subscription(&payer, &merchant, &sub_id, &make_plan(&env, &token));
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
    client.process_payment_with_signature(&payer, &o1, &sig, &BytesN::from_array(&env, &[0u8; 32]));

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
    client.process_payment_with_signature(&payer, &o2, &sig2, &BytesN::from_array(&env, &[0u8; 32]));

    use crate::types::{PaymentFilter, StatusFilter};
    let filter = PaymentFilter {
        date_start: Some(500),
        date_end: Some(2000),
        amount_min: None,
        amount_max: None,
        tokens: None,
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
        tokens: None,
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
fn test_process_subscription_payment_after_interval() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let token1 = create_token(&env, &admin);
    let token2 = create_token(&env, &admin);

    client.set_admin(&vec![&env, admin.clone()], &1);
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
    client.process_payment_with_signature(&payer, &o1, &sig, &BytesN::from_array(&env, &[0u8; 32]));

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
    client.process_payment_with_signature(&payer, &o2, &sig2, &BytesN::from_array(&env, &[0u8; 32]));

    use crate::types::{PaymentFilter, StatusFilter};
    let filter = PaymentFilter {
        date_start: None,
        date_end: None,
        amount_min: None,
        amount_max: None,
        tokens: Some(Vec::from_array(&env, [token2.clone()])),
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
        tokens: None,
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
fn test_process_subscription_payment_before_interval_fails() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = create_token(&env, &admin);
    client.set_admin(&admins(&env, &admin), &1);
    client.register_merchant(&merchant, &str(&env, "Store"), &str(&env, "desc"), &str(&env, "c@c.com"), &MerchantCategory::Retail, &None);
    mint(&env, &token, &payer, 5000);
    let sub_id = bytes(&env, "SUB_001");
    client.create_subscription(&payer, &merchant, &sub_id, &make_plan(&env, &token));
    env.ledger().with_mut(|l| l.timestamp = 1000);
    client.process_subscription_payment(&payer, &sub_id);
    // Try again before interval elapses
    env.ledger().with_mut(|l| l.timestamp = 1001);
    assert_eq!(
        client.try_process_subscription_payment(&payer, &sub_id),
        Err(Ok(PaymentError::SubscriptionIntervalNotElapsed))
    );
}

#[test]
fn test_process_subscription_payment_cancelled_fails() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = create_token(&env, &admin);
    client.set_admin(&admins(&env, &admin), &1);
    client.register_merchant(&merchant, &str(&env, "Store"), &str(&env, "desc"), &str(&env, "c@c.com"), &MerchantCategory::Retail, &None);
    mint(&env, &token, &payer, 5000);
    let sub_id = bytes(&env, "SUB_001");
    client.create_subscription(&payer, &merchant, &sub_id, &make_plan(&env, &token));
    client.cancel_subscription(&payer, &sub_id);
    assert_eq!(
        client.try_process_subscription_payment(&payer, &sub_id),
        Err(Ok(PaymentError::SubscriptionNotActive))
    );
}

#[test]
fn test_get_subscription_not_found_fails() {
    let (env, client) = setup();
    assert_eq!(
        client.try_get_subscription(&bytes(&env, "NONEXISTENT")),
        Err(Ok(PaymentError::SubscriptionNotFound))
    );
}

// ── T-020: inactive merchant payment ─────────────────────────────────────────

#[test]
fn test_payment_with_inactive_merchant_fails() {
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

    // Deactivate the merchant
    client.deactivate_merchant(&merchant, &None);
    let m = client.get_merchant(&merchant);
    assert!(!m.active);

    // Attempt payment → must fail with MerchantInactive
    let order = make_order(&env, &merchant, &payer, &token);
    let (pub_key, sig) = sign_order(&env, &order);
    let result = client.try_process_payment_with_signature(&payer, &order, &sig, &pub_key);
    assert_eq!(result, Err(Ok(PaymentError::MerchantInactive)));
}

// ── SEC-011: max pending refunds per order ────────────────────────────────────

#[test]
fn test_max_pending_refunds_per_order() {
    let (env, client) = setup();
    let (_admin, _merchant, payer, _token) = setup_paid_order(&env, &client);

    // Initiate 10 refunds of 1 each (within the 1000 payment amount)
    for i in 0..10u32 {
        let refund_id = alloc::format!("RF_{:02}", i);
        client.initiate_refund(
            &payer,
            &Bytes::from_slice(&env, refund_id.as_bytes()),
            &bytes(&env, "ORDER_001"),
            &1,
            &str(&env, "reason"),
        );
    }

    // 11th refund must be rejected with InvalidInput
    let result = client.try_initiate_refund(
        &payer,
        &bytes(&env, "RF_10"),
        &bytes(&env, "ORDER_001"),
        &1,
        &str(&env, "over limit"),
    );
    assert_eq!(result, Err(Ok(PaymentError::InvalidInput)));
}
