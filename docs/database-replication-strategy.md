# Database Replication Strategy

Relates to: [#303](https://github.com/devEunicee/pulsar-contracts/issues/303)

## Overview

This document describes the replication architecture for the Pulsar off-chain indexing layer, providing high availability and horizontal read scaling.

## Architecture

```
             ┌─────────────────┐
             │   Primary (RW)  │
             └────────┬────────┘
          async streaming replication
         ┌────────────┴────────────┐
         ▼                         ▼
 ┌──────────────┐         ┌──────────────┐
 │  Replica 1   │         │  Replica 2   │
 │  (read-only) │         │  (standby)   │
 └──────────────┘         └──────────────┘
```

- **Primary**: handles all writes (payment inserts, merchant updates).
- **Replica 1**: serves read queries (`get_payment_history`, `get_global_stats`).
- **Replica 2**: warm standby for automatic failover.

## Primary-Replica Setup (PostgreSQL)

### Primary (`postgresql.conf`)

```conf
wal_level = replica
max_wal_senders = 5
wal_keep_size = 1GB
synchronous_commit = on
```

### Replica (`recovery.conf` / `postgresql.conf`)

```conf
primary_conninfo = 'host=<primary-host> port=5432 user=replicator password=<secret>'
hot_standby = on
```

### Replication user

```sql
CREATE ROLE replicator REPLICATION LOGIN PASSWORD '<secret>';
```

## Automatic Failover

Use **Patroni** (or pgAutoFailover) to manage leader election:

1. All nodes register with a distributed config store (etcd / Consul).
2. On primary failure, the node with the lowest replication lag is promoted.
3. Other replicas re-attach to the new primary automatically.
4. Application connection strings point to the Patroni VIP / HAProxy endpoint — no code change required on failover.

## Read Replicas for Queries

Route read-only queries to Replica 1 at the application layer:

```rust
// pseudo-code: connection pool selection
fn db_pool(write: bool) -> &Pool {
    if write { &PRIMARY_POOL } else { &REPLICA_POOL }
}
```

Affected read paths: `get_merchant_payment_history`, `get_payer_payment_history`, `get_payment_by_id`, `get_global_payment_stats`.

## Replication Lag Monitoring

```sql
-- Run on primary
SELECT
    client_addr,
    state,
    sent_lsn,
    replay_lsn,
    (sent_lsn - replay_lsn) AS lag_bytes
FROM pg_stat_replication;
```

Alert when `lag_bytes > 10 MB` or lag age > 30 s.

## Eventual Consistency Handling

| Operation | Source | Notes |
|---|---|---|
| Write + immediate read | Primary | Route both to primary within same request |
| Background stats queries | Replica | Tolerate lag up to 30 s |
| Refund status checks | Primary | Correctness-critical; always primary |

## Tested Failover Process

1. Simulate primary failure: `pg_ctl stop -m immediate`.
2. Verify Patroni promotes Replica 2 within < 30 s.
3. Confirm application reconnects and writes succeed.
4. Restore old primary as new replica; verify it reattaches and catches up.
5. Document RTO (target < 30 s) and RPO (target 0 with synchronous commit).

## Documentation

- Runbook: `docs/runbooks/db-failover.md` (to be created by ops).
- Connection strings managed via environment variables; never hardcoded.
- Secrets stored in the project's secrets manager (e.g., AWS Secrets Manager).
