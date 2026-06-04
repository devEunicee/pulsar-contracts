# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

- Fix duplicate signer vulnerability in `initiate_multisig_payment` by returning `InvalidInput` (SC-031).
- Remove unused `StorageError` variant from the payment contract ABI.
- Harden payment and refund flows against external token transfer re-entrancy by committing state before external calls.
- Add best-effort zero/burn admin address validation and test coverage for invalid admin assignment.
- Add date-filter coverage for `get_global_payment_stats` to cover all-payments and no-payments ranges.
