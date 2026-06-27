# Idempotency Key Support

## Overview

Every payment and refund operation accepts an `Idempotency-Key` header. Sending the same key with the same request body within the TTL window returns the original response without re-executing the operation.

## How It Works

```
Client → POST /payments
         Idempotency-Key: <uuid>
         Body: { ... }

Server:
  1. check(key, "payment", body, now)
  2a. New      → execute operation, store(key, response), return response
  2b. Duplicate → return cached response (HTTP 200, same body)
  2c. Conflict  → return HTTP 422 (different body for same key)
```

## TTL

Default: **24 hours** (`DEFAULT_TTL_SECONDS = 86_400`). Configurable via `IdempotencyService::with_ttl(store, seconds)`.

Expired entries are cleaned up by calling `service.cleanup(now)` from a scheduled background job.

## Key Requirements

- Keys must be **globally unique per client** — use a UUID v4.
- Keys are **per-operation type**: a key used for a payment cannot be reused for a refund (returns 422 Conflict).
- Maximum key length: 128 characters.

## Usage Example (Rust)

```rust
use idempotency::{IdempotencyService, IdempotencyResult};

// On each incoming request:
let result = svc.check(&idempotency_key, "payment", &canonical_body, now);
match result {
    IdempotencyResult::New => {
        let response = execute_payment(...)?;
        svc.store(idempotency_key, "payment".into(), &canonical_body, response.clone(), now);
        return Ok(response);
    }
    IdempotencyResult::Duplicate(cached) => return Ok(cached),
    IdempotencyResult::Conflict => return Err(http_422("Idempotency key reused with different request")),
}
```

## Storage Backend

Implement `IdempotencyStore` for your backend (PostgreSQL, Redis, etc.):

```rust
impl IdempotencyStore for MyDbStore {
    fn get(&self, key: &str) -> Option<IdempotentEntry> { ... }
    fn set(&mut self, entry: IdempotentEntry) { ... }
    fn evict_expired(&mut self, now: u64) { ... }
}
```

The `idempotency_keys` table in `db/schema.sql` provides a ready-made PostgreSQL schema.
