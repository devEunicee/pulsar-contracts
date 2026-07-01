// SPDX-License-Identifier: MIT

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
    types::{MerchantCategory, PaymentOrder},
    PaymentContract, PaymentContractClient,
};

fn setup() -> (Env, PaymentContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(PaymentContract, ());
    let client = PaymentContractClient::new(&env, &contract_id);
    (env, client)
}

fn create_token(env: &Env, admin: &Address) -> Address {
    env.register_stellar_asset_contract_v2(admin.clone())
        .address()
}

fn mint(env: &Env, token: &Address, to: &Address, amount: i128) {
    let client = StellarAssetClient::new(env, token);
    client.mint(to, &amount);
}

fn bytes(env: &Env, s: &str) -> Bytes {
    Bytes::from_slice(env, s.as_bytes())
}

fn str(env: &Env, s: &str) -> String {
    String::from_str(env, s)
}

fn admins(env: &Env, admin: &Address) -> Vec<Address> {
    let mut v = Vec::new(env);
    v.push_back(admin.clone());
    v
}

fn zero_key(env: &Env) -> BytesN<32> {
    BytesN::from_array(env, &[0u8; 32])
}

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

fn make_order(env: &Env, merchant: &Address, payer: &Address, token: &Address) -> PaymentOrder {
    PaymentOrder {
        order_id: bytes(env, "ORDER_001"),
        merchant_address: merchant.clone(),
        payer: payer.clone(),
        token: token.clone(),
        amount: 1000,
        description: str(env, "test payment"),
        expires_at: 0,
    }
}

fn pay_at(
    env: &Env,
    client: &PaymentContractClient,
    admin: &Address,
    merchant: &Address,
    payer: &Address,
    token: &Address,
    order_id: &str,
    amount: i128,
    timestamp: u64,
) {
    env.ledger().with_mut(|l| l.timestamp = timestamp);
    let order = PaymentOrder {
        order_id: bytes(env, order_id),
        merchant_address: merchant.clone(),
        payer: payer.clone(),
        token: token.clone(),
        amount,
        description: str(env, "payment"),
        expires_at: 0,
    };
    let (_pk, sig) = sign_order(env, &order);
    client.process_payment_with_signature(payer, &order, &sig, &zero_key(env));
    let _ = admin;
}

/// SC-039: date filtering for get_global_payment_stats.
#[test]
fn test_get_global_payment_stats_no_filter() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = create_token(&env, &admin);

    client.set_admin(&admins(&env, &admin), &1);
    client.register_merchant(
        &merchant,
        &str(&env, "Store"),
        &str(&env, "desc"),
        &str(&env, "c@c.com"),
        &MerchantCategory::Retail,
        &None,
    );
    mint(&env, &token, &payer, 10_000);

    pay_at(&env, &client, &admin, &merchant, &payer, &token, "ORDER_001", 1000, 1000);
    pay_at(&env, &client, &admin, &merchant, &payer, &token, "ORDER_002", 2000, 2000);

    env.ledger().with_mut(|l| l.timestamp = 3000);
    client.initiate_refund(
        &payer,
        &bytes(&env, "R1"),
        &bytes(&env, "ORDER_001"),
        &500,
        &str(&env, "reason"),
    );
    client.approve_refund(&merchant, &bytes(&env, "R1"), &None);
    env.ledger().with_mut(|l| l.timestamp = 4000);
    client.execute_refund(&merchant, &bytes(&env, "R1"));

    let stats = client.get_global_payment_stats(&admins(&env, &admin), &None, &None);
    assert_eq!(stats.total_payments, 2);
    assert_eq!(stats.total_volume, 3000);
    assert_eq!(stats.total_refunds, 1);
    assert_eq!(stats.total_refund_volume, 500);
}

#[test]
fn test_get_global_payment_stats_start_only() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = create_token(&env, &admin);

    client.set_admin(&admins(&env, &admin), &1);
    client.register_merchant(
        &merchant,
        &str(&env, "Store"),
        &str(&env, "desc"),
        &str(&env, "c@c.com"),
        &MerchantCategory::Retail,
        &None,
    );
    mint(&env, &token, &payer, 10_000);

    pay_at(&env, &client, &admin, &merchant, &payer, &token, "ORDER_001", 1000, 1000);
    pay_at(&env, &client, &admin, &merchant, &payer, &token, "ORDER_002", 2000, 2500);

    let stats = client.get_global_payment_stats(&admins(&env, &admin), &Some(2000), &None);
    assert_eq!(stats.total_payments, 1);
    assert_eq!(stats.total_volume, 2000);
}

#[test]
fn test_get_global_payment_stats_end_only() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = create_token(&env, &admin);

    client.set_admin(&admins(&env, &admin), &1);
    client.register_merchant(
        &merchant,
        &str(&env, "Store"),
        &str(&env, "desc"),
        &str(&env, "c@c.com"),
        &MerchantCategory::Retail,
        &None,
    );
    mint(&env, &token, &payer, 10_000);

    pay_at(&env, &client, &admin, &merchant, &payer, &token, "ORDER_001", 1000, 1000);
    pay_at(&env, &client, &admin, &merchant, &payer, &token, "ORDER_002", 2000, 2500);

    let stats = client.get_global_payment_stats(&admins(&env, &admin), &None, &Some(1500));
    assert_eq!(stats.total_payments, 1);
    assert_eq!(stats.total_volume, 1000);
}

#[test]
fn test_get_global_payment_stats_both_filters() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = create_token(&env, &admin);

    client.set_admin(&admins(&env, &admin), &1);
    client.register_merchant(
        &merchant,
        &str(&env, "Store"),
        &str(&env, "desc"),
        &str(&env, "c@c.com"),
        &MerchantCategory::Retail,
        &None,
    );
    mint(&env, &token, &payer, 10_000);

    pay_at(&env, &client, &admin, &merchant, &payer, &token, "ORDER_001", 1000, 1000);
    pay_at(&env, &client, &admin, &merchant, &payer, &token, "ORDER_002", 2000, 2000);
    pay_at(&env, &client, &admin, &merchant, &payer, &token, "ORDER_003", 3000, 3000);

    let stats = client.get_global_payment_stats(&admins(&env, &admin), &Some(1500), &Some(2500));
    assert_eq!(stats.total_payments, 1);
    assert_eq!(stats.total_volume, 2000);
}

#[test]
fn test_get_global_payment_stats_empty_range() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = create_token(&env, &admin);

    client.set_admin(&admins(&env, &admin), &1);
    client.register_merchant(
        &merchant,
        &str(&env, "Store"),
        &str(&env, "desc"),
        &str(&env, "c@c.com"),
        &MerchantCategory::Retail,
        &None,
    );
    mint(&env, &token, &payer, 10_000);

    pay_at(&env, &client, &admin, &merchant, &payer, &token, "ORDER_001", 1000, 1000);
    pay_at(&env, &client, &admin, &merchant, &payer, &token, "ORDER_002", 2000, 2000);

    let stats = client.get_global_payment_stats(&admins(&env, &admin), &Some(5000), &Some(6000));
    assert_eq!(stats.total_payments, 0);
    assert_eq!(stats.total_volume, 0);
    assert_eq!(stats.total_refunds, 0);
    assert_eq!(stats.total_refund_volume, 0);
}
