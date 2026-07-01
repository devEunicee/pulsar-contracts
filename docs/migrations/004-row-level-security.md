# Migration 004 — Row-Level Security

**Issue:** #306  
**Date:** 2026-06-29

## Summary

Enables PostgreSQL Row-Level Security (RLS) on `payments`, `refunds`, `merchants`, and `merchant_audit_log`. Adds an `rls_access_log` table for auditing access attempts.

## Roles

Three DB roles are required before applying this migration:

```sql
CREATE ROLE pulsar_admin;
CREATE ROLE pulsar_merchant;
CREATE ROLE pulsar_customer;
```

Grant each role CONNECT and appropriate table privileges, then assign roles per session via `SET ROLE` after JWT validation.

## Runtime context

The API layer sets `app.current_user` (Stellar address) and `app.current_role` via `set_config()` on each request. Policies reference these settings to filter rows.

## Rollback

Run the `-- DOWN` block at the bottom of `0004_row_level_security.sql`.
