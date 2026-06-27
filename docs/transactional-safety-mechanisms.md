# Transactional Safety Mechanisms

Relates to: [#300](https://github.com/devEunicee/pulsar-contracts/issues/300)
Depends on: #293 (Schema)

## Overview

Pulsar's off-chain indexer must protect shared state (payments, refunds, merchants) against concurrent writes. This document defines the transaction isolation level, locking strategy, conflict resolution approach, and timeout policy.

## ACID Compliance

All write operations run inside explicit transactions with **Serializable** isolation where correctness is critical (refunds, multisig), and **Read Committed** for append-only inserts (new payments):

```sql
-- Refund execution: serializable to prevent double-spend
BEGIN ISOLATION LEVEL SERIALIZABLE;
  SELECT amount, refunded_total FROM payments WHERE order_id = $1 FOR UPDATE;
  -- validate remaining refundable amount
  UPDATE payments SET refunded_total = refunded_total + $2 WHERE order_id = $1;
  INSERT INTO refunds (...) VALUES (...);
COMMIT;

-- Payment insert: read committed is sufficient (order_id is unique)
BEGIN ISOLATION LEVEL READ COMMITTED;
  INSERT INTO payments (...) VALUES (...);
COMMIT;
```

## Optimistic Locking for Updates

Merchant profile and multisig state updates use a `version` column to detect concurrent modifications without holding a lock:

```sql
ALTER TABLE merchants ADD COLUMN version BIGINT NOT NULL DEFAULT 0;

-- Update pattern
UPDATE merchants
SET name = $1, version = version + 1
WHERE merchant_address = $2
  AND version = $3;          -- $3 = version read before update
-- 0 rows updated → version mismatch → caller retries
```

The application checks `rows_affected == 0` and returns a conflict error for the caller to retry.

## Deadlock Detection and Handling

Acquire multiple row-level locks in a **consistent order** (e.g., always lock `payments` before `refunds`) to prevent circular waits:

```sql
-- Correct: payments first, then refunds
SELECT * FROM payments  WHERE order_id = $1 FOR UPDATE;
SELECT * FROM refunds   WHERE order_id = $1 FOR UPDATE;
```

PostgreSQL detects deadlocks automatically and rolls back one of the transactions with `ERROR 40P01`. The application catches this error and retries up to **3 times** with exponential back-off (50 ms, 100 ms, 200 ms).

```rust
// pseudo-code
const MAX_RETRIES: u8 = 3;
for attempt in 0..MAX_RETRIES {
    match execute_tx(&pool).await {
        Err(e) if is_deadlock(&e) => {
            sleep(50 << attempt).await;
            continue;
        }
        result => return result,
    }
}
```

## Transaction Rollback on Error

Every write path uses a transaction block. Any error triggers an automatic rollback:

```sql
BEGIN;
  -- operation A
  -- operation B
COMMIT;
-- Any mid-transaction error causes implicit ROLLBACK
```

In the application layer, database connection pool transactions are always wrapped in a closure that rolls back on `?` / `unwrap` failure:

```rust
let mut tx = pool.begin().await?;
do_work(&mut tx).await.map_err(|e| { tx.rollback().await.ok(); e })?;
tx.commit().await?;
```

## Concurrent Operation Testing

Test scenarios (run with multiple parallel workers):

| Scenario | Expected outcome |
|---|---|
| Duplicate `order_id` inserts | Second insert fails with `PaymentAlreadyExists` (unique constraint) |
| Concurrent refund requests for same order | Only one proceeds; other gets serialization failure and retries |
| Concurrent merchant deactivation + payment | Deactivation wins; payment gets `MerchantInactive` |
| Concurrent multisig sign from same signer | Second sign fails with `MultisigAlreadySigned` |

Run with `pgbench` custom script or a dedicated integration test using tokio concurrent tasks.

## Performance Under Concurrent Load

Target metrics (100 concurrent writers):

| Metric | Target |
|---|---|
| Payment insert throughput | > 500 TPS |
| Refund execution latency P99 | < 200 ms |
| Deadlock retry rate | < 0.1 % |
| Transaction abort rate | < 0.5 % |

## Timeout on Long Transactions

Prevent long-running transactions from holding locks indefinitely:

```sql
-- Per-session (set in connection pool config)
SET lock_timeout      = '5s';   -- fail if waiting > 5 s for a lock
SET statement_timeout = '10s';  -- fail if query runs > 10 s
SET idle_in_transaction_session_timeout = '30s'; -- close idle open tx
```

These are set as default parameters on the application database role:

```sql
ALTER ROLE indexer SET lock_timeout      = '5s';
ALTER ROLE indexer SET statement_timeout = '10s';
ALTER ROLE indexer SET idle_in_transaction_session_timeout = '30s';
```
