# Query Optimization Monitoring

Relates to: [#301](https://github.com/devEunicee/pulsar-contracts/issues/301)

## Overview

This document defines the strategy for monitoring query performance and identifying optimization opportunities in the Pulsar off-chain indexing layer.

## Slow Query Log

Enable slow query logging at the database level:

```sql
-- PostgreSQL
ALTER SYSTEM SET log_min_duration_statement = 1000; -- log queries > 1 s
ALTER SYSTEM SET log_statement = 'none';
SELECT pg_reload_conf();
```

Any query exceeding **1 second** is written to the slow query log and triggers an alert.

## Query Execution Plan Analysis

Use `EXPLAIN ANALYZE` to inspect plans for payment-history and merchant-lookup queries:

```sql
EXPLAIN (ANALYZE, BUFFERS, FORMAT JSON)
SELECT * FROM payments
WHERE merchant_address = $1
ORDER BY paid_at DESC
LIMIT 100;
```

Review:
- Sequential scans on large tables → candidate for index.
- Nested-loop joins with high row estimates → rewrite or materialise.

## Performance Baselines

| Query | P50 (ms) | P95 (ms) | P99 (ms) |
|---|---|---|---|
| `get_merchant_payment_history` | < 10 | < 50 | < 200 |
| `get_payer_payment_history`   | < 10 | < 50 | < 200 |
| `get_payment_by_id`           | < 5  | < 20 | < 50  |
| `get_global_payment_stats`    | < 50 | < 200| < 500 |

Baselines are re-evaluated after every schema migration.

## Alerting

| Condition | Threshold | Action |
|---|---|---|
| Single query duration | > 1 s | Page on-call engineer |
| P99 latency (5-min window) | > 500 ms | Slack `#alerts-db` |
| Error rate | > 1 % | PagerDuty |

## Index Usage Monitoring

Identify unused indexes weekly:

```sql
SELECT schemaname, tablename, indexname, idx_scan
FROM pg_stat_user_indexes
WHERE idx_scan = 0
ORDER BY schemaname, tablename;
```

Indexes with `idx_scan = 0` after 30 days in production are candidates for removal.

## Regular Optimization Reviews

- **Weekly**: review slow query log, triage new entries.
- **Monthly**: run `VACUUM ANALYZE` on high-write tables, update table statistics.
- **Quarterly**: full index audit; drop unused, add missing, rewrite fragmented.

## Relevant Indexes

```sql
-- payments table (off-chain replica)
CREATE INDEX IF NOT EXISTS idx_payments_merchant_paid_at
    ON payments (merchant_address, paid_at DESC);

CREATE INDEX IF NOT EXISTS idx_payments_payer_paid_at
    ON payments (payer, paid_at DESC);

CREATE INDEX IF NOT EXISTS idx_payments_order_id
    ON payments (order_id);
```
