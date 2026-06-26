# ADR-0004: `expires_at == 0` treated as no-expiry

**Status:** Accepted
**Date:** 2026-06-24

## Context

The `PaymentOrder.expires_at` field is used by the contract to reject payments
that are presented after an expiry time. Historically the implementation has
treated a value of `0` as a special sentinel meaning the order never expires.
This behaviour is relied upon by existing integrations and was not documented
in the public API or type definitions.

## Decision

Keep the existing behaviour: `expires_at == 0` means "never expires". The
contract will continue to accept orders with `expires_at` set to `0` and will
not reject them as expired.

We will document this behaviour in the on-chain types (code comments), the
public README/API docs, and in this ADR to make the semantics explicit.

## Consequences

- Positive: Backwards compatibility with existing integrations that rely on
  non-expiring orders.
- Negative: Integrators may accidentally create non-expiring orders if they
  omit the field or set it to `0`. We mitigate this by documenting the behaviour
  clearly in the types and README.

## Future

If we later decide to disallow non-expiring orders, we will record a migration
plan in a follow-up ADR that includes a storage migration strategy and a
reasonable deprecation timeline.
