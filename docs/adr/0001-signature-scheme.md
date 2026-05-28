# ADR-0001: Use ed25519 for Payment Order Signature Verification

**Status:** Accepted  
**Date:** 2024-01-01  
**Deciders:** Pulsar Contributors

## Context

Payment orders must be authorised by the merchant before a payer can execute them. Two options were considered:

1. **Stellar native auth** — require the merchant's Stellar account to sign the Soroban transaction itself via `require_auth()`.
2. **ed25519 off-chain signature** — merchant signs the serialised `PaymentOrder` XDR off-chain; the contract verifies the signature on-chain using `env.crypto().ed25519_verify`.

Stellar native auth ties authorisation to the transaction signer, which means the merchant must be online and co-sign every payment transaction. This is impractical for server-side merchant integrations where the merchant pre-authorises an order (e.g. at checkout) and the payer submits the transaction later.

## Decision

Use ed25519 off-chain signatures over the full XDR-serialised `PaymentOrder`. The merchant's ed25519 public key is stored on-chain in the `Merchant` struct (`signing_public_key`). `process_payment_with_signature` verifies the signature against the stored key.

## Consequences

### Positive
- Merchant can pre-sign orders without being online at payment time.
- Signature covers the full order (amount, token, payer, expiry), preventing tampering.
- Key is bound to the registered merchant on-chain, preventing key substitution attacks.

### Negative
- Merchants must manage an ed25519 key pair separately from their Stellar account key.
- If the signing key is compromised, the merchant must call `register_merchant` again (or a future `set_merchant_signing_key`) to rotate it.

### Neutral
- `signing_public_key` is `Option<BytesN<32>>`; merchants without a stored key skip signature verification, allowing gradual migration.
