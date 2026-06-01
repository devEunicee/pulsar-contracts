# Pulsar

> Scalable, secure, and decentralized smart contracts for Soroban Stellar.

[![CI](https://github.com/devEunicee/pulsar-contracts/actions/workflows/ci.yml/badge.svg)](https://github.com/devEunicee/pulsar-contracts/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

Pulsar is a comprehensive payment-processing smart contract for the Stellar Soroban network. It provides merchant management, payment processing with signature verification, refunds, multi-signature payments, and paginated payment history queries — all on-chain.

---

## Table of Contents

- [Overview](#overview)
- [Prerequisites](#prerequisites)
- [Project Structure](#project-structure)
- [Setup](#setup)
- [Testing](#testing)
- [Local Network](#local-network)
- [Deployment](#deployment)
- [Contract API](#contract-api)
  - [Admin](#admin)
  - [Merchant Management](#merchant-management)
  - [Payment Processing](#payment-processing)
  - [Payment Queries](#payment-queries)
  - [Refunds](#refunds)
  - [Multi-Signature Payments](#multi-signature-payments)
  - [Admin Config](#admin-config)
- [Events](#events)
- [Error Codes](#error-codes)
- [Contributing](#contributing)
- [License](#license)

---

## Overview

| Feature | Description |
|---|---|
| Merchant registry | Register, deactivate, and query merchants |
| Signed payments | Process payments verified by ed25519 merchant signature |
| Refunds | Initiate → Approve/Reject → Execute with 30-day window |
| Multi-sig | Require N-of-N signers before executing a payment |
| History queries | Cursor-based pagination with filtering and sorting |
| Global stats | Admin-only aggregate payment and refund statistics |

---

## Prerequisites

| Tool | Install |
|---|---|
| Rust (stable) | https://www.rust-lang.org/tools/install |
| Stellar CLI | https://developers.stellar.org/docs/tools/stellar-cli |
| Docker Desktop | https://www.docker.com/products/docker-desktop |

Verify:

```bash
rustc --version
cargo --version
stellar --version
docker --version
```

Add the WASM target:

```bash
rustup target add wasm32-unknown-unknown
```

---

## Project Structure

```
pulsar-contracts/
├── .github/
│   └── workflows/
│       └── ci.yml                  # CI: fmt, clippy, test, WASM build, audit
├── contracts/
│   └── payment-processing-contract/
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs              # Contract entry-point and all public functions
│           ├── types.rs            # All data structures and storage keys
│           ├── storage.rs          # Storage read/write helpers
│           ├── error.rs            # ContractError enum
│           ├── helper.rs           # Auth, validation, filter helpers
│           └── test.rs             # Unit tests (soroban testutils)
├── Cargo.toml                      # Workspace manifest
├── .gitignore
├── CONTRIBUTING.md
├── LICENSE
└── README.md
```

---

## Setup

```bash
git clone https://github.com/devEunicee/pulsar-contracts.git
cd pulsar-contracts
```

---

## Testing

```bash
cd contracts/payment-processing-contract
cargo test
```

Run a specific test:

```bash
cargo test test_successful_payment_with_signature
cargo test test_successful_refund_flow
cargo test test_get_merchant_payment_history
cargo test test_initiate_multisig_payment_success
```

---

## Local Network

```bash
# Start Docker Desktop, then:
stellar network container start local

# Build
cargo build --target wasm32-unknown-unknown --release

# Deploy
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/payment_processing_contract.wasm \
  --source-account <YOUR_SECRET_KEY> \
  --network local
```

Save the returned contract ID as `CONTRACT_ID`.

---

## Deployment

### Prerequisites — Keys

Before deploying you need a Stellar keypair and a funded account.

**Generate a keypair:**

```bash
stellar keys generate --global deployer
stellar keys address deployer        # prints your public key (G...)
stellar keys show deployer           # prints your secret key (S...) — keep this private
```

**Fund your testnet account via Friendbot:**

Testnet accounts start with zero balance. Friendbot is a faucet that credits 10 000 XLM to any new address:

```bash
curl "https://friendbot.stellar.org?addr=$(stellar keys address deployer)"
```

Or open the URL in a browser. Once funded, use the secret key wherever `<TESTNET_SECRET_KEY>` or `<ADMIN_SECRET_KEY>` appears below.

`$CONTRACT_ID` is the contract address printed by `stellar contract deploy` — save it immediately after running the deploy command.

---

### Testnet

```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/payment_processing_contract.wasm \
  --source-account <TESTNET_SECRET_KEY> \
  --network testnet
```

### Initialise

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source-account <ADMIN_SECRET_KEY> \
  --network testnet \
  -- set_admin \
  --admin <ADMIN_ADDRESS>
```

---

## Contract API

### Admin

#### `set_admin`

One-time initialisation. Caller becomes admin. Sets contract version to `1`.

```bash
stellar contract invoke --id $CONTRACT_ID --source-account <KEY> --network local \
  -- set_admin --admin <ADDRESS>
```

#### `get_version`

Returns the current contract version (u32).

```bash
stellar contract invoke --id $CONTRACT_ID --source-account <KEY> --network local \
  -- get_version
```

#### `upgrade`

Upgrades the contract WASM in-place. Admin only. Existing storage and contract address are preserved.

```bash
stellar contract invoke --id $CONTRACT_ID --source-account <ADMIN_KEY> --network local \
  -- upgrade \
  --admin <ADMIN_ADDRESS> \
  --new_wasm_hash <32_BYTE_HEX>
```

**Upgrade procedure:**

1. Build the new WASM: `cargo build --target wasm32-unknown-unknown --release`
2. Upload the WASM to the network and obtain its hash:
   ```bash
   stellar contract upload \
     --wasm target/wasm32-unknown-unknown/release/payment_processing_contract.wasm \
     --source-account <ADMIN_KEY> \
     --network testnet
   ```
3. Call `upgrade` with the returned hash.
4. Verify with `get_version`.

---

### Merchant Management

#### `register_merchant`

```bash
stellar contract invoke --id $CONTRACT_ID --source-account <MERCHANT_KEY> --network local \
  -- register_merchant \
  --merchant_address <ADDRESS> \
  --name "My Store" \
  --description "Store description" \
  --contact_info "contact@store.com" \
  --category Retail
```

Categories: `Retail` | `Food` | `Services` | `Digital` | `Other`

#### `deactivate_merchant`

Callable by the merchant themselves or the admin.

```bash
stellar contract invoke --id $CONTRACT_ID --source-account <KEY> --network local \
  -- deactivate_merchant --caller <ADDRESS> --merchant_address <ADDRESS>
```

#### `get_merchant`

```bash
stellar contract invoke --id $CONTRACT_ID --source-account <KEY> --network local \
  -- get_merchant --merchant_address <ADDRESS>
```

---

### Payment Processing

#### `process_payment_with_signature`

Transfers tokens from payer to merchant after verifying an ed25519 signature.

```bash
stellar contract invoke --id $CONTRACT_ID --source-account <PAYER_KEY> --network local \
  -- process_payment_with_signature \
  --payer <PAYER_ADDRESS> \
  --order '{"order_id":"ORDER_001","merchant_address":"...","payer":"...","token":"...","amount":1000,"description":"desc","expires_at":0}' \
  --signature <64_BYTE_HEX> \
  --merchant_public_key <32_BYTE_HEX>
```

---

### Payment Queries

#### `get_payment_by_id`

Accessible by the payer, merchant, or admin.

```bash
stellar contract invoke --id $CONTRACT_ID --source-account <KEY> --network local \
  -- get_payment_by_id --caller <ADDRESS> --order_id "ORDER_001"
```

#### `get_merchant_payment_history`

```bash
stellar contract invoke --id $CONTRACT_ID --source-account <MERCHANT_KEY> --network local \
  -- get_merchant_payment_history \
  --merchant <ADDRESS> \
  --cursor null \
  --limit 10 \
  --filter null \
  --sort_field Date \
  --sort_order Descending
```

#### `get_payer_payment_history`

```bash
stellar contract invoke --id $CONTRACT_ID --source-account <PAYER_KEY> --network local \
  -- get_payer_payment_history \
  --payer <ADDRESS> \
  --cursor null \
  --limit 10 \
  --filter '{"amount_min":100,"amount_max":1000,"status":"Any"}' \
  --sort_field Amount \
  --sort_order Ascending
```

**Filter fields** (all optional):

| Field | Type | Description |
|---|---|---|
| `date_start` | `u64` | Unix timestamp lower bound |
| `date_end` | `u64` | Unix timestamp upper bound |
| `amount_min` | `i128` | Minimum payment amount |
| `amount_max` | `i128` | Maximum payment amount |
| `token` | `Address` | Filter by token contract |
| `status` | `StatusFilter` | `Any` \| `Completed` \| `PartiallyRefunded` \| `FullyRefunded` |

**Sort fields**: `Date` | `Amount`  
**Sort orders**: `Ascending` | `Descending`  
**Pagination**: pass the `next_cursor` from the previous response as `cursor`. Max 100 results per page.

#### `get_global_payment_stats` *(admin only)*

```bash
stellar contract invoke --id $CONTRACT_ID --source-account <ADMIN_KEY> --network local \
  -- get_global_payment_stats \
  --admin <ADDRESS> \
  --date_start null \
  --date_end null
```

---

### Refunds

Refund window: **30 days** from `paid_at`. Partial refunds are allowed; cumulative refunds cannot exceed the original amount.

#### `initiate_refund`

Callable by payer or merchant.

```bash
stellar contract invoke --id $CONTRACT_ID --source-account <KEY> --network local \
  -- initiate_refund \
  --caller <ADDRESS> \
  --refund_id "REFUND_001" \
  --order_id "ORDER_001" \
  --amount 500 \
  --reason "Customer request"
```

#### `approve_refund`

Callable by merchant or admin.

```bash
stellar contract invoke --id $CONTRACT_ID --source-account <MERCHANT_KEY> --network local \
  -- approve_refund --caller <ADDRESS> --refund_id "REFUND_001"
```

#### `reject_refund`

```bash
stellar contract invoke --id $CONTRACT_ID --source-account <MERCHANT_KEY> --network local \
  -- reject_refund --caller <ADDRESS> --refund_id "REFUND_001"
```

#### `execute_refund`

Transfers funds from merchant to payer. Requires merchant auth.

```bash
stellar contract invoke --id $CONTRACT_ID --source-account <MERCHANT_KEY> --network local \
  -- execute_refund --refund_id "REFUND_001"
```

#### `get_refund_status`

```bash
stellar contract invoke --id $CONTRACT_ID --source-account <KEY> --network local \
  -- get_refund_status --refund_id "REFUND_001"
```

Returns: `Pending` | `Approved` | `Rejected` | `Completed`

---

### Multi-Signature Payments

Require all listed signers to sign before the payment can be executed.

#### `initiate_multisig_payment`

```bash
stellar contract invoke --id $CONTRACT_ID --source-account <KEY> --network local \
  -- initiate_multisig_payment \
  --initiator <ADDRESS> \
  --payment_id "MS_001" \
  --order '{...}' \
  --required_signers '["<ADDR1>","<ADDR2>"]'
```

#### `sign_multisig_payment`

```bash
stellar contract invoke --id $CONTRACT_ID --source-account <SIGNER_KEY> --network local \
  -- sign_multisig_payment --signer <ADDRESS> --payment_id "MS_001"
```

#### `execute_multisig_payment`

Executes once all required signers have signed.

```bash
stellar contract invoke --id $CONTRACT_ID --source-account <KEY> --network local \
  -- execute_multisig_payment --executor <ADDRESS> --payment_id "MS_001"
```

---

### Admin Config

#### `archive_payment_record`

Removes a payment record from storage.

```bash
stellar contract invoke --id $CONTRACT_ID --source-account <ADMIN_KEY> --network local \
  -- archive_payment_record --admin <ADDRESS> --order_id "ORDER_001"
```

#### `cleanup_expired_payments`

```bash
stellar contract invoke --id $CONTRACT_ID --source-account <ADMIN_KEY> --network local \
  -- cleanup_expired_payments --admin <ADDRESS>
```

#### `set_payment_cleanup_period`

Default: 90 days (7 776 000 seconds).

```bash
stellar contract invoke --id $CONTRACT_ID --source-account <ADMIN_KEY> --network local \
  -- set_payment_cleanup_period --admin <ADDRESS> --period 7776000
```

---

## Events

| Event | Emitted by |
|---|---|
| `admin_set` | `set_admin` |
| `merchant_registered` | `register_merchant` |
| `merchant_deactivated` | `deactivate_merchant` |
| `payment_processed` | `process_payment_with_signature` |
| `refund_initiated` | `initiate_refund` |
| `refund_approved` | `approve_refund` |
| `refund_rejected` | `reject_refund` |
| `refund_executed` | `execute_refund` |
| `multisig_initiated` | `initiate_multisig_payment` |
| `multisig_signed` | `sign_multisig_payment` |
| `multisig_executed` | `execute_multisig_payment` |

---

## Error Codes

| Code | Name | Description |
|---|---|---|
| 1 | `Unauthorized` | Caller lacks permission |
| 2 | `AdminAlreadySet` | Admin already initialised |
| 10 | `MerchantNotFound` | Merchant not registered |
| 11 | `MerchantAlreadyRegistered` | Duplicate registration |
| 12 | `MerchantInactive` | Merchant is deactivated |
| 20 | `PaymentNotFound` | Order ID not found |
| 21 | `PaymentAlreadyExists` | Duplicate order ID |
| 22 | `InvalidAmount` | Amount ≤ 0 |
| 23 | `InvalidSignature` | Signature verification failed |
| 24 | `PaymentExpired` | Order past `expires_at` |
| 25 | `InsufficientBalance` | Merchant balance too low for refund |
| 30 | `RefundNotFound` | Refund ID not found |
| 31 | `RefundAlreadyExists` | Duplicate refund ID |
| 32 | `RefundWindowExpired` | Past 30-day refund window |
| 33 | `RefundAmountExceedsPayment` | Cumulative refund > original amount |
| 34 | `RefundNotApproved` | Refund not in Approved state |
| 35 | `RefundAlreadyCompleted` | Refund already completed or rejected |
| 40 | `MultisigNotFound` | Multisig payment not found |
| 41 | `MultisigAlreadySigned` | Signer already signed |
| 42 | `MultisigAlreadyExecuted` | Payment already executed |
| 43 | `InsufficientSignatures` | Not all required signers have signed |
| 50 | `InvalidInput` | General input validation failure |

---

## TTL Strategy

Soroban persistent storage entries expire after a ledger TTL. Without active renewal, long-lived records (payments, merchants, refunds) would be evicted and become permanently inaccessible.

**Constants** (defined in `storage.rs`):

| Constant | Value | Description |
|---|---|---|
| `TTL_LEDGERS` | 6,307,200 | ~1 year at 5-second ledger close time |
| `TTL_THRESHOLD` | 3,153,600 | ~6 months — refresh TTL when remaining life drops below this |

**Strategy**: every `get_*` and `save_*` call on a persistent entry calls `env.storage().persistent().extend_ttl(key, TTL_THRESHOLD, TTL_LEDGERS)`. This means:

- A record's TTL is reset to ~1 year on every read or write.
- Records that are never accessed will expire after ~1 year and be evicted by the network.
- Frequently accessed records (active merchants, recent payments) are automatically kept alive.
- Instance storage (admin, config, stats) is managed by the Soroban host and does not require manual TTL extension.

---

## Troubleshooting

**Build errors** — ensure the WASM target is installed:
```bash
rustup target add wasm32-unknown-unknown
```

**Local network fails** — restart Docker and the container:
```bash
stellar network container restart local
```

**Test failures** — check `soroban-sdk` version matches `22.0.0` in `Cargo.toml`.

---

## Rate Limiting and Spam Prevention

Pulsar is a Soroban smart contract on Stellar. Stellar's fee market provides natural, protocol-level rate limiting:

- **Resource fees**: every contract invocation pays a fee proportional to the CPU instructions, memory, and storage it consumes. Spamming `process_payment` or `initiate_refund` from a single account rapidly drains that account's XLM balance.
- **Surge pricing**: when the network is congested, base fees rise automatically, making bulk spam economically prohibitive.
- **Sequence number enforcement**: each transaction must use the account's next sequence number, so parallel spam from a single account is serialised by the protocol.

For applications that require stricter per-account throttling (e.g., preventing storage inflation from many small refund initiations), implement rate limiting in an **off-chain API gateway** in front of your contract invocation endpoint:

```
Client → API Gateway (rate limit: N req/min per account) → Stellar RPC → Contract
```

A simple token-bucket or sliding-window limiter keyed on the caller's Stellar address is sufficient. Libraries such as `express-rate-limit` (Node.js) or `slowapi` (Python) can be used for this purpose.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

---

## License

[MIT](LICENSE) © Pulsar Contributors
