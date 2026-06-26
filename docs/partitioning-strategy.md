# Partitioning Strategy

Relates to: [#304](https://github.com/devEunicee/pulsar-contracts/issues/304)
Depends on: #293 (Schema)

## Overview

Range partitioning on the `payments` table by `paid_at` (month) keeps individual partition sizes manageable, enables partition pruning for date-range queries, and simplifies time-based archival.

## Partition Key Selection

`paid_at TIMESTAMPTZ` is the natural partition key because:
- All major query filters include a date range.
- Historical data access patterns decrease sharply after 90 days.
- Archival / cleanup aligns with monthly boundaries.

## DDL

```sql
-- Parent table (declarative partitioning)
CREATE TABLE payments (
    order_id        TEXT        NOT NULL,
    merchant_address TEXT       NOT NULL,
    payer           TEXT        NOT NULL,
    token           TEXT        NOT NULL,
    amount          NUMERIC     NOT NULL,
    status          TEXT        NOT NULL,
    paid_at         TIMESTAMPTZ NOT NULL,
    description     TEXT,
    PRIMARY KEY (order_id, paid_at)
) PARTITION BY RANGE (paid_at);

-- Monthly partitions (bootstrap: last 12 months + current)
CREATE TABLE payments_2025_07 PARTITION OF payments
    FOR VALUES FROM ('2025-07-01') TO ('2025-08-01');

-- ... repeat for each month ...

CREATE TABLE payments_2026_06 PARTITION OF payments
    FOR VALUES FROM ('2026-06-01') TO ('2026-07-01');

-- Default partition catches rows outside defined ranges
CREATE TABLE payments_default PARTITION OF payments DEFAULT;
```

## Automated Partition Maintenance

Use `pg_partman` to create future partitions automatically:

```sql
SELECT partman.create_parent(
    p_parent_table := 'public.payments',
    p_control      := 'paid_at',
    p_type         := 'range',
    p_interval     := 'monthly',
    p_premake      := 3   -- pre-create 3 months ahead
);

-- Run maintenance weekly (e.g., via pg_cron)
SELECT cron.schedule('0 2 * * 0', $$SELECT partman.run_maintenance()$$);
```

This ensures new partitions exist before data arrives and old ones can be detached for archival.

## Query Planning Verification

Confirm partition pruning is active:

```sql
EXPLAIN (ANALYZE, BUFFERS)
SELECT * FROM payments
WHERE paid_at BETWEEN '2026-06-01' AND '2026-06-30'
  AND merchant_address = $1;
-- Expected: only payments_2026_06 in plan, no seq scan of other partitions
```

Set `enable_partition_pruning = on` (PostgreSQL default).

## Indexes Per Partition

Each partition inherits the parent index definitions:

```sql
CREATE INDEX ON payments (merchant_address, paid_at DESC);
CREATE INDEX ON payments (payer, paid_at DESC);
CREATE INDEX ON payments (order_id);
```

## Data Distribution

Monitor row distribution across partitions monthly:

```sql
SELECT
    child.relname AS partition,
    pg_size_pretty(pg_relation_size(child.oid)) AS size,
    pg_stat_user_tables.n_live_tup AS live_rows
FROM pg_inherits
JOIN pg_class child ON pg_inherits.inhrelid = child.oid
JOIN pg_stat_user_tables ON pg_stat_user_tables.relname = child.relname
ORDER BY child.relname;
```

Alert when any single partition exceeds **5 M rows** (review interval strategy).

## Performance Improvement Measurement

| Metric | Before partitioning | After partitioning | Target |
|---|---|---|---|
| `get_merchant_payment_history` P95 | baseline | measure post-deploy | -50 % |
| Full-table `VACUUM` duration | baseline | measure post-deploy | -70 % |
| Date-range query planning time | baseline | measure post-deploy | pruning active |

Run `pgbench` or the project's load-test suite before and after migration to capture actual numbers.

## Archival

Detach old partitions instead of deleting rows:

```sql
ALTER TABLE payments DETACH PARTITION payments_2025_07;
-- Optionally move to cold storage or drop after backup
```
