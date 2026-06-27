# ADR-0004: Off-Chain Database Schema Design

## Status

Accepted

## Context

The Pulsar smart contract stores all critical state on-chain (Soroban persistent storage), but several use-cases require off-chain relational storage:

- Rich text search on merchant names
- Webhook delivery tracking
- Recurring subscriptions
- Idempotency key TTL management

## Decision

Use a relational SQL database (PostgreSQL compatible) with the schema defined in `db/schema.sql`.

**Tables and responsibilities:**

| Table | Purpose |
|---|---|
| `merchants` | Off-chain merchant profile, whitelist flag, category |
| `payments` | Transaction mirror with indexed query fields |
| `refunds` | Refund lifecycle with status tracking |
| `subscriptions` | Recurring payment schedules |
| `webhooks` | Event subscription endpoints per merchant |
| `merchant_audit_log` | Immutable change history for merchants |
| `idempotency_keys` | Deduplication cache with TTL |

**Indexing strategy:** indexes placed on all columns used in WHERE / ORDER BY clauses (merchant, payer, dates, status, amount).

**Relationships:** `payments.merchant_address` → `merchants.address`, `refunds.order_id` → `payments.order_id`, ensuring referential integrity at the DB layer.

## Consequences

- Off-chain service must sync from on-chain events to keep both stores consistent.
- `idempotency_keys` table TTL must be enforced by a background cleanup job.
