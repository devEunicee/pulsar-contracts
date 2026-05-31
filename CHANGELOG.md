# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Pause / emergency stop mechanism (`pause` / `unpause` / `is_paused`) with
  `ContractPaused` error and `contract_paused` / `contract_unpaused` events
  (SEC-010).
- WASM binary size gate in CI: fails at 100 KB, warns at 80 KB, reports size
  in the GitHub Actions step summary (DO-003).
- Real ed25519 key-pair signature tests: `test_real_ed25519_valid_signature_succeeds`
  and `test_real_ed25519_tampered_signature_fails` exercise the actual
  `ed25519_verify` path without `mock_all_auths` bypass (T-001).

### Changed
- Remove unused `StorageError` variant from the payment contract ABI.
- Harden payment and refund flows against external token transfer re-entrancy
  by committing state before external calls.
- Add best-effort zero/burn admin address validation and test coverage for
  invalid admin assignment.
- Add date-filter coverage for `get_global_payment_stats` to cover
  all-payments and no-payments ranges.

---

## [0.1.0] - 2024-01-01

### Added
- **Admin management** — one-time `set_admin` initialisation; `upgrade` for
  WASM upgrades; `get_version` for on-chain version tracking.
- **Merchant management** — `register_merchant` with name / description /
  contact-info field validation; `deactivate_merchant` (self or admin);
  `get_merchant` query; optional admin-whitelist mode
  (`set_whitelist_mode`, `approve_merchant_registration`).
- **Payment processing** — `process_payment_with_signature` with ed25519
  signature verification over the full XDR-serialised `PaymentOrder`; payer
  mismatch guard; duplicate-payment guard; expiry check.
- **Payment queries** — `get_payment_by_id` (payer / merchant / admin);
  `get_merchant_payment_history` and `get_payer_payment_history` with
  cursor-based pagination, date / amount / token / status filters, and
  ascending / descending sort by date or amount; `get_global_payment_stats`
  with optional date-range filtering.
- **Payment management** — `archive_payment_record` (admin); 
  `cleanup_expired_payments` (admin) with configurable retention period
  (`set_payment_cleanup_period`).
- **Refund workflow** — `initiate_refund` → `approve_refund` / `reject_refund`
  → `execute_refund` with 30-day refund window, pending-amount tracking to
  prevent over-refund, and `get_refund_status` query.
- **Multi-signature payments** — `initiate_multisig_payment` (funds locked in
  contract escrow); `sign_multisig_payment`; `execute_multisig_payment`
  (releases escrow to merchant); configurable expiry
  (`set_default_multisig_expiry`); duplicate-signer guard.
- **Global statistics** — cumulative `total_payments`, `total_volume`,
  `total_refunds`, `total_refund_volume` with overflow protection.
- **Storage** — persistent TTL management (~1 year) with automatic refresh;
  chunked payment-index lists for merchants, payers, and global index.
- **Events** — `admin_set`, `merchant_registered`, `merchant_deactivated`,
  `payment_processed`, `refund_initiated`, `refund_approved`,
  `refund_rejected`, `refund_executed`, `multisig_initiated`,
  `multisig_signed`, `multisig_executed`.
- **Error codes** — typed `PaymentError` enum covering auth, merchant,
  payment, refund, multisig, and general error categories.
- **CI pipeline** — test matrix (stable + MSRV 1.79.0), clippy, rustfmt,
  WASM build, release artifact upload, security audit (`cargo-audit`),
  dependency policy (`cargo-deny`).

[Unreleased]: https://github.com/jhayniffy/pulsar-contracts/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/jhayniffy/pulsar-contracts/releases/tag/v0.1.0
