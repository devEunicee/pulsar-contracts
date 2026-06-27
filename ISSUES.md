# Pulsar Contracts — Development Issues

---

## 🔐 Smart Contracts

---

### SC-001 · Signature payload covers only order_id, not full order [COMPLETED]

**Description:** `process_payment_with_signature` builds the ed25519 payload from `order.order_id` bytes only. An attacker can reuse a valid signature from one order on a different order that shares the same `order_id` prefix, or craft a collision. The payload must commit to the entire order struct.

**Acceptance Criteria:**
- [x] Payload is the canonical serialisation of the full `PaymentOrder` (all fields)
- [x] Existing tests updated to reflect the new payload
- [x] A test proves that a signature over a different amount is rejected

**Priority:** Critical
**Effort:** Small
**Labels:** `smart-contract`, `security`, `bug`

---

### SC-002 · `cleanup_expired_payments` is a no-op

**Description:** The function always returns `0` with a comment "we skip that for brevity". There is no global payment index, so expired records are never cleaned up, causing unbounded storage growth.

**Acceptance Criteria:**
- A `GlobalPaymentIndex` key stores all payment IDs in insertion order
- `cleanup_expired_payments` iterates the index and removes records older than the cleanup period
- Returns the actual count of removed records
- Unit test verifies records are removed after the period elapses

**Priority:** High
**Effort:** Medium
**Labels:** `smart-contract`, `bug`, `storage`

---

### SC-003 · `get_global_payment_stats` ignores date range parameters

**Description:** `date_start` and `date_end` are accepted but silently discarded (`let _ = (date_start, date_end)`). Callers receive all-time totals regardless of the requested window, making the API misleading.

**Acceptance Criteria:**
- Time-bucketed counters (e.g. per-day or per-epoch) maintained in storage, OR
- A scan-based fallback that filters the global index by timestamp
- `date_start`/`date_end` of `None` still returns all-time totals
- Tests cover filtered and unfiltered calls

**Priority:** High
**Effort:** Large
**Labels:** `smart-contract`, `bug`, `feature`

---

### SC-004 · `execute_refund` has no caller authentication

**Description:** `execute_refund` calls `record.merchant_address.require_auth()` after loading the record, but the function signature takes no `caller` parameter. Any account can invoke the function; the SDK will require the merchant's auth but there is no explicit access-control check before the token transfer begins.

**Acceptance Criteria:**
- Add an explicit `caller: Address` parameter
- Verify `caller == record.merchant_address` before `require_auth`
- Test that a non-merchant caller is rejected

**Priority:** High
**Effort:** Small
**Labels:** `smart-contract`, `security`, `bug`

---

### SC-005 · Multisig executor pays from their own balance, not a shared escrow

**Description:** `execute_multisig_payment` calls `token_client.transfer(&executor, ...)`, meaning the executor — not the initiator or a locked escrow — funds the payment. This breaks the multi-party intent and allows any signer to be drained.

**Acceptance Criteria:**
- Funds are locked into the contract (or a dedicated escrow account) at `initiate_multisig_payment`
- `execute_multisig_payment` releases from escrow, not from the executor's wallet
- Tests verify the initiator's balance is debited at initiation, not at execution

**Priority:** Critical
**Effort:** Large
**Labels:** `smart-contract`, `security`, `bug`

---

### SC-006 · No expiry on multisig payments

**Description:** `MultisigPayment` has a `created_at` field but no expiry enforcement. A multisig payment can sit unsigned indefinitely, locking funds or allowing execution long after the business intent has lapsed.

**Acceptance Criteria:**
- `MultisigPayment` gains an `expires_at: u64` field set at initiation
- `sign_multisig_payment` and `execute_multisig_payment` reject expired payments
- Default expiry configurable via admin; minimum 1 hour
- Tests cover expiry rejection

**Priority:** High
**Effort:** Small
**Labels:** `smart-contract`, `enhancement`

---

### SC-007 · Pagination cursor logic is inverted — skips records before the cursor

**Description:** In `paginate_payments`, `skip` starts as `true` when a cursor is provided and records are skipped until the cursor ID is found. This means the cursor record itself is excluded and the page starts from the record *after* it — but the cursor is set to `records.get(cap - 1)`, i.e. the last record on the current page. On the next call the first record of the new page is skipped, causing data loss.

**Acceptance Criteria:**
- Cursor semantics clearly defined: cursor is the last seen ID; next page starts after it
- Implementation correctly skips up to and including the cursor record
- Test with 5 records, page size 2, verifies all records are returned across pages with no duplicates or gaps

**Priority:** High
**Effort:** Small
**Labels:** `smart-contract`, `bug`

---

### SC-008 · Insertion-sort on `soroban_sdk::Vec` is O(n²) and hits instruction limits

**Description:** `paginate_payments` performs an in-place insertion sort over all matching records before truncating to `cap`. For merchants with thousands of payments this will exceed Soroban's instruction budget and cause transaction failure.

**Acceptance Criteria:**
- Payment IDs stored in pre-sorted order per merchant (separate ascending/descending indexes), OR
- Sort deferred to client side with a documented limit, OR
- Merge-sort or radix-sort implementation with instruction-budget benchmarks
- Existing pagination tests still pass

**Priority:** High
**Effort:** Large
**Labels:** `smart-contract`, `performance`, `bug`

---

### SC-009 · `StorageError` (code 51) is defined but never returned

**Description:** `PaymentError::StorageError` exists in the enum but is unused. Dead error variants inflate the ABI and confuse integrators.

**Acceptance Criteria:**
- Either remove `StorageError` or wire it to actual storage failure paths
- If kept, at least one code path returns it
- Changelog entry added

**Priority:** Low
**Effort:** Small
**Labels:** `smart-contract`, `cleanup`

---

### SC-010 · `InsufficientBalance` (code 25) is defined but never returned

**Description:** `PaymentError::InsufficientBalance` is declared but no code path returns it. Token transfer failures surface as SDK panics rather than a typed error.

**Acceptance Criteria:**
- Wrap token `transfer` calls to catch insufficient-balance panics and return `InsufficientBalance`
- Or remove the variant if the SDK guarantees a panic (document the decision)
- Test covers the insufficient-balance path

**Priority:** Medium
**Effort:** Small
**Labels:** `smart-contract`, `bug`, `cleanup`

---

### SC-011 · No re-entrancy guard on token transfers

**Description:** Token transfers in `process_payment_with_signature` and `execute_refund` are made to external token contracts. A malicious token contract could re-enter the payment contract before state is finalised.

**Acceptance Criteria:**
- State (record saved, stats incremented) committed before any external `transfer` call, OR
- Document why Soroban's execution model prevents re-entrancy and add a code comment
- Security review sign-off

**Priority:** High
**Effort:** Small
**Labels:** `smart-contract`, `security`

---

### SC-012 · `update_payment_status` is a public function with no clear caller restriction

**Description:** `update_payment_status` allows the merchant or admin to arbitrarily set `refunded_amount` without going through the refund workflow. This bypasses the approval flow and can be used to manipulate payment records.

**Acceptance Criteria:**
- Either remove the function (refund state is managed exclusively by the refund workflow), OR
- Restrict it to internal use only and remove from the public ABI
- Tests verify the refund workflow is the sole path to status changes

**Priority:** High
**Effort:** Small
**Labels:** `smart-contract`, `security`, `bug`

---

### SC-013 · Merchant payment ID list grows unboundedly in persistent storage [COMPLETED]

**Description:** `push_merchant_payment_id` appends to a `Vec` stored under a single persistent key. Loading and re-saving this vector on every payment is O(n) in storage reads/writes and will eventually exceed ledger entry size limits.

**Acceptance Criteria:**
- [x] Replace with a linked-list or chunked index structure, OR
- [x] Cap the in-contract index at a configurable maximum and document the trade-off
- [x] Benchmark storage cost at 1 000, 10 000, and 100 000 entries

**Priority:** High
**Effort:** Large
**Labels:** `smart-contract`, `performance`, `storage`

---

### SC-014 · `validate_order_id` only checks for empty string

**Description:** Order IDs are not validated for maximum length or character set. An attacker can submit an arbitrarily long string as an order ID, inflating storage costs.

**Acceptance Criteria:**
- Maximum length enforced (e.g. 64 bytes)
- Allowed character set documented and enforced (alphanumeric + `-_`)
- `InvalidInput` returned on violation
- Tests cover boundary values

**Priority:** Medium
**Effort:** Small
**Labels:** `smart-contract`, `security`, `validation`

---

### SC-015 · No maximum length validation on merchant string fields

**Description:** `name`, `description`, `contact_info` in `register_merchant` have no length limits. Unbounded strings inflate storage and can be used for griefing.

**Acceptance Criteria:**
- `name` ≤ 64 bytes, `description` ≤ 256 bytes, `contact_info` ≤ 128 bytes
- `InvalidInput` returned on violation
- Tests cover boundary values

**Priority:** Medium
**Effort:** Small
**Labels:** `smart-contract`, `validation`, `security`

---

### SC-016 · `set_admin` allows the zero/burn address as admin

**Description:** There is no check that the provided `admin` address is a valid, non-zero account. Setting the burn address as admin would permanently lock all admin-only functions.

**Acceptance Criteria:**
- Validate that `admin` is not the zero address (if the SDK exposes such a check)
- Document the limitation if the SDK does not support zero-address detection
- Test attempts to set an invalid admin

**Priority:** Medium
**Effort:** Small
**Labels:** `smart-contract`, `validation`, `security`

---

### SC-017 · No admin transfer / rotation mechanism

**Description:** Once set, the admin address cannot be changed. If the admin key is compromised or lost, there is no recovery path.

**Acceptance Criteria:**
- Add `transfer_admin(env, current_admin, new_admin)` function
- Requires auth from both current and new admin (two-step handoff)
- Emits `admin_transferred` event
- Test covers successful transfer and rejection of unauthorised transfer

**Priority:** High
**Effort:** Small
**Labels:** `smart-contract`, `feature`, `security`

---

### SC-018 · Refund race condition: two refunds can exceed payment amount

**Description:** `initiate_refund` checks `record.refunded_amount + amount <= record.amount` at initiation time, but `refunded_amount` is only updated at `execute_refund`. Two concurrent pending refunds can each pass the check independently and together exceed the original amount.

**Acceptance Criteria:**
- Track `pending_refund_amount` on `PaymentRecord` incremented at initiation and decremented on rejection
- Check `refunded_amount + pending_refund_amount + new_amount <= record.amount` at initiation
- Tests simulate two simultaneous refund initiations

**Priority:** Critical
**Effort:** Medium
**Labels:** `smart-contract`, `security`, `bug`

---

### SC-019 · No event emitted on merchant deactivation

**Description:** `deactivate_merchant` modifies state but emits no event, making it impossible for off-chain indexers to track merchant lifecycle changes.

**Acceptance Criteria:**
- Emit `merchant_deactivated` event with merchant address and caller
- Event documented in README Events table
- Test asserts event is emitted

**Priority:** Medium
**Effort:** Small
**Labels:** `smart-contract`, `enhancement`

---

### SC-020 · No event emitted on `update_payment_status`

**Description:** `update_payment_status` silently mutates payment records with no on-chain trace.

**Acceptance Criteria:**
- Emit `payment_status_updated` event with order_id, new status, and caller
- Test asserts event is emitted

**Priority:** Medium
**Effort:** Small
**Labels:** `smart-contract`, `enhancement`

---

### SC-021 · `require_merchant` helper is defined but never called

**Description:** `helper::require_merchant` exists but is unused. Dead code increases maintenance burden and may indicate missing access-control checks.

**Acceptance Criteria:**
- Audit all merchant-restricted functions and apply `require_merchant` where appropriate, OR
- Remove the helper if genuinely unneeded
- No `#[allow(dead_code)]` suppressions added

**Priority:** Low
**Effort:** Small
**Labels:** `smart-contract`, `cleanup`

---

### SC-022 · `PaymentOrder.payer` field is unused in payment processing

**Description:** `PaymentOrder` contains a `payer` field, but `process_payment_with_signature` uses the `payer` function parameter instead. The field is never validated against the parameter, allowing a mismatch.

**Acceptance Criteria:**
- Either remove `payer` from `PaymentOrder` (breaking change — document migration), OR
- Assert `order.payer == payer` parameter and return `InvalidInput` on mismatch
- Tests cover the mismatch case

**Priority:** Medium
**Effort:** Small
**Labels:** `smart-contract`, `bug`, `validation`

---

### SC-023 · Global stats `total_volume` can overflow `i128`

**Description:** `increment_payment_stats` adds `amount: i128` to `total_volume: i128` with no overflow check. In a high-volume deployment this will silently wrap.

**Acceptance Criteria:**
- Use `checked_add` and return/log an error on overflow, OR
- Use `u128` for volume fields (requires type migration)
- Test with values near `i128::MAX`

**Priority:** Medium
**Effort:** Small
**Labels:** `smart-contract`, `bug`

---

### SC-024 · No mechanism to reactivate a deactivated merchant

**Description:** Once deactivated, a merchant cannot be reactivated. There is no `reactivate_merchant` function, forcing a new registration with a different address.

**Acceptance Criteria:**
- Add `reactivate_merchant(env, caller, merchant_address)` callable by admin or the merchant
- Emits `merchant_reactivated` event
- Test covers reactivation and subsequent payment processing

**Priority:** Medium
**Effort:** Small
**Labels:** `smart-contract`, `feature`

---

### SC-025 · Multisig payment does not validate merchant is active

**Description:** `initiate_multisig_payment` validates amount and uniqueness but does not check that the target merchant is registered and active, unlike `process_payment_with_signature`.

**Acceptance Criteria:**
- Add merchant active check in `initiate_multisig_payment`
- Return `MerchantInactive` or `MerchantNotFound` as appropriate
- Test covers inactive merchant rejection

**Priority:** Medium
**Effort:** Small
**Labels:** `smart-contract`, `bug`, `validation`

---

### SC-026 · No upper bound on `required_signers` list in multisig

**Description:** `required_signers` can be arbitrarily large. Iterating over it in `sign_multisig_payment` (`.contains`) is O(n) and a very large list will exceed instruction limits.

**Acceptance Criteria:**
- Enforce a maximum signer count (e.g. 10)
- Return `InvalidInput` if exceeded
- Test boundary at max and max+1

**Priority:** Medium
**Effort:** Small
**Labels:** `smart-contract`, `validation`, `performance`

---

### SC-027 · `archive_payment_record` does not remove the ID from merchant/payer indexes

**Description:** Archiving a payment removes the `Payment(order_id)` storage entry but leaves the ID in `MerchantPayments` and `PayerPayments` vectors. Subsequent history queries will encounter missing records and silently skip them, corrupting pagination counts.

**Acceptance Criteria:**
- `archive_payment_record` also removes the ID from both index vectors
- Or marks the record as archived rather than deleting it
- Test verifies history query after archival returns correct count

**Priority:** High
**Effort:** Medium
**Labels:** `smart-contract`, `bug`

---

### SC-028 · No contract version / upgrade path

**Description:** There is no version field in contract storage and no upgrade mechanism. Future bug fixes require deploying a new contract address, breaking all existing integrations.

**Acceptance Criteria:**
- Store a `ContractVersion` key in instance storage set at `set_admin`
- Add `upgrade(env, admin, new_wasm_hash)` using Soroban's `update_current_contract_wasm`
- Document upgrade procedure in README
- Test covers version read

**Priority:** High
**Effort:** Medium
**Labels:** `smart-contract`, `feature`, `devops`

---

### SC-029 · TTL / ledger entry expiry not managed for persistent storage

**Description:** Soroban persistent storage entries expire after a ledger TTL. The contract never calls `extend_ttl` on payment, merchant, or refund entries, meaning long-lived records will be evicted and become permanently inaccessible.

**Acceptance Criteria:**
- Call `env.storage().persistent().extend_ttl(key, threshold, extend_to)` on every read/write of persistent entries
- Define sensible TTL constants (e.g. 1 year in ledgers)
- Document TTL strategy in README

**Priority:** Critical
**Effort:** Medium
**Labels:** `smart-contract`, `bug`, `storage`

---

### SC-030 · Instance storage TTL not extended

**Description:** Admin, GlobalStats, and CleanupPeriod are stored in instance storage, which also has a TTL. If the contract goes dormant, instance storage expires and the contract becomes unusable.

**Acceptance Criteria:**
- Extend instance storage TTL on every contract invocation (or at minimum on admin operations)
- Add a public `bump_instance_ttl(env)` function callable by anyone
- Test simulates TTL expiry scenario

**Priority:** High
**Effort:** Small
**Labels:** `smart-contract`, `bug`, `storage`


---

## 🧪 Testing

---

### T-001 · Signature verification is never tested with a real ed25519 key pair

**Description:** All tests use `mock_all_auths()` and dummy zero-byte keys/signatures. The actual `ed25519_verify` path is never exercised, so a regression in signature logic would not be caught.

**Acceptance Criteria:**
- At least one test generates a real ed25519 key pair, signs the correct payload, and verifies the payment succeeds
- A test with a tampered signature verifies `InvalidSignature` is returned
- `mock_all_auths` removed from signature-specific tests

**Priority:** Critical
**Effort:** Medium
**Labels:** `testing`, `security`

---

### T-002 · No integration test for the full refund lifecycle with token balance assertions

**Description:** `test_successful_refund_flow` checks refund status but never asserts that the payer's token balance increased after `execute_refund`.

**Acceptance Criteria:**
- Assert payer balance before and after `execute_refund`
- Assert merchant balance decreases by the refund amount
- Test covers partial and full refund scenarios

**Priority:** High
**Effort:** Small
**Labels:** `testing`, `bug`

---

### T-003 · No test for `get_payer_payment_history`

**Description:** `get_payer_payment_history` has no dedicated test. Filtering, sorting, and pagination for the payer view are untested.

**Acceptance Criteria:**
- Tests cover: no payments, single payment, multiple payments
- Tests cover all filter fields (date range, amount range, token, status)
- Tests cover both sort fields and both sort orders
- Tests cover multi-page pagination

**Priority:** High
**Effort:** Medium
**Labels:** `testing`

---

### T-004 · No test for `get_global_payment_stats` with date filters

**Description:** The only stats test checks all-time totals. Date-filtered stats are untested (and currently broken — see SC-003).

**Acceptance Criteria:**
- Tests added once SC-003 is resolved
- Cover: no payments in range, some payments in range, all payments in range

**Priority:** Medium
**Effort:** Small
**Dependencies:** SC-003
**Labels:** `testing`

---

### T-005 · No negative test for `deactivate_merchant` by unauthorised caller

**Description:** There is no test verifying that a random address cannot deactivate another merchant.

**Acceptance Criteria:**
- Test: random caller attempts `deactivate_merchant` → `Unauthorized`
- Test: merchant deactivates themselves → succeeds
- Test: admin deactivates merchant → succeeds

**Priority:** Medium
**Effort:** Small
**Labels:** `testing`, `security`

---

### T-006 · No test for `archive_payment_record`

**Description:** `archive_payment_record` has no test coverage.

**Acceptance Criteria:**
- Test: admin archives existing payment → payment no longer retrievable
- Test: non-admin attempts archive → `Unauthorized`
- Test: archive non-existent payment → `PaymentNotFound`

**Priority:** Medium
**Effort:** Small
**Labels:** `testing`

---

### T-007 · No test for `set_payment_cleanup_period` with zero value

**Description:** The zero-period guard exists but is untested.

**Acceptance Criteria:**
- Test: `set_payment_cleanup_period(0)` → `InvalidInput`
- Test: valid period is persisted and readable

**Priority:** Low
**Effort:** Small
**Labels:** `testing`

---

### T-008 · No test for multisig with duplicate signer in `required_signers` [COMPLETED]

**Description:** If the same address appears twice in `required_signers`, they can sign twice and satisfy the threshold alone.

**Acceptance Criteria:**
- [x] Test: duplicate signer in `required_signers` → `InvalidInput` (after fix)
- [x] Deduplication logic added to `initiate_multisig_payment`

**Priority:** High
**Effort:** Small
**Labels:** `testing`, `security`, `bug`

---

### T-009 · No test for `approve_refund` by non-merchant, non-admin caller [COMPLETED]

**Description:** Unauthorised approval path is untested.

**Acceptance Criteria:**
- [x] Test: random caller attempts `approve_refund` → `Unauthorized`

**Priority:** Medium
**Effort:** Small
**Labels:** `testing`, `security`

---

### T-010 · No test for payment with zero amount

**Description:** `validate_amount` rejects zero, but there is no test confirming this for `process_payment_with_signature`.

**Acceptance Criteria:**
- Test: `amount = 0` → `InvalidAmount`
- Test: `amount = -1` → `InvalidAmount`

**Priority:** Medium
**Effort:** Small
**Labels:** `testing`, `validation`

---

### T-011 · No fuzz / property-based tests

**Description:** The contract has no fuzz or property-based tests. Edge cases in pagination, arithmetic, and string handling are only covered by hand-written cases.

**Acceptance Criteria:**
- Add `cargo-fuzz` or `proptest` targets for at minimum: `validate_amount`, `paginate_payments`, `matches_filter`
- Fuzz targets added to CI as a separate job
- Document how to run fuzz tests locally

**Priority:** Medium
**Effort:** Large
**Labels:** `testing`, `security`

---

### T-012 · No test for `cleanup_expired_payments` once implemented

**Description:** Once SC-002 is resolved, the cleanup function needs comprehensive tests.

**Acceptance Criteria:**
- Test: no expired payments → returns 0
- Test: some expired, some not → only expired removed
- Test: all expired → all removed, index cleared
- Test: non-admin caller → `Unauthorized`

**Priority:** High
**Effort:** Small
**Dependencies:** SC-002
**Labels:** `testing`

---

### T-013 · No test for concurrent refund race condition

**Description:** Once SC-018 is resolved, the race condition fix needs a regression test.

**Acceptance Criteria:**
- Test: two refunds initiated in same ledger, combined amount ≤ payment → both succeed
- Test: two refunds initiated, combined amount > payment → second rejected

**Priority:** High
**Effort:** Small
**Dependencies:** SC-018
**Labels:** `testing`, `security`

---

### T-014 · Test helper `setup_paid_order` is not reused consistently

**Description:** Several tests duplicate the setup logic instead of using `setup_paid_order`, making tests harder to maintain.

**Acceptance Criteria:**
- All tests that need a paid order use `setup_paid_order`
- No duplicated setup blocks longer than 5 lines

**Priority:** Low
**Effort:** Small
**Labels:** `testing`, `cleanup`

---

### T-015 · No test coverage report in CI

**Description:** CI runs tests but does not measure or enforce coverage thresholds.

**Acceptance Criteria:**
- Add `cargo-llvm-cov` step to CI
- Coverage report uploaded as artifact
- Minimum 80% line coverage enforced (warning, not hard fail initially)

**Priority:** Medium
**Effort:** Small
**Labels:** `testing`, `devops`

---

## 🔒 Security

---

### SEC-001 · No formal security audit before mainnet deployment

**Description:** The contract handles real token transfers. It must be audited by an independent security firm before any mainnet deployment.

**Acceptance Criteria:**
- Engage a recognised Soroban/Stellar smart contract auditor
- All Critical and High findings resolved before deployment
- Audit report published in the repository

**Priority:** Critical
**Effort:** Large
**Labels:** `security`, `compliance`

---

### SEC-002 · ed25519 public key not tied to the registered merchant

**Description:** `process_payment_with_signature` accepts any `merchant_public_key` bytes. There is no on-chain binding between the merchant's registered `Address` and their ed25519 public key, allowing an attacker to substitute a key they control.

**Acceptance Criteria:**
- `Merchant` struct gains an optional `signing_public_key: Option<Bytes>` field
- `register_merchant` or a new `set_merchant_signing_key` function stores the key
- `process_payment_with_signature` verifies against the stored key, not the caller-supplied one
- Tests cover key mismatch rejection

**Priority:** Critical
**Effort:** Medium
**Labels:** `security`, `smart-contract`

---

### SEC-003 · No rate limiting on merchant registration

**Description:** Anyone can register unlimited merchants, spamming storage and inflating indexes at minimal cost.

**Acceptance Criteria:**
- Require a minimum XLM deposit (base reserve) to register, OR
- Admin-whitelist mode: registration requires admin pre-approval
- Document the chosen anti-spam mechanism

**Priority:** High
**Effort:** Medium
**Labels:** `security`, `smart-contract`

---

### SEC-004 · Refund `reason` field has no length limit

**Description:** The `reason` string in `RefundRecord` is unbounded, enabling storage bloat attacks.

**Acceptance Criteria:**
- Maximum 256 bytes enforced
- `InvalidInput` returned on violation
- Test covers boundary

**Priority:** Medium
**Effort:** Small
**Labels:** `security`, `validation`

---

### SEC-005 · No replay protection for signed payment orders

**Description:** A valid `(order, signature)` pair can be replayed if the `order_id` is not already in storage (e.g. after `archive_payment_record`). The duplicate-ID check is the only replay guard.

**Acceptance Criteria:**
- Document that `order_id` must never be reused after archival
- Prevent `archive_payment_record` from removing IDs that could be replayed (add a tombstone), OR
- Include a nonce or block-height in the signed payload

**Priority:** High
**Effort:** Medium
**Labels:** `security`, `smart-contract`

---

### SEC-006 · Admin address stored in instance storage is a single point of failure

**Description:** A single admin key controls merchant deactivation, refund approval, payment archival, and stats. Compromise of this key is catastrophic.

**Acceptance Criteria:**
- Implement a multi-sig admin model (N-of-M), OR
- Add a time-locked admin action queue for destructive operations
- Document the governance model

**Priority:** High
**Effort:** Large
**Labels:** `security`, `feature`

---

### SEC-007 · No input sanitisation on `contact_info` field

**Description:** `contact_info` is stored and potentially displayed by frontends without sanitisation. Stored XSS or injection attacks are possible if the field is rendered in a web UI.

**Acceptance Criteria:**
- Enforce allowed character set (printable ASCII, max 128 bytes)
- Frontend must also sanitise on render (separate frontend issue)
- Test covers rejection of control characters

**Priority:** Medium
**Effort:** Small
**Labels:** `security`, `validation`

---

### SEC-008 · Dependency audit not pinned to a specific advisory database version

**Description:** `cargo audit` in CI uses the latest advisory database at run time. A new advisory could break CI unexpectedly without a code change.

**Acceptance Criteria:**
- Pin `cargo-audit` to a specific version in CI
- Add `--deny warnings` flag to fail on any advisory
- Schedule weekly advisory database refresh as a separate CI job

**Priority:** Medium
**Effort:** Small
**Labels:** `security`, `devops`

---

### SEC-009 · No check that token address is a valid Stellar asset contract

**Description:** `order.token` is used directly as a token contract address with no validation. A malicious token contract could implement `transfer` to perform arbitrary actions.

**Acceptance Criteria:**
- Maintain an admin-managed allowlist of approved token contracts, OR
- Document the trust model and warn integrators
- Test with a mock malicious token contract

**Priority:** High
**Effort:** Medium
**Labels:** `security`, `smart-contract`

---

### SEC-010 · No pause / emergency stop mechanism

**Description:** If a critical vulnerability is discovered post-deployment, there is no way to halt the contract while a fix is prepared.

**Acceptance Criteria:**
- Add `pause(env, admin)` / `unpause(env, admin)` functions
- All state-mutating functions check a `Paused` flag and return `Unauthorized` when paused
- Read-only functions remain accessible when paused
- Emits `contract_paused` / `contract_unpaused` events

**Priority:** High
**Effort:** Medium
**Labels:** `security`, `feature`


---

## ⚙️ DevOps

---

### DO-001 · No `Cargo.lock` committed to the repository

**Description:** The workspace `.gitignore` likely excludes `Cargo.lock`. For a smart contract, reproducible builds are critical — the exact dependency tree must be locked and auditable.

**Acceptance Criteria:**
- `Cargo.lock` committed and tracked in git
- `.gitignore` updated to not exclude it
- CI uses `--locked` flag on all `cargo` commands

**Priority:** High
**Effort:** Small
**Labels:** `devops`, `security`

---

### DO-002 · CI has no deployment job for testnet

**Description:** CI builds the WASM artifact but has no automated deployment to testnet on merge to `main`. Developers must deploy manually.

**Acceptance Criteria:**
- Add a `deploy-testnet` job triggered on push to `main`
- Uses `stellar contract deploy` with a CI-managed testnet key stored as a GitHub secret
- Outputs and stores the deployed contract ID as a CI artifact
- Job is skipped on PRs

**Priority:** Medium
**Effort:** Medium
**Labels:** `devops`, `ci-cd`

---

### DO-003 · No WASM size check in CI

**Description:** Soroban has a WASM size limit (~128 KB). There is no CI gate to catch size regressions before they cause deployment failures.

**Acceptance Criteria:**
- Add a CI step that checks WASM binary size after build
- Fail if size exceeds 100 KB (warn at 80 KB)
- Report size in CI summary

**Priority:** Medium
**Effort:** Small
**Labels:** `devops`, `ci-cd`

---

### DO-004 · No branch protection rules documented

**Description:** The README and CONTRIBUTING.md do not document required branch protection rules for `main` (required reviews, status checks, no force-push).

**Acceptance Criteria:**
- Document required branch protection settings in CONTRIBUTING.md
- Enforce: at least 1 approving review, all CI checks must pass, no direct push to `main`

**Priority:** Medium
**Effort:** Small
**Labels:** `devops`, `process`

---

### DO-005 · `cargo audit` installs from crates.io on every CI run

**Description:** `cargo install cargo-audit --locked` downloads and compiles the tool on every run, adding ~2 minutes to CI. There is no caching.

**Acceptance Criteria:**
- Cache `cargo-audit` binary between runs using `actions/cache`
- Or use a pre-built Docker image that includes `cargo-audit`
- CI audit job runtime reduced to under 30 seconds

**Priority:** Low
**Effort:** Small
**Labels:** `devops`, `ci-cd`

---

### DO-006 · No semantic versioning or release tagging process

**Description:** The contract is at `0.1.0` with no documented process for bumping versions, tagging releases, or publishing changelogs.

**Acceptance Criteria:**
- Define a versioning policy (semver: major = breaking ABI change)
- Add a `CHANGELOG.md` with an initial entry
- CI creates a GitHub Release and tags on version bump in `Cargo.toml`

**Priority:** Medium
**Effort:** Small
**Labels:** `devops`, `process`

---

### DO-007 · No `deny.toml` for supply-chain policy enforcement

**Description:** There is no `cargo-deny` configuration to enforce license compatibility, ban known-bad crates, or restrict duplicate dependencies.

**Acceptance Criteria:**
- Add `deny.toml` with license allowlist (MIT, Apache-2.0)
- Ban crates with known vulnerabilities
- Add `cargo deny check` step to CI

**Priority:** Medium
**Effort:** Small
**Labels:** `devops`, `security`

---

### DO-008 · CI does not test on multiple Rust toolchain versions

**Description:** CI only tests on `stable`. Soroban SDK may require a specific minimum Rust version; regressions on older toolchains are not caught.

**Acceptance Criteria:**
- Document minimum supported Rust version (MSRV) in `Cargo.toml` (`rust-version` field)
- CI matrix includes `stable` and MSRV
- `rust-version` field kept in sync with CI matrix

**Priority:** Low
**Effort:** Small
**Labels:** `devops`, `ci-cd`

---

### DO-009 · No local development environment setup script

**Description:** New contributors must manually install Rust, add the WASM target, and install Stellar CLI. There is no script to automate this.

**Acceptance Criteria:**
- Add `scripts/setup.sh` that installs all prerequisites
- Script is idempotent (safe to run multiple times)
- README updated to reference the script
- Script tested on Ubuntu and macOS

**Priority:** Medium
**Effort:** Small
**Labels:** `devops`, `dx`

---

### DO-010 · No Docker-based local development environment

**Description:** Local network setup requires Docker Desktop but there is no `docker-compose.yml` or `Dockerfile` to standardise the environment.

**Acceptance Criteria:**
- Add `docker-compose.yml` that starts a local Stellar network
- Add a `Dockerfile.dev` with all build tools pre-installed
- README updated with Docker-based quickstart

**Priority:** Medium
**Effort:** Medium
**Labels:** `devops`, `dx`

---

### DO-011 · WASM artifact retention in CI is 30 days — no permanent release assets

**Description:** Built WASM artifacts are uploaded with `retention-days: 30`. After 30 days, the artifact for a given commit is gone, making it impossible to reproduce a historical deployment.

**Acceptance Criteria:**
- On tagged releases, upload WASM to GitHub Releases (permanent)
- CI artifact retention for non-release builds can remain 30 days
- Release WASM includes a SHA-256 checksum file

**Priority:** Medium
**Effort:** Small
**Labels:** `devops`, `ci-cd`

---

### DO-012 · No monitoring or alerting for deployed contract

**Description:** There is no off-chain monitoring for unusual activity (large payments, high refund rates, failed transactions) on the deployed contract.

**Acceptance Criteria:**
- Define key metrics: payment volume, refund rate, error rate
- Set up a Stellar Horizon event stream listener (or equivalent)
- Alert on anomalies (e.g. refund rate > 20% in 1 hour)
- Document monitoring setup

**Priority:** High
**Effort:** Large
**Labels:** `devops`, `monitoring`

---

### DO-013 · No `pre-commit` hook for formatting and linting

**Description:** Developers can commit unformatted or clippy-failing code. CI catches it, but the feedback loop is slow.

**Acceptance Criteria:**
- Add a `pre-commit` hook (or `.pre-commit-config.yaml`) that runs `cargo fmt --check` and `cargo clippy`
- Document hook installation in CONTRIBUTING.md

**Priority:** Low
**Effort:** Small
**Labels:** `devops`, `dx`

---

### DO-014 · No environment-specific configuration management

**Description:** There is no structured way to manage contract IDs, network endpoints, or admin keys across local, testnet, and mainnet environments.

**Acceptance Criteria:**
- Add a `config/` directory with per-environment TOML files
- Document which values are secrets (never committed) vs. public config
- Scripts reference config files rather than hardcoded values

**Priority:** Medium
**Effort:** Small
**Labels:** `devops`, `configuration`

---

### DO-015 · CI `cargo test` does not use `--locked`

**Description:** Without `--locked`, CI may resolve different dependency versions than what is in `Cargo.lock`, undermining reproducibility.

**Acceptance Criteria:**
- All `cargo` commands in CI use `--locked`
- `Cargo.lock` committed (see DO-001)

**Priority:** High
**Effort:** Small
**Dependencies:** DO-001
**Labels:** `devops`, `ci-cd`

---

## 📚 Documentation

---

### DOC-001 · No CHANGELOG.md

**Description:** There is no changelog. Contributors and integrators cannot track what changed between versions.

**Acceptance Criteria:**
- Create `CHANGELOG.md` following Keep a Changelog format
- Initial entry documents the 0.1.0 feature set
- PR template reminds contributors to update the changelog

**Priority:** Medium
**Effort:** Small
**Labels:** `documentation`

---

### DOC-002 · README deployment section uses placeholder values without explanation

**Description:** `<YOUR_SECRET_KEY>`, `<ADMIN_SECRET_KEY>`, and `$CONTRACT_ID` are used without explaining how to obtain or generate them, blocking new users.

**Acceptance Criteria:**
- Add a "Prerequisites — Keys" subsection explaining how to generate a Stellar keypair
- Explain how to fund a testnet account via Friendbot
- Replace `<YOUR_SECRET_KEY>` with a note pointing to the keys section

**Priority:** Medium
**Effort:** Small
**Labels:** `documentation`, `onboarding`

---

### DOC-003 · No architecture decision records (ADRs)

**Description:** Key design decisions (why ed25519 over Stellar native auth, why insertion-sort, why no global index) are not documented, making it hard for new contributors to understand trade-offs.

**Acceptance Criteria:**
- Create `docs/adr/` directory
- Write ADRs for: signature scheme choice, storage layout, pagination design
- ADR template added for future decisions

**Priority:** Low
**Effort:** Medium
**Labels:** `documentation`

---

### DOC-004 · `get_global_payment_stats` date filter behaviour not documented as a known limitation

**Description:** The README documents the function but does not mention that date filtering is currently a no-op (see SC-003). Integrators will be misled.

**Acceptance Criteria:**
- Add a "Known Limitations" section to README
- Document the date filter no-op until SC-003 is resolved
- Remove the limitation note once SC-003 is fixed

**Priority:** High
**Effort:** Small
**Labels:** `documentation`, `bug`

---

### DOC-005 · No inline rustdoc on public functions

**Description:** Only `set_admin` and `process_payment_with_signature` have doc comments. All other public functions lack documentation, making `cargo doc` output sparse.

**Acceptance Criteria:**
- All public functions in `lib.rs` have `///` doc comments
- Doc comments include: purpose, parameters, return value, errors
- `cargo doc --no-deps` produces no warnings

**Priority:** Medium
**Effort:** Medium
**Labels:** `documentation`

---

### DOC-006 · CONTRIBUTING.md does not document the issue triage process

**Description:** CONTRIBUTING.md covers code contributions but not how issues are triaged, labelled, or prioritised.

**Acceptance Criteria:**
- Add "Issue Triage" section to CONTRIBUTING.md
- Document label taxonomy (bug, enhancement, security, etc.)
- Document SLA for first response on issues

**Priority:** Low
**Effort:** Small
**Labels:** `documentation`, `process`

---

### DOC-007 · No SDK / ABI reference for off-chain integrators

**Description:** There is no machine-readable ABI or SDK documentation for developers building off-chain clients (e.g. a React frontend or a Node.js backend).

**Acceptance Criteria:**
- Generate and publish the contract ABI JSON (`stellar contract inspect`)
- Add a `docs/api-reference.md` with all function signatures, parameter types, and return types
- Document event schemas with field names and types

**Priority:** High
**Effort:** Medium
**Labels:** `documentation`, `integration`

---

### DOC-008 · README does not document storage costs / ledger entry fees

**Description:** Integrators need to understand the storage cost implications of registering merchants, processing payments, and maintaining history.

**Acceptance Criteria:**
- Add a "Storage Costs" section to README
- Document which operations create new ledger entries
- Provide approximate XLM cost estimates per operation

**Priority:** Medium
**Effort:** Small
**Labels:** `documentation`

---

### DOC-009 · No troubleshooting guide for common test failures

**Description:** The README has a brief troubleshooting section but does not cover common test failures (e.g. mock auth issues, token minting errors).

**Acceptance Criteria:**
- Expand troubleshooting section with at least 5 common test failure scenarios
- Each entry includes: symptom, cause, fix

**Priority:** Low
**Effort:** Small
**Labels:** `documentation`, `dx`

---

### DOC-010 · Security vulnerability reporting process is vague

**Description:** CONTRIBUTING.md says "email the maintainers directly" but provides no email address or GitHub security advisory link.

**Acceptance Criteria:**
- Add a `SECURITY.md` file with a clear disclosure policy
- Include the GitHub private security advisory URL
- Define response SLA (e.g. acknowledge within 48 hours)

**Priority:** High
**Effort:** Small
**Labels:** `documentation`, `security`


---

## 🔗 Backend / Off-Chain Integration

---

### BE-001 · No off-chain event indexer

**Description:** The contract emits events but there is no off-chain service to index them into a queryable database. Frontends and analytics tools have no way to query historical events without replaying the entire ledger.

**Acceptance Criteria:**
- Build or configure a Horizon event stream listener
- Index events into a PostgreSQL or equivalent database
- Expose a REST or GraphQL API for event queries
- Document the indexer setup and schema

**Priority:** High
**Effort:** Large
**Labels:** `backend`, `integration`

---

### BE-002 · No webhook / notification system for payment events

**Description:** Merchants have no way to receive real-time notifications when a payment is processed, a refund is initiated, or a multisig is signed.

**Acceptance Criteria:**
- Off-chain service listens to contract events
- Merchants can register a webhook URL via an off-chain API
- Webhook payload documented with example JSON
- Retry logic with exponential backoff on delivery failure

**Priority:** High
**Effort:** Large
**Labels:** `backend`, `feature`

---

### BE-003 · No off-chain order creation / signing service

**Description:** Merchants must manually construct `PaymentOrder` structs and sign them with ed25519. There is no SDK or service to simplify this.

**Acceptance Criteria:**
- Provide a TypeScript/JavaScript SDK with `createOrder(params)` and `signOrder(order, privateKey)` helpers
- SDK published to npm
- README links to SDK documentation

**Priority:** High
**Effort:** Large
**Labels:** `backend`, `dx`, `integration`

---

### BE-004 · No idempotency handling for duplicate API calls

**Description:** If a client submits the same payment twice (network retry), the second call returns `PaymentAlreadyExists`. Off-chain clients need guidance on handling this gracefully.

**Acceptance Criteria:**
- Document idempotency behaviour in API reference
- Off-chain SDK treats `PaymentAlreadyExists` as a success (idempotent)
- Test covers retry scenario

**Priority:** Medium
**Effort:** Small
**Labels:** `backend`, `integration`

---

### BE-005 · No rate limiting on contract invocations from a single account

**Description:** A single account can spam the contract with payment or refund initiations, inflating storage and degrading performance for other users.

**Acceptance Criteria:**
- Implement per-account invocation rate limiting in an off-chain gateway, OR
- Document that Stellar's fee market provides natural rate limiting
- Add a note in README about fee-based spam prevention

**Priority:** Medium
**Effort:** Medium
**Labels:** `backend`, `security`

---

### BE-006 · No structured error mapping for off-chain clients

**Description:** Contract errors are returned as numeric codes. Off-chain clients must hardcode the mapping from code to message.

**Acceptance Criteria:**
- Publish a machine-readable error code registry (JSON or TypeScript enum)
- SDK maps error codes to human-readable messages
- Error registry kept in sync with `error.rs` via a code generation script

**Priority:** Medium
**Effort:** Small
**Labels:** `backend`, `dx`

---

### BE-007 · No health check endpoint for the deployed contract

**Description:** There is no standard way for monitoring systems to verify the contract is live and responsive.

**Acceptance Criteria:**
- Add a `ping(env) -> u64` function that returns the current ledger timestamp
- Document as a health check endpoint
- Off-chain monitoring calls `ping` every 5 minutes

**Priority:** Low
**Effort:** Small
**Labels:** `backend`, `devops`

---

### BE-008 · No pagination support in off-chain API

**Description:** If an off-chain API wraps the contract, it must handle the cursor-based pagination correctly and expose it to API consumers.

**Acceptance Criteria:**
- Off-chain API exposes `cursor`, `limit` query parameters
- Response includes `next_cursor` and `total`
- API documentation includes pagination example

**Priority:** Medium
**Effort:** Small
**Labels:** `backend`, `integration`

---

## 🖥️ Frontend

---

### FE-001 · No merchant dashboard UI

**Description:** There is no web interface for merchants to view their payment history, initiate refunds, or manage their profile.

**Acceptance Criteria:**
- React (or equivalent) dashboard with: payment history table, refund management, merchant profile editor
- Connects to Stellar Freighter or Albedo wallet
- Responsive design (mobile and desktop)
- Accessibility: WCAG 2.1 AA compliant

**Priority:** High
**Effort:** Large
**Labels:** `frontend`, `feature`

---

### FE-002 · No payer payment history UI

**Description:** Payers have no way to view their payment history or track refund status without using the CLI.

**Acceptance Criteria:**
- Payer view: payment history with filter/sort controls, refund status tracker
- Wallet connection via Freighter
- Paginated table with cursor-based navigation

**Priority:** High
**Effort:** Large
**Labels:** `frontend`, `feature`

---

### FE-003 · No admin panel UI

**Description:** Admin operations (stats, cleanup, merchant management) require CLI access.

**Acceptance Criteria:**
- Admin panel: global stats dashboard, merchant list with deactivation controls, cleanup trigger
- Protected by wallet-based admin auth
- Audit log of admin actions displayed

**Priority:** Medium
**Effort:** Large
**Labels:** `frontend`, `feature`

---

### FE-004 · No payment checkout widget for merchant integration

**Description:** Merchants need an embeddable checkout widget that generates a signed payment order and submits it to the contract.

**Acceptance Criteria:**
- Embeddable `<PulsarCheckout />` React component
- Accepts `orderId`, `amount`, `token`, `merchantAddress` props
- Handles wallet connection, signing, and submission
- Emits `onSuccess`, `onError` callbacks

**Priority:** High
**Effort:** Large
**Labels:** `frontend`, `feature`

---

### FE-005 · No loading / error states in UI components

**Description:** (Applies to any future UI) Contract calls are async and can fail. UI must handle loading, success, and error states explicitly.

**Acceptance Criteria:**
- All contract-calling components show a loading spinner during submission
- Error messages map to human-readable strings (see BE-006)
- Success state shows transaction hash with Stellar Explorer link

**Priority:** High
**Effort:** Medium
**Labels:** `frontend`, `ux`

---

### FE-006 · No internationalisation (i18n) support

**Description:** The UI (once built) will be English-only. Pulsar targets a global merchant base.

**Acceptance Criteria:**
- i18n framework integrated (e.g. `react-i18next`)
- All user-facing strings externalised to translation files
- Initial support for English and Spanish
- RTL layout support considered

**Priority:** Low
**Effort:** Medium
**Labels:** `frontend`, `i18n`

---

### FE-007 · No dark mode support

**Description:** No theme switching capability planned.

**Acceptance Criteria:**
- CSS variables or Tailwind dark mode classes used throughout
- System preference respected via `prefers-color-scheme`
- Manual toggle available in UI

**Priority:** Low
**Effort:** Small
**Labels:** `frontend`, `ux`

---

## 🗂️ Miscellaneous / Product

---

### MISC-001 · No product roadmap or milestone planning

**Description:** There is no public roadmap communicating planned features, priorities, or release milestones to contributors and users.

**Acceptance Criteria:**
- Create a GitHub Project board with milestones: v0.1 (current), v0.2 (bug fixes), v1.0 (production-ready)
- Each milestone has defined acceptance criteria
- Roadmap linked from README

**Priority:** Medium
**Effort:** Small
**Labels:** `product`, `process`

---

### MISC-002 · No support for partial merchant profile updates

**Description:** Merchants cannot update their `name`, `description`, or `contact_info` after registration. They must deactivate and re-register.

**Acceptance Criteria:**
- Add `update_merchant(env, merchant_address, name, description, contact_info)` function
- Only the merchant themselves can update their profile
- Emits `merchant_updated` event
- Tests cover successful update and unauthorised attempt

**Priority:** Medium
**Effort:** Small
**Labels:** `smart-contract`, `feature`, `product`

---

### MISC-003 · No support for multi-token payment history filtering by multiple tokens

**Description:** `PaymentFilter.token` accepts only a single token address. Merchants accepting multiple tokens cannot filter for "all stablecoin payments" in one query.

**Acceptance Criteria:**
- Change `token: Option<Address>` to `tokens: Option<Vec<Address>>`
- Filter matches if payment token is in the list
- Tests cover single-token and multi-token filter

**Priority:** Low
**Effort:** Small
**Labels:** `smart-contract`, `feature`, `product`

---

### MISC-004 · No dispute resolution mechanism

**Description:** The refund flow has no dispute state. If a merchant rejects a refund and the payer disagrees, there is no on-chain escalation path.

**Acceptance Criteria:**
- Add `dispute_refund(env, caller, refund_id, reason)` callable by payer after rejection
- Admin can resolve disputes by approving or upholding the rejection
- Emits `refund_disputed` and `dispute_resolved` events
- Tests cover full dispute lifecycle

**Priority:** Medium
**Effort:** Large
**Labels:** `smart-contract`, `feature`, `product`

---

### MISC-005 · No support for recurring / subscription payments

**Description:** The contract only supports one-off payments. Subscription-based merchants cannot use Pulsar without building their own scheduling layer.

**Acceptance Criteria:**
- Design a `SubscriptionPlan` struct with interval, amount, and token
- `create_subscription`, `cancel_subscription`, `process_subscription_payment` functions
- Off-chain scheduler triggers `process_subscription_payment` at each interval
- Document the off-chain scheduling requirement

**Priority:** Low
**Effort:** Large
**Labels:** `smart-contract`, `feature`, `product`

---

### MISC-006 · No merchant category validation beyond enum variants

**Description:** `MerchantCategory` is an enum, which is good, but there is no way for the admin to add new categories without a contract upgrade.

**Acceptance Criteria:**
- Document that category additions require a contract upgrade
- Add a migration guide template for category additions
- Consider a string-based category with an admin-managed allowlist for flexibility

**Priority:** Low
**Effort:** Small
**Labels:** `smart-contract`, `product`

---

### MISC-007 · No analytics or reporting beyond global stats

**Description:** `get_global_payment_stats` provides only aggregate totals. There are no per-merchant stats, per-token breakdowns, or time-series data.

**Acceptance Criteria:**
- Add `get_merchant_stats(env, merchant, date_start, date_end)` returning per-merchant totals
- Off-chain indexer (see BE-001) provides richer analytics
- Document the on-chain vs. off-chain analytics split

**Priority:** Medium
**Effort:** Medium
**Labels:** `smart-contract`, `feature`, `product`

---

### MISC-008 · No test environment seeding script

**Description:** New developers have no way to quickly populate a local or testnet environment with sample merchants, payments, and refunds for manual testing.

**Acceptance Criteria:**
- Add `scripts/seed.sh` that registers 3 merchants, processes 10 payments, and initiates 2 refunds
- Script uses the Stellar CLI and reads config from `config/local.toml`
- README documents how to run the seed script

**Priority:** Low
**Effort:** Small
**Labels:** `dx`, `devops`

---

### MISC-009 · License header missing from source files

**Description:** None of the Rust source files contain an SPDX license header. This is a requirement for many open-source compliance tools.

**Acceptance Criteria:**
- Add `// SPDX-License-Identifier: MIT` to the top of every `.rs` file
- Add a CI check (`cargo deny` or a custom script) that enforces headers on new files

**Priority:** Low
**Effort:** Small
**Labels:** `compliance`, `cleanup`

---

### MISC-010 · No code of conduct

**Description:** The repository has no `CODE_OF_CONDUCT.md`, which is expected for open-source projects and required by some package registries.

**Acceptance Criteria:**
- Add `CODE_OF_CONDUCT.md` based on the Contributor Covenant v2.1
- Link from README and CONTRIBUTING.md
- Designate a contact for code of conduct reports

**Priority:** Low
**Effort:** Small
**Labels:** `process`, `community`

---

---

## 🔐 Smart Contracts (continued)

---

### SC-031 · No check for duplicate signers in `required_signers` list [COMPLETED]

**Description:** `initiate_multisig_payment` does not deduplicate `required_signers`. The same address appearing twice lets one signer satisfy two slots and execute a payment alone.

**Acceptance Criteria:**
- Deduplicate `required_signers` at initiation, OR return `InvalidInput` on duplicates
- Test: duplicate signer → `InvalidInput`

**Priority:** Critical
**Effort:** Small
**Labels:** `smart-contract`, `security`, `bug`

---

### SC-032 · `PaymentRecord` has no `description` field

**Description:** `PaymentOrder` carries a `description` but `PaymentRecord` (the stored receipt) drops it. Payers and merchants cannot retrieve the payment description after the fact.

**Acceptance Criteria:**
- Add `description: String` to `PaymentRecord`
- Populate it from `order.description` in both `process_payment_with_signature` and `execute_multisig_payment`
- Tests assert description is retrievable via `get_payment_by_id`

**Priority:** Low
**Effort:** Small
**Labels:** `smart-contract`, `feature`

---

### SC-033 · No way to query all refunds for a given order

**Description:** There is no index of refund IDs per order. Callers must know refund IDs in advance; there is no discovery mechanism.

**Acceptance Criteria:**
- Maintain a `OrderRefunds(String)` index mapping order_id → `Vec<String>` of refund IDs
- Add `get_order_refunds(env, caller, order_id) -> Vec<RefundRecord>` function
- Access restricted to payer, merchant, or admin
- Tests cover single and multiple refunds per order

**Priority:** Medium
**Effort:** Medium
**Labels:** `smart-contract`, `feature`

---

### SC-034 · `SortField` only supports `Date` and `Amount` — no `Status` sort

**Description:** Merchants reviewing refunded payments cannot sort by status, forcing them to scan all records manually.

**Acceptance Criteria:**
- Add `SortField::Status` variant
- `paginate_payments` sorts by `PaymentStatus` ordinal when selected
- Tests cover status sort ascending and descending

**Priority:** Low
**Effort:** Small
**Labels:** `smart-contract`, `feature`

---

### SC-035 · No `get_multisig_payment` query function

**Description:** There is no way to inspect the state of a multisig payment (who has signed, whether it is executed) without executing it.

**Acceptance Criteria:**
- Add `get_multisig_payment(env, caller, payment_id) -> MultisigPayment`
- Access restricted to required signers or admin
- Tests cover retrieval before and after signing

**Priority:** Medium
**Effort:** Small
**Labels:** `smart-contract`, `feature`

---

## 🧪 Testing (continued)

---

### T-016 · No test for `get_payment_by_id` by unauthorised caller

**Description:** The access control check (payer, merchant, or admin only) is untested for the rejection path.

**Acceptance Criteria:**
- Test: random address calls `get_payment_by_id` → `Unauthorized`

**Priority:** Medium
**Effort:** Small
**Labels:** `testing`, `security`

---

### T-017 · No test for `initiate_refund` by an address that is neither payer nor merchant

**Description:** The unauthorised initiator path in `initiate_refund` is untested.

**Acceptance Criteria:**
- Test: random caller initiates refund → `Unauthorized`

**Priority:** Medium
**Effort:** Small
**Labels:** `testing`, `security`

---

### T-018 · No test for `execute_refund` on a rejected refund

**Description:** Attempting to execute a rejected refund should return `RefundNotApproved` but this path is untested.

**Acceptance Criteria:**
- Test: reject a refund, then call `execute_refund` → `RefundNotApproved`

**Priority:** Medium
**Effort:** Small
**Labels:** `testing`

---

### T-019 · No test for `sign_multisig_payment` by a non-required signer

**Description:** The `Unauthorized` path in `sign_multisig_payment` (signer not in `required_signers`) is untested.

**Acceptance Criteria:**
- Test: address not in `required_signers` calls `sign_multisig_payment` → `Unauthorized`

**Priority:** Medium
**Effort:** Small
**Labels:** `testing`, `security`

---

### T-020 · No test for `process_payment_with_signature` against an inactive merchant

**Description:** The `MerchantInactive` guard in payment processing is untested.

**Acceptance Criteria:**
- Test: deactivate merchant, then attempt payment → `MerchantInactive`

**Priority:** Medium
**Effort:** Small
**Labels:** `testing`

---

## 🔒 Security (continued)

---

### SEC-011 · No maximum refund count per order

**Description:** An unlimited number of refund records can be created for a single order (each with a unique `refund_id`), inflating storage unboundedly even if each individual amount is valid.

**Acceptance Criteria:**
- Enforce a maximum of N pending refunds per order (e.g. 10)
- Return `InvalidInput` when the limit is exceeded
- Test covers the limit boundary

**Priority:** Medium
**Effort:** Small
**Labels:** `security`, `smart-contract`, `validation`

---

### SEC-012 · Ledger timestamp used for refund window — manipulable by validators

**Description:** `env.ledger().timestamp()` is used to enforce the 30-day refund window. Stellar validators can drift timestamps within protocol-allowed bounds, potentially allowing a refund slightly outside the window.

**Acceptance Criteria:**
- Document the timestamp trust model in code comments and README
- Consider using ledger sequence number instead of timestamp for time-sensitive windows
- Add a small grace buffer (e.g. 1 hour) to the refund window check

**Priority:** Medium
**Effort:** Small
**Labels:** `security`, `smart-contract`

---

## ⚙️ DevOps (continued)

---

### DO-016 · No automated testnet smoke test after deployment

**Description:** After deploying to testnet (DO-002), there is no automated smoke test that invokes key contract functions to verify the deployment is healthy.

**Acceptance Criteria:**
- Add a `scripts/smoke-test.sh` that: sets admin, registers a merchant, processes a payment, checks stats
- Smoke test runs automatically after the `deploy-testnet` CI job
- Failure alerts the team via CI notification

**Priority:** Medium
**Effort:** Small
**Dependencies:** DO-002
**Labels:** `devops`, `testing`

---

### DO-017 · No dependency update automation (Dependabot / Renovate)

**Description:** `soroban-sdk` and other dependencies are pinned but never automatically updated. Security patches may be missed.

**Acceptance Criteria:**
- Enable GitHub Dependabot for Cargo dependencies
- Configure weekly update schedule
- Auto-merge patch updates that pass CI; require review for minor/major

**Priority:** Medium
**Effort:** Small
**Labels:** `devops`, `security`

---

### DO-018 · CI cache key does not include `rust-toolchain` version

**Description:** The cargo cache key is `${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}`. If the Rust toolchain version changes without a `Cargo.lock` change, a stale cache may be used.

**Acceptance Criteria:**
- Include `rustc --version` output or toolchain channel in the cache key
- Verify cache invalidates correctly on toolchain upgrade

**Priority:** Low
**Effort:** Small
**Labels:** `devops`, `ci-cd`

---

## 📚 Documentation (continued)

---

### DOC-011 · No migration guide for breaking contract changes

**Description:** When the contract ABI changes (new fields, renamed functions), existing integrators have no migration path documented.

**Acceptance Criteria:**
- Create `docs/migrations/` directory
- Add a migration guide template
- Each breaking release includes a migration guide entry

**Priority:** Medium
**Effort:** Small
**Labels:** `documentation`, `process`

---

### DOC-012 · README does not explain the ed25519 signing flow end-to-end

**Description:** The signing flow (who generates the key pair, who signs, what the payload is) is not explained. Integrators cannot implement it without reading the source code.

**Acceptance Criteria:**
- Add a "Payment Signing Flow" section to README with a step-by-step diagram or numbered list
- Include a code example (TypeScript or shell) showing key generation, payload construction, and signing
- Cross-reference SEC-002 fix once implemented

**Priority:** High
**Effort:** Small
**Labels:** `documentation`, `integration`

---

## 🗂️ Miscellaneous / Product (continued)

---

### MISC-011 · No support for payment metadata / custom fields

**Description:** `PaymentOrder.description` is a single string. Merchants often need structured metadata (e.g. line items, customer ID, invoice number) attached to a payment.

**Acceptance Criteria:**
- Add an optional `metadata: Option<String>` field to `PaymentOrder` and `PaymentRecord` (JSON string, max 512 bytes)
- Validate length at processing time
- Tests cover metadata storage and retrieval

**Priority:** Low
**Effort:** Small
**Labels:** `smart-contract`, `feature`, `product`

---

### MISC-012 · No support for cancelling a pending multisig payment

**Description:** Once initiated, a multisig payment cannot be cancelled. If a required signer is unavailable, funds (once escrow is implemented) are locked indefinitely.

**Acceptance Criteria:**
- Add `cancel_multisig_payment(env, initiator, payment_id)` callable by the initiator before execution
- Releases escrowed funds back to the initiator
- Emits `multisig_cancelled` event
- Tests cover cancellation and rejection of cancellation by non-initiator

**Priority:** Medium
**Effort:** Small
**Labels:** `smart-contract`, `feature`, `product`

---

### MISC-013 · No support for tipping / overpayment

**Description:** The contract enforces exact payment amounts. Some use cases (donations, tips) require the payer to send more than the order amount.

**Acceptance Criteria:**
- Add an optional `min_amount: i128` field to `PaymentOrder` (amount becomes the actual transfer, min_amount is the floor)
- Or add a separate `process_donation` function with no fixed amount
- Document the intended use case

**Priority:** Low
**Effort:** Small
**Labels:** `smart-contract`, `feature`, `product`

---

*Total issues: 125*
*Generated: 2026-05-22*

