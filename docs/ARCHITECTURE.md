# Pulsar — System Architecture

> Architecture reference for the Pulsar payment-processing smart contract on Stellar Soroban.

---

## Table of Contents

1. [System Architecture Diagram](#1-system-architecture-diagram)
2. [Component Responsibilities](#2-component-responsibilities)
3. [Data Flow Diagram](#3-data-flow-diagram)
4. [Deployment Architecture](#4-deployment-architecture)
5. [Storage Schema](#5-storage-schema)
6. [API Gateway Design](#6-api-gateway-design)
7. [Caching Strategy](#7-caching-strategy)
8. [Sequence Diagrams](#8-sequence-diagrams)

---

## 1. System Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            EXTERNAL ACTORS                                  │
│                                                                             │
│   ┌───────────┐   ┌───────────┐   ┌───────────┐   ┌──────────────────┐    │
│   │   Payer   │   │ Merchant  │   │   Admin   │   │  Off-chain       │    │
│   │ (Stellar  │   │ (Stellar  │   │ (Stellar  │   │  Scheduler /     │    │
│   │  account) │   │  account) │   │  account) │   │  Indexer svc     │    │
│   └─────┬─────┘   └─────┬─────┘   └─────┬─────┘   └────────┬─────────┘    │
└─────────┼───────────────┼───────────────┼──────────────────┼──────────────┘
          │               │               │                  │
          ▼               ▼               ▼                  ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                         API / ACCESS LAYER                                  │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │  Optional: Off-chain API Gateway                                    │  │
│   │  (rate limiting, auth, request routing)                             │  │
│   └──────────────────────────┬──────────────────────────────────────────┘  │
│                              │                                              │
│   ┌──────────────────────────▼──────────────────────────────────────────┐  │
│   │  Stellar RPC / Horizon API                                          │  │
│   │  (transaction submission, event streaming, account queries)         │  │
│   └──────────────────────────┬──────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                    STELLAR / SOROBAN NETWORK                                │
│                                                                             │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │               Pulsar Smart Contract (WASM)                           │  │
│  │                                                                      │  │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐  ┌──────────────┐  │  │
│  │  │  lib.rs    │  │  helper.rs │  │  storage.rs│  │  error.rs    │  │  │
│  │  │ (entry pts)│  │ (auth/val) │  │ (persist.) │  │ (error enum) │  │  │
│  │  └────────────┘  └────────────┘  └─────┬──────┘  └──────────────┘  │  │
│  │                                         │                            │  │
│  │  ┌──────────────────────────────────────▼──────────────────────┐    │  │
│  │  │                   types.rs (data structures)                │    │  │
│  │  └─────────────────────────────────────────────────────────────┘    │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │             Soroban Persistent Storage (ledger entries)              │  │
│  │  Merchant │ Payment │ Refund │ Multisig │ Subscription │ Indexes    │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │             Stellar Token Contracts (SEP-41 / native XLM)           │  │
│  │             (cross-contract token transfer calls)                   │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Component Responsibilities

### Contract source modules

| Module | File | Responsibility |
|---|---|---|
| Entry points | `lib.rs` | Declares all public contract functions; orchestrates calls to helper and storage layers; emits events |
| Types | `types.rs` | All `#[contracttype]` structs and enums: `Merchant`, `PaymentOrder`, `PaymentRecord`, `RefundRecord`, `MultisigPayment`, `SubscriptionState`, query helpers, stats, `DataKey` |
| Storage | `storage.rs` | Thin read/write wrappers over `env.storage().persistent()` and `env.storage().instance()`; owns TTL constants and all TTL extension calls |
| Helpers | `helper.rs` | Authorization checks, input validation, pagination/filtering logic, signature verification utilities |
| Errors | `error.rs` | `ContractError` / `PaymentError` enum with numeric codes (1–50) used across the contract |
| Tests | `test.rs` | Unit and integration tests using `soroban-sdk` testutils and mock auth |

### External components

| Component | Responsibility |
|---|---|
| Stellar Validator quorum | Closes ledgers; provides monotonically increasing `close_time` timestamps used for refund deadline enforcement |
| Stellar Token Contract | SEP-41-compatible token; executes `transfer` calls during payment and refund execution |
| Off-chain scheduler | Triggers recurring subscription payments by calling `process_subscription_payment` at interval boundaries (contract cannot self-schedule) |
| Off-chain indexer | Listens to contract events for analytics, dashboards, and complex historical queries (planned, BE-001) |
| API Gateway | Optional layer between clients and Stellar RPC; enforces per-account rate limiting and request authentication |

---

## 3. Data Flow Diagram

```
 Payer                  API Gateway           Stellar RPC          Pulsar Contract
   │                        │                      │                      │
   │── submit tx ──────────►│                      │                      │
   │                        │── forward tx ───────►│                      │
   │                        │                      │── invoke ───────────►│
   │                        │                      │                      │── verify_auth()
   │                        │                      │                      │── check_whitelist()
   │                        │                      │                      │── verify_ed25519_sig()
   │                        │                      │                      │── token::transfer(
   │                        │                      │    ◄─ xfer call ─────│     payer→merchant)
   │                        │                      │                      │── save_payment()
   │                        │                      │                      │── push_indexes()
   │                        │                      │                      │── update_stats()
   │                        │                      │                      │── emit event
   │                        │                      │◄── result ───────────│
   │                        │◄── tx result ────────│                      │
   │◄── response ───────────│                      │                      │
   │                        │                      │                      │

 Merchant               API Gateway           Stellar RPC          Pulsar Contract
   │                        │                      │                      │
   │── approve_refund ─────►│── forward ──────────►│── invoke ───────────►│
   │                        │                      │                      │── get_refund()
   │                        │                      │                      │── check_status==Pending
   │                        │                      │                      │── save_refund(Approved)
   │                        │                      │                      │── emit refund_approved
   │◄── result ─────────────│◄─────────────────────│◄────────────────────│

   │── execute_refund ─────►│── forward ──────────►│── invoke ───────────►│
   │                        │                      │                      │── check_window
   │                        │                      │                      │── token::transfer(
   │                        │                      │    ◄─ xfer call ─────│     merchant→payer)
   │                        │                      │                      │── save_refund(Completed)
   │                        │                      │                      │── update_stats()
   │◄── result ─────────────│◄─────────────────────│◄────────────────────│
```

---

## 4. Deployment Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│                        Developer machine                         │
│                                                                  │
│  cargo build --target wasm32-unknown-unknown --release           │
│       └─► payment_processing_contract.wasm                       │
│                                                                  │
│  stellar contract upload --wasm <file> --network <net>           │
│       └─► wasm_hash (32-byte hex)                                │
│                                                                  │
│  stellar contract deploy --wasm <file> --network <net>           │
│       └─► CONTRACT_ID (contract address)                         │
│                                                                  │
│  stellar contract invoke --id $CONTRACT_ID -- set_admin          │
│       └─► contract initialised, admin set                        │
└──────────────────────────────────────────────────────────────────┘

Environments:

  Local (Docker Compose)            Testnet                  Mainnet
  ────────────────────────          ──────────────────────   ──────────────────────
  stellar-local container           testnet.stellar.org RPC  mainnet.stellar.org RPC
  Horizon at :8000                  Friendbot (faucet)       Real XLM required
  Fast iteration / CI               Integration testing      Production
  docker compose up / down          stellar --network testnet stellar --network public

Upgrade path (in-place WASM upgrade):
  1. cargo build  →  new .wasm
  2. stellar contract upload  →  new wasm_hash
  3. contract invoke -- upgrade --new_wasm_hash <hash>
  4. contract invoke -- get_version  (verify)
  Storage and CONTRACT_ID are preserved across upgrades.
```

---

## 5. Storage Schema

Pulsar uses **Soroban persistent storage** keyed by `DataKey` enum variants. All persistent entries are extended to a ~1-year TTL on every read or write (see [Caching Strategy](#7-caching-strategy)).

### Instance storage (contract-global, managed by Soroban host)

| Key | Type | Description |
|---|---|---|
| `DataKey::Admin` | `Address` | Single admin address set at initialisation |
| `DataKey::AdminConfig` | `AdminConfig` | Multi-admin list + threshold |
| `DataKey::ContractVersion` | `u32` | Incremented on each upgrade |
| `DataKey::GlobalStats` | `GlobalStats` | Aggregate payment/refund counters |
| `DataKey::CleanupPeriod` | `u64` | Seconds before a payment is eligible for cleanup (default 90 days) |
| `DataKey::DefaultMultisigExpiry` | `u64` | Default multisig payment TTL in seconds (default 24 h) |
| `DataKey::WhitelistEnabled` | `bool` | Whether merchant whitelist enforcement is active |
| `DataKey::TokenAllowlistEnabled` | `bool` | Whether token allowlist enforcement is active |

### Persistent storage (per-record)

| Key | Type | Description |
|---|---|---|
| `DataKey::Merchant(Address)` | `Merchant` | Merchant profile: name, category, active flag, optional signing key |
| `DataKey::Payment(Bytes)` | `PaymentRecord` | Full payment record keyed by `order_id` |
| `DataKey::ArchivedPayment(Bytes)` | `bool` | Tombstone: prevents replay of archived order IDs |
| `DataKey::Refund(Bytes)` | `RefundRecord` | Refund record keyed by `refund_id` |
| `DataKey::Multisig(Bytes)` | `MultisigPayment` | Multisig payment state keyed by `payment_id` |
| `DataKey::Subscription(Bytes)` | `SubscriptionState` | Recurring subscription keyed by `subscription_id` |
| `DataKey::MerchantStats(Address)` | `MerchantStats` | Cached per-merchant aggregate stats |
| `DataKey::Whitelist(Address)` | `bool` | Whether an address is whitelisted |
| `DataKey::AllowedToken(Address)` | `bool` | Whether a token contract is on the allowlist |
| `DataKey::OrderRefundCount(Bytes)` | `u32` | Number of pending refunds for an order (max `MAX_PENDING_REFUNDS = 10`) |

### Index lists (persistent)

| Key | Type | Description |
|---|---|---|
| `DataKey::MerchantPayments(Address)` | `Vec<Bytes>` | Ordered list of `order_id`s for a merchant (used for paginated history) |
| `DataKey::PayerPayments(Address)` | `Vec<Bytes>` | Ordered list of `order_id`s for a payer |
| `DataKey::GlobalPaymentIndex` | `Vec<Bytes>` | Ordered global list of all `order_id`s |
| `DataKey::AllRefunds` | `Vec<Bytes>` | Ordered global list of all `refund_id`s |

### Key data structures

```
Merchant {
  address:          Address
  name:             String
  description:      String
  contact_info:     String
  category:         MerchantCategory  // Retail|Food|Services|Digital|Other
  active:           bool
  registered_at:    u64               // ledger timestamp
  signing_public_key: Option<BytesN<32>>
}

PaymentRecord {
  order_id:              Bytes
  merchant_address:      Address
  payer:                 Address
  token:                 Address       // SEP-41 token contract
  amount:                i128
  refunded_amount:       i128
  pending_refund_amount: i128
  status:                PaymentStatus // Completed|PartiallyRefunded|FullyRefunded
  paid_at:               u64           // ledger timestamp
  description:           String
}

RefundRecord {
  refund_id:      Bytes
  order_id:       Bytes
  amount:         i128
  reason:         String
  status:         RefundStatus  // Pending|Approved|Rejected|Completed|Disputed
  initiated_by:   Address
  initiated_at:   u64
  dispute_reason: String
}

MultisigPayment {
  payment_id:       Bytes
  order:            PaymentOrder
  required_signers: Vec<Address>
  signatures:       Vec<Address>  // signers who have signed so far
  executed:         bool
  expires_at:       u64
  created_at:       u64
}

SubscriptionState {
  subscription_id: Bytes
  payer:           Address
  merchant:        Address
  plan:            SubscriptionPlan { interval: u64, amount: i128, token: Address }
  status:          SubscriptionStatus  // Active|Cancelled
  created_at:      u64
  last_charged_at: u64
}

GlobalStats  { total_payments: u64, total_volume: i128,
               total_refunds: u64,  total_refund_volume: i128 }
MerchantStats { merchant_address: Address, total_payments: u64, total_volume: i128,
                total_refunds: u64, total_refund_volume: i128 }
AdminConfig  { admins: Vec<Address>, threshold: u32 }
```

---

## 6. API Gateway Design

Pulsar itself is a pure on-chain contract; all calls are standard Stellar transactions. An optional off-chain API gateway sits between clients and the Stellar RPC to provide application-level concerns.

```
┌────────────┐     HTTPS      ┌─────────────────────────────────────────┐
│  Client    │───────────────►│           API Gateway                   │
│ (browser / │                │                                         │
│  mobile /  │                │  ┌────────────────────────────────────┐ │
│  backend)  │◄───────────────│  │ 1. Authentication                  │ │
└────────────┘                │  │    JWT / API-key validation        │ │
                              │  ├────────────────────────────────────┤ │
                              │  │ 2. Rate Limiting                   │ │
                              │  │    Token-bucket per Stellar addr   │ │
                              │  │    e.g. 60 req/min per account     │ │
                              │  ├────────────────────────────────────┤ │
                              │  │ 3. Request routing                 │ │
                              │  │    Read queries → Horizon REST     │ │
                              │  │    Write txns  → Stellar RPC       │ │
                              │  ├────────────────────────────────────┤ │
                              │  │ 4. Response caching                │ │
                              │  │    Idempotent reads cached in Redis│ │
                              │  │    TTL matched to ledger close time│ │
                              │  └────────────────────────────────────┘ │
                              └────────────────┬────────────────────────┘
                                               │
                              ┌────────────────▼────────────────────────┐
                              │         Stellar RPC / Horizon           │
                              └────────────────┬────────────────────────┘
                                               │
                              ┌────────────────▼────────────────────────┐
                              │         Pulsar Smart Contract           │
                              └─────────────────────────────────────────┘
```

### Rate limiting strategy

- Key: caller Stellar address (extracted from transaction source or function argument)
- Algorithm: sliding-window or token-bucket (e.g., `express-rate-limit` / `slowapi`)
- Thresholds (suggested): 60 read requests/min, 10 write transactions/min per address
- Beyond the gateway: Stellar's own fee market provides protocol-level surge pricing for spam on-chain

### Recommended gateway responsibilities

| Concern | Mechanism |
|---|---|
| Auth | API key or OAuth JWT; map to Stellar address for rate-limit key |
| Rate limiting | Per-address sliding window; return HTTP 429 with `Retry-After` |
| Read caching | Cache `get_payment_by_id`, `get_merchant`, paginated history responses for 1 ledger close (~5 s) |
| Write routing | Sign transactions server-side or relay pre-signed client transactions |
| Event streaming | Subscribe to Horizon `GET /transactions` or event stream for contract events |

---

## 7. Caching Strategy

### On-chain TTL management

Every persistent storage read and write automatically extends the entry's TTL:

```
TTL_LEDGERS   = 6_307_200 ledgers  (~1 year at 5-second close time)
TTL_THRESHOLD = 3_153_600 ledgers  (~6 months)

On each get_* or save_*:
  env.storage().persistent().extend_ttl(key, TTL_THRESHOLD, TTL_LEDGERS)
  → resets the entry to ~1 year remaining life
```

Instance storage (Admin, GlobalStats, config flags) is bumped with `bump_instance_ttl()` on every invocation.

Effect:
- Frequently accessed records (active merchants, recent payments) are kept alive automatically.
- Inactive records expire after ~1 year and are evicted by the network.
- No manual renewal cron job is required for active data.

### On-chain stats cache

`GlobalStats` and `MerchantStats` are **write-through caches** maintained in instance/persistent storage respectively:

| Cache entry | Updated on | Query cost |
|---|---|---|
| `GlobalStats` | Every `process_payment` and `execute_refund` | O(1) read |
| `MerchantStats(addr)` | Every payment/refund for that merchant | O(1) read (unfiltered) |
| Date-filtered merchant stats | Computed on demand | O(n) scan of merchant payment list |

### Off-chain caching (API gateway layer)

| Data | Suggested TTL | Invalidation |
|---|---|---|
| `get_merchant` | 30 s | On `merchant_registered` or `merchant_deactivated` event |
| `get_payment_by_id` | 60 s | On `refund_executed` event (status change) |
| Paginated history | 5 s (1 ledger) | Time-based expiry |
| `get_global_payment_stats` | 10 s | Time-based expiry |
| `get_merchant_stats` (unfiltered) | 10 s | On payment/refund event for that merchant |

---

## 8. Sequence Diagrams

### 8.1 Signed Payment (`process_payment_with_signature`)

```
 Payer          Merchant (off-chain)    Pulsar Contract       Token Contract
   │                    │                      │                     │
   │── request order ──►│                      │                     │
   │◄── PaymentOrder + ─│                      │                     │
   │    ed25519 sig      │                      │                     │
   │                    │                      │                     │
   │── invoke process_payment_with_signature ──►│                     │
   │   (payer, order, signature, merchant_pubkey)                    │
   │                    │                      │                     │
   │                    │            check payer auth                │
   │                    │            check merchant active           │
   │                    │            check order not expired         │
   │                    │            check order_id not duplicate    │
   │                    │            verify ed25519(sig, order)      │
   │                    │            check token allowlist           │
   │                    │                      │                     │
   │                    │                      │── transfer(payer,  ►│
   │                    │                      │   merchant, amount) │
   │                    │                      │◄── ok ──────────────│
   │                    │                      │                     │
   │                    │            save PaymentRecord              │
   │                    │            push MerchantPayments index     │
   │                    │            push PayerPayments index        │
   │                    │            push GlobalPaymentIndex         │
   │                    │            increment GlobalStats           │
   │                    │            increment MerchantStats         │
   │                    │            emit payment_processed event    │
   │◄── success ─────────────────────│                     │
```

### 8.2 Refund Flow

```
 Payer / Merchant       Pulsar Contract          Token Contract
        │                      │                       │
        │── initiate_refund ──►│                       │
        │   (refund_id,         │                       │
        │    order_id,          │                       │
        │    amount, reason)    │                       │
        │                      │── check caller is payer or merchant
        │                      │── check refund window (paid_at + 30d + 1h)
        │                      │── check amount + existing refunds ≤ payment amount
        │                      │── check pending refund count < MAX_PENDING_REFUNDS
        │                      │── save RefundRecord(Pending)
        │                      │── emit refund_initiated
        │◄── ok ───────────────│                       │
        │                      │                       │
 Merchant                      │                       │
        │── approve_refund ───►│                       │
        │   (refund_id)         │                       │
        │                      │── check caller is merchant or admin
        │                      │── check status == Pending
        │                      │── save RefundRecord(Approved)
        │                      │── emit refund_approved
        │◄── ok ───────────────│                       │
        │                      │                       │
        │── execute_refund ───►│                       │
        │   (refund_id)         │                       │
        │                      │── check status == Approved
        │                      │── re-check refund window
        │                      │── require merchant auth
        │                      │                       │
        │                      │── transfer(merchant, ►│
        │                      │   payer, amount)      │
        │                      │◄── ok ────────────────│
        │                      │                       │
        │                      │── save RefundRecord(Completed)
        │                      │── update PaymentRecord.refunded_amount
        │                      │── update PaymentRecord.status
        │                      │── increment GlobalStats.total_refunds
        │                      │── increment MerchantStats.total_refunds
        │                      │── emit refund_executed
        │◄── ok ───────────────│                       │
```

### 8.3 Multi-Signature Payment

```
 Initiator         Signer A         Signer B        Pulsar Contract   Token Contract
     │                 │                │                  │                │
     │── initiate_multisig_payment ────────────────────►  │                │
     │   (payment_id, order,            │                  │                │
     │    required_signers=[A,B])       │                  │                │
     │                 │                │         check initiator auth       │
     │                 │                │         save MultisigPayment       │
     │                 │                │         (executed=false,sigs=[])  │
     │                 │                │         emit multisig_initiated   │
     │◄── ok ──────────────────────────────────────│                │
     │                 │                │                  │                │
     │── sign_multisig_payment ────────────────────────►  │                │
     │   (signer=A)     │                │                  │                │
     │                 │                │         check signer auth (A)     │
     │                 │                │         check A in required_signers
     │                 │                │         check A not already signed
     │                 │                │         append A to signatures    │
     │                 │                │         emit multisig_signed      │
     │◄── ok ──────────────────────────────────────│                │
     │                 │                │                  │                │
     │                 │── sign_multisig_payment ────────►│                │
     │                 │   (signer=B)    │                  │                │
     │                 │                │         check signer auth (B)     │
     │                 │                │         append B to signatures    │
     │                 │                │         emit multisig_signed      │
     │                 │◄── ok ──────────────────────────│                │
     │                 │                │                  │                │
     │── execute_multisig_payment ─────────────────────►  │                │
     │   (executor)     │                │                  │                │
     │                 │                │         check all required signed │
     │                 │                │         check not expired         │
     │                 │                │         check not already executed│
     │                 │                │                  │── transfer ───►│
     │                 │                │                  │◄── ok ─────────│
     │                 │                │         save MultisigPayment      │
     │                 │                │         (executed=true)           │
     │                 │                │         save PaymentRecord        │
     │                 │                │         update indexes + stats    │
     │                 │                │         emit multisig_executed    │
     │◄── ok ──────────────────────────────────────│                │
```

### 8.4 Subscription Payment (off-chain scheduler driven)

```
 Off-chain Scheduler    Pulsar Contract         Token Contract
         │                     │                      │
         │  (at interval)      │                      │
         │                     │                      │
         │── process_subscription_payment ───────────►│
         │   (subscription_id) │                      │
         │                     │── get_subscription() │
         │                     │── check status == Active
         │                     │── check now >= last_charged_at + interval
         │                     │── check not duplicate
         │                     │                      │
         │                     │── transfer(payer,   ►│
         │                     │   merchant, amount)  │
         │                     │◄── ok ───────────────│
         │                     │                      │
         │                     │── update last_charged_at = now
         │                     │── save PaymentRecord
         │                     │── update indexes + stats
         │◄── ok ──────────────│                      │

NOTE: The contract enforces idempotency and interval guards.
The scheduler is responsible for timely invocation — the contract
cannot autonomously schedule future calls.
```

---

*Last updated: 2026-06-29*
