# Security Documentation

> **Pulsar Payment Processing Contract — Main Security Reference**
>
> For audit requirements see [docs/security-audit.md](security-audit.md).

---

## Table of Contents

1. [Threat Model](#threat-model)
2. [Authentication Flow](#authentication-flow)
3. [Authorization Model](#authorization-model)
4. [Input Validation Rules](#input-validation-rules)
5. [Encryption Approach](#encryption-approach)
6. [Security Best Practices for Integrators](#security-best-practices-for-integrators)
7. [Security Headers for Off-Chain API Gateways](#security-headers-for-off-chain-api-gateways)
8. [Vulnerability Disclosure Policy](#vulnerability-disclosure-policy)

---

## Threat Model

### Replay Attacks

**Risk:** An attacker intercepts a valid signed payment order and resubmits it to charge the payer a second time.

**Mitigations:**
- Every `order_id` is stored as a tombstone after a payment is processed. Subsequent submissions of the same `order_id` are rejected with `PaymentAlreadyExists` (error 21).
- Tombstones are intentionally retained even after a payment record is archived via `archive_payment_record`. The order ID remains in storage to block replay indefinitely.
- Signed orders carry an `expires_at` Unix timestamp. The contract rejects orders where `expires_at > 0` and `expires_at < env.ledger().timestamp()` with `PaymentExpired` (error 24). Integrators **must** set a short expiry (recommended: ≤ 15 minutes) on all production orders.

### Unauthorized Access

**Risk:** A caller invokes a state-changing function without the appropriate role.

**Mitigations:**
- Every state-changing function calls `env.require_auth(&caller)` / `env.require_auth(&admin)` via Soroban's built-in authorization framework. Transactions that do not carry a valid signature for the required address are rejected at the protocol level before contract code executes.
- Admin-only functions (`set_admin`, `upgrade`, `archive_payment_record`, `cleanup_expired_payments`, `set_payment_cleanup_period`, `get_global_payment_stats`) verify the caller against the stored admin address.
- `deactivate_merchant` accepts either the merchant themselves or the admin, and rejects all other callers.

### Token Manipulation

**Risk:** An attacker substitutes a different token contract address in an order to redirect funds or drain a merchant's balance.

**Mitigations:**
- The token address is embedded in the signed `Order` struct. Because the merchant signs the entire struct (including `token` and `amount`), any field substitution invalidates the ed25519 signature and causes rejection with `InvalidSignature` (error 23).
- Token transfers are executed via Soroban's `transfer` call on the token contract specified in the verified order, so the token cannot be swapped after verification.

### Denial of Service (DoS)

**Risk:** An attacker floods the contract with transactions to exhaust storage, inflate indexes, or block legitimate users.

**Mitigations (on-chain):**
- Stellar's resource fee model charges CPU, memory, and storage fees per invocation. Sustained spam rapidly exhausts the attacker's XLM balance.
- Surge pricing automatically raises fees during network congestion.
- Each Stellar account's transactions are serialised by sequence number, preventing parallel flooding from a single account.

**Mitigations (off-chain — integrator responsibility):**
- Deploy a rate-limiting API gateway in front of contract invocations (see [Rate Limiting](#rate-limiting-and-spam-prevention) in the README and the gateway security section below).
- Monitor merchant/payer payment indexes for abnormal growth and use `archive_payment_record` for housekeeping.

**Storage TTL:**
- Persistent storage entries carry a ~1-year TTL (`TTL_LEDGERS = 6,307,200`). Entries that are never accessed expire and are evicted by the network, capping long-term storage accumulation.

### Key Compromise

**Risk:** A merchant's ed25519 signing key or a Stellar account secret key is exposed.

**Mitigations and recommended response:**
1. **Merchant signing key** — immediately deactivate the merchant account via `deactivate_merchant`. This prevents new payments from being processed for that merchant. Rotate the key by registering a new merchant account.
2. **Admin key** — the admin key has elevated privileges. Store it in a hardware security module (HSM) or a secrets manager. If compromised, no in-contract key rotation exists in the current version; a contract upgrade (`upgrade`) performed from a secondary recovery key should be planned for before deployment.
3. **Payer key** — a compromised payer key allows an attacker to approve multisig payments on behalf of the payer. Revoke Stellar account signers at the protocol level immediately.
4. **General hygiene** — never commit secret keys to source control; use environment variables or a vault service; rotate keys on any suspected exposure.

---

## Authentication Flow

### ed25519 Merchant Signature

`process_payment_with_signature` requires a valid ed25519 signature over the serialized `Order` struct:

```
signature_valid = ed25519_verify(
    public_key  = merchant_public_key,   // 32-byte key passed by caller
    message     = sha256(order_bytes),   // canonical XDR/JSON serialisation
    signature   = signature              // 64-byte signature passed by caller
)
```

1. The merchant constructs the `Order` struct off-chain (order_id, merchant_address, payer, token, amount, description, expires_at).
2. The merchant signs the struct with their ed25519 private key and sends the signature to the payer (or a backend service).
3. The payer (or backend) calls `process_payment_with_signature`, supplying the order, signature, and the merchant's public key.
4. The contract verifies the signature using `env.crypto().ed25519_verify()`. Failure returns `InvalidSignature` (error 23).
5. The contract checks `expires_at` and the `order_id` tombstone before executing the token transfer.

### Soroban `require_auth`

All state-changing entry points invoke `env.require_auth(&address)` for the relevant principal:

| Function | `require_auth` target |
|---|---|
| `register_merchant` | `merchant_address` |
| `deactivate_merchant` | `caller` (merchant or admin) |
| `process_payment_with_signature` | `payer` |
| `initiate_refund` | `caller` (payer or merchant) |
| `approve_refund` / `reject_refund` | `caller` (merchant or admin) |
| `execute_refund` | merchant of the associated payment |
| `initiate_multisig_payment` | `initiator` |
| `sign_multisig_payment` | `signer` |
| `execute_multisig_payment` | `executor` |
| `set_admin` / `upgrade` / admin config | `admin` |

Soroban enforces auth at the host level; a transaction missing the required account signature is rejected before the contract function body runs.

### Admin Checks

Admin-sensitive functions load the stored admin address and assert `caller == stored_admin`. This is in addition to `require_auth`, providing a defence-in-depth check at the application layer.

`set_admin` can only be called once (guarded by `AdminAlreadySet`, error 2). The deployer must call `set_admin` immediately after deployment.

---

## Authorization Model

| Role | Identity | Permitted operations |
|---|---|---|
| **Admin** | Single address stored at `set_admin` | `set_admin` (once), `upgrade`, `archive_payment_record`, `cleanup_expired_payments`, `set_payment_cleanup_period`, `get_global_payment_stats`, `get_merchant_stats` (any merchant), `deactivate_merchant` (any merchant), `approve_refund`, `reject_refund` |
| **Merchant** | Any registered, active merchant address | `register_merchant` (own), `deactivate_merchant` (own), `initiate_refund` (own payments), `approve_refund` (own payments), `reject_refund` (own payments), `execute_refund` (own payments), `get_merchant_payment_history` (own), `get_merchant_stats` (own) |
| **Payer** | Any Stellar address | `process_payment_with_signature`, `initiate_refund` (own payments), `get_payer_payment_history` (own), `get_payment_by_id` (own payments) |
| **Any authenticated caller** | Any Stellar address | `ping`, `get_version`, `get_merchant`, `get_refund_status`, `initiate_multisig_payment`, `sign_multisig_payment` (if listed as required signer), `execute_multisig_payment` (if all signers signed) |

**Principle of least privilege:** read-only queries for public data (merchant info, contract version) do not require `require_auth`. All writes and sensitive reads are gated by role checks.

---

## Input Validation Rules

The contract enforces the following at runtime. Integrators should mirror these checks client-side to provide early feedback.

| Field | Rule | Error on violation |
|---|---|---|
| `amount` | Must be > 0 | `InvalidAmount` (22) |
| `merchant_address` | Must be a registered, active merchant | `MerchantNotFound` (10) / `MerchantInactive` (12) |
| `payer` | Must be a valid Stellar address; `require_auth` enforced | `Unauthorized` (1) |
| `signature` | Must be exactly 64 bytes; must verify against `merchant_public_key` and order payload | `InvalidSignature` (23) |
| `merchant_public_key` | Must be exactly 32 bytes (ed25519 public key) | `InvalidSignature` (23) / `InvalidInput` (50) |
| `order_id` | Must be globally unique (tombstone check) | `PaymentAlreadyExists` (21) |
| `expires_at` | `0` = never expires; any other value must be ≥ current ledger timestamp | `PaymentExpired` (24) |
| `refund_id` | Must be globally unique | `RefundAlreadyExists` (31) |
| `refund amount` | Must be > 0 and cumulative refunds ≤ original payment amount | `InvalidAmount` (22) / `RefundAmountExceedsPayment` (33) |
| `refund window` | `paid_at + 30 days + 1-hour grace` must be ≥ current ledger timestamp | `RefundWindowExpired` (32) |
| `required_signers` (multisig) | Must be non-empty; each address must be distinct | `InvalidInput` (50) |
| `limit` (pagination) | Must be between 1 and 100 inclusive | `InvalidInput` (50) |

**Integrator guidance:** always set a short, non-zero `expires_at` (e.g., `now + 900` seconds) on production orders. A value of `0` disables expiry enforcement and increases replay risk if a tombstone is somehow lost.

---

## Encryption Approach

### Payment Authorization — ed25519 Signatures

Pulsar does not encrypt payment data. Instead it uses **ed25519 digital signatures** to authenticate and authorise payment orders:

- ed25519 provides 128-bit security with 32-byte public keys and 64-byte signatures.
- The signature covers the entire `Order` struct, binding every field (including amount, token, payer, and merchant) to the authorisation. Altering any field invalidates the signature.
- Key generation and signing are the merchant's responsibility. Merchants must protect their ed25519 private key with the same diligence as their Stellar secret key.

### Transport Security — Stellar Network TLS

- All communication between clients and Stellar RPC / Horizon nodes is over HTTPS (TLS 1.2+).
- Contract invocation payloads are signed Stellar transactions transmitted over the Stellar peer-to-peer network, which uses its own TLS-secured overlay protocol.
- Integrators must ensure their RPC endpoint URLs use `https://` and validate TLS certificates; never connect to an RPC node over plain HTTP in production.

### Storage

- Contract storage on Soroban is stored on the Stellar ledger, which is public. Do not store sensitive personal data (PII, private keys, passwords) in contract storage fields such as `description` or `contact_info`.

---

## Security Best Practices for Integrators

### Key Management

- Generate merchant ed25519 keys using a cryptographically secure random number generator (e.g., `libsodium`, `openssl genpkey`).
- Store private keys in a secrets manager (AWS Secrets Manager, HashiCorp Vault, GCP Secret Manager) or an HSM. Never hard-code keys.
- Rotate keys periodically. After rotation: deactivate the old merchant account and register a new one.
- Use separate keys for testnet and mainnet.

### Order Construction

- Always set a short, non-zero `expires_at` — recommended ≤ 15 minutes from order creation time.
- Generate `order_id` values using a UUID v4 or a cryptographically random identifier to prevent enumeration and collisions.
- Construct and sign orders server-side; never expose the ed25519 private key to browser/mobile clients.

### Admin Key Security

- The admin key has the highest privilege in the contract. Protect it with multi-factor authentication and an HSM if possible.
- Consider a multi-sig Stellar account as the admin address so that admin operations require M-of-N approvals.
- Document and test the key-rotation and upgrade procedure before going to production.

### Off-Chain Gateway

- Place a rate-limiting API gateway between external clients and your Stellar RPC endpoint.
- Validate all inputs (amounts, addresses, signatures) in the gateway before forwarding to the network, to save fees on obviously invalid transactions.
- Log all transaction attempts with timestamps and caller addresses for audit trails.

### Dependency Management

- Pin `soroban-sdk` and all other dependencies to exact versions in `Cargo.toml`.
- Run `cargo audit` regularly and in CI to detect known vulnerabilities.
- Review dependency changelogs before upgrading.

### Monitoring and Alerting

- Monitor on-chain events (`payment_processed`, `refund_initiated`, `merchant_deactivated`) via a Stellar event indexer.
- Alert on unusual payment volumes, repeated failed signature verifications (possible attack probe), or unexpected admin actions.
- Set up alerts for admin key usage outside of scheduled maintenance windows.

---

## Security Headers for Off-Chain API Gateways

If you expose a REST or GraphQL API that proxies contract invocations, configure the following HTTP security headers:

```
# Prevent clickjacking
X-Frame-Options: DENY

# Block MIME-type sniffing
X-Content-Type-Options: nosniff

# Enable browser XSS filter (legacy support)
X-XSS-Protection: 1; mode=block

# Enforce HTTPS
Strict-Transport-Security: max-age=31536000; includeSubDomains; preload

# Restrict browser features
Permissions-Policy: geolocation=(), microphone=(), camera=()

# Content Security Policy (adjust for your UI)
Content-Security-Policy: default-src 'self'; script-src 'self'; object-src 'none'

# Control referrer information
Referrer-Policy: strict-origin-when-cross-origin

# CORS — restrict to known origins
Access-Control-Allow-Origin: https://your-app-domain.com
Access-Control-Allow-Methods: POST, GET, OPTIONS
Access-Control-Allow-Headers: Content-Type, Authorization
```

### Rate Limiting

Configure a token-bucket or sliding-window rate limiter keyed on the caller's Stellar address:

```
# Example: 60 requests per minute per address
Rate-Limit-Policy: 60;w=60
```

Recommended limits (adjust to your traffic profile):

| Endpoint | Limit |
|---|---|
| `process_payment_with_signature` | 10 req/min per address |
| `initiate_refund` | 5 req/min per address |
| Read-only queries | 120 req/min per address |

### TLS Configuration

- Minimum TLS version: **1.2**; prefer **1.3**.
- Use strong cipher suites; disable RC4, DES, 3DES, and export ciphers.
- Obtain certificates from a trusted CA; automate renewal (e.g., Let's Encrypt with certbot).

---

## Vulnerability Disclosure Policy

### Scope

This policy covers security vulnerabilities in:
- The Pulsar smart contract (`contracts/payment-processing-contract/`)
- Supporting scripts and tooling in this repository

Out of scope: vulnerabilities in the Stellar/Soroban protocol itself (report those to the [Stellar Bug Bounty Program](https://hackerone.com/stellar)).

### How to Report

**Please do not open a public GitHub issue for security vulnerabilities.**

Report vulnerabilities privately by emailing the maintainers. Include:

1. A description of the vulnerability and its potential impact.
2. Steps to reproduce or a proof-of-concept (smart contract unit test preferred).
3. The affected contract function(s) and error conditions.
4. Any suggested mitigations.

Contact: open a [GitHub Security Advisory](https://github.com/Hellenjoseph/pulsar-contracts/security/advisories/new) on this repository (Settings → Security → Advisories → New draft advisory). This keeps the report confidential until a fix is released.

### Response Timeline

| Milestone | Target |
|---|---|
| Acknowledgement | Within 2 business days |
| Initial assessment | Within 5 business days |
| Fix or mitigation | Within 30 days for critical/high; 90 days for medium/low |
| Public disclosure | After fix is deployed and users have had time to upgrade |

### Severity Classification

| Severity | Examples |
|---|---|
| **Critical** | Funds theft, signature bypass, admin key takeover |
| **High** | Replay attack vector, unauthorized refund execution |
| **Medium** | DoS via storage inflation, information disclosure |
| **Low** | Minor input validation gaps, misleading error messages |

### Responsible Disclosure

We request that reporters:
- Allow us the response timeline above before public disclosure.
- Avoid testing on mainnet or testnet accounts not owned by the reporter.
- Do not exploit any vulnerability beyond what is necessary to demonstrate it.

We commit to:
- Acknowledging receipt promptly.
- Keeping reporters informed of progress.
- Crediting reporters in the release notes (unless anonymity is requested).
- Not pursuing legal action against good-faith researchers following this policy.
