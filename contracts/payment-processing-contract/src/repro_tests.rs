#![cfg(test)]

extern crate alloc;
use alloc::format;
use alloc::vec;

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::StellarAssetClient,
    Address, Bytes, Env, String, Vec, BytesN,
};

use crate::{
    error::PaymentError,
    types::{MerchantCategory, PaymentOrder, PaymentStatus, SortField, SortOrder},
    PaymentContract, PaymentContractClient,
};

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
fn test_approve_refund_unauthorized_fails() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let stranger = Address::generate(&env);
    let token = create_token(&env, &admin);

    client.set_admin(&admin);
    client.register_merchant(&merchant, &str(&env, "M"), &str(&env, "D"), &str(&env, "E"), &MerchantCategory::Retail);
    mint(&env, &token, &admin, &payer, 1000);

    let order = make_order(&env, &merchant, &payer, &token);
    client.process_payment_with_signature(&payer, &order, &BytesN::from_array(&env, &[0u8; 64]), &BytesN::from_array(&env, &[0u8; 32]));

    client.initiate_refund(&payer, &bytes(&env, "R1"), &order.order_id, &100, &str(&env, "reason"));

    let result = client.try_approve_refund(&stranger, &bytes(&env, "R1"));
    assert_eq!(result, Err(Ok(PaymentError::Unauthorized)));
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

#[test]
fn test_pagination_cursor_inverted() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = create_token(&env, &admin);

    client.set_admin(&admin);
    client.register_merchant(&merchant, &str(&env, "M"), &str(&env, "D"), &str(&env, "E"), &MerchantCategory::Retail);
    mint(&env, &token, &admin, &payer, 10000);

    // Create 5 payments with increasing amounts
    for i in 1..=5 {
        let mut order = make_order(&env, &merchant, &payer, &token);
        order.order_id = bytes(&env, &format!("ORDER_{:03}", i));
        order.amount = i * 100;
        client.process_payment_with_signature(&payer, &order, &BytesN::from_array(&env, &[0u8; 64]), &BytesN::from_array(&env, &[0u8; 32]));
    }

    // Page 1: limit 2, sorted by Amount Descending
    // Order in storage: 1, 2, 3, 4, 5
    // Sorted order: 5, 4, 3, 2, 1
    let page1_desc = client.get_merchant_payment_history(&merchant, &None, &2, &None, &SortField::Amount, &SortOrder::Descending);
    assert_eq!(page1_desc.records.len(), 2);
    assert_eq!(page1_desc.records.get(0).unwrap().amount, 500);
    assert_eq!(page1_desc.records.get(1).unwrap().amount, 400);
    let cursor_desc = page1_desc.next_cursor; // ORDER_004

    // Page 2: limit 2, sorted by Amount Descending, cursor ORDER_004
    let page2_desc = client.get_merchant_payment_history(&merchant, &cursor_desc, &2, &None, &SortField::Amount, &SortOrder::Descending);
    
    // We expect 300 and 200
    assert_eq!(page2_desc.records.len(), 2);
    assert_eq!(page2_desc.records.get(0).unwrap().amount, 300);
    assert_eq!(page2_desc.records.get(1).unwrap().amount, 200);
}

#[test]
fn test_contract_upgrade_emits_event() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    client.set_admin(&admin);

    let new_hash = BytesN::from_array(&env, &[1u8; 32]);
    client.migrate(&admin, &new_hash);

    let events = env.events().all();
    let last_event = events.get(events.len() - 1).unwrap();

    let topics = last_event.1;
    assert_eq!(topics.len(), 1);
    let topic: String = topics.get(0).unwrap().into_val(&env);
    assert_eq!(topic, str(&env, "contract_upgraded"));

    let (emitted_admin, emitted_hash, ts): (Address, BytesN<32>, u64) = last_event.2.into_val(&env);
    assert_eq!(emitted_admin, admin);
    assert_eq!(emitted_hash, new_hash);
    assert!(ts > 0);
}
