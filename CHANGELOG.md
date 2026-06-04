# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

- Add `ping` endpoint to return current ledger timestamp for health monitoring.
- Remove unused `StorageError` variant from the payment contract ABI.
- Harden payment and refund flows against external token transfer re-entrancy by committing state before external calls.
- Add best-effort zero/burn admin address validation and test coverage for invalid admin assignment.
- Add date-filter coverage for `get_global_payment_stats` to cover all-payments and no-payments ranges.
