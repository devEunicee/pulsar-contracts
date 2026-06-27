# Smart Contract API Reference

This document summarizes the public API surface of the payment-processing contract and the expected behavior of each entry point.

## Contract overview

The payment-processing contract exposes functions for merchant onboarding, payment processing, refunds, multisig flows, and data queries.

## Admin functions

### `set_admin`

- Purpose: Initialise the contract administrator.
- Parameters:
  - `admin`: the address that will act as the initial admin.
- Returns: `Result<(), PaymentError>`.
- Errors:
  - `AdminAlreadySet` if the admin has already been configured.
- Notes: The caller must authenticate.

### `set_payment_cleanup_period`

- Purpose: Configure the retention window for expired payments.
- Parameters:
  - `admin`: admin address.
  - `period`: cleanup period in seconds.
- Returns: `Result<(), PaymentError>`.
- Errors:
  - `InvalidInput` for a zero value.

## Merchant functions

### `register_merchant`

- Purpose: Register a merchant for payment processing.
- Parameters:
  - `merchant_address`: merchant identity.
  - `name`, `description`, `contact_info`: merchant metadata.
  - `category`: enum-driven business category.
- Returns: `Result<(), PaymentError>`.
- Errors:
  - `MerchantAlreadyRegistered`, `InvalidInput`.

### `deactivate_merchant`

- Purpose: Disable a merchant account.
- Parameters:
  - `caller`: requester address.
  - `merchant_address`: merchant to deactivate.
- Returns: `Result<(), PaymentError>`.
- Errors:
  - `Unauthorized`, `MerchantNotFound`.

## Payment functions

### `process_payment_with_signature`

- Purpose: Process a payment after verifying a merchant signature.
- Parameters:
  - `payer`: authenticated payer address.
  - `order`: payment payload.
  - `signature`: 64-byte signature.
  - `merchant_public_key`: 32-byte public key.
- Returns: `Result<(), PaymentError>`.
- Errors:
  - `InvalidInput`, `PaymentAlreadyExists`, `PaymentExpired`, `MerchantNotFound`, `MerchantInactive`.
- Notes: The contract transfers tokens from the payer to the merchant when validation succeeds.

### `get_payment_by_id`

- Purpose: Fetch a payment record for an authorised caller.
- Parameters:
  - `caller`: caller address.
  - `order_id`: payment identifier.
- Returns: `Result<PaymentRecord, PaymentError>`.
- Errors:
  - `PaymentNotFound`, `Unauthorized`.

## Refund functions

### `initiate_refund`

- Purpose: Start a refund request for a completed payment.
- Parameters:
  - `caller`: payer or merchant.
  - `refund_id`: unique refund id.
  - `order_id`: target payment id.
  - `amount`: refund amount.
  - `reason`: human-readable reason.
- Returns: `Result<(), PaymentError>`.
- Errors:
  - `RefundWindowExpired`, `RefundAmountExceedsPayment`, `RefundAlreadyExists`.

### `approve_refund`

- Purpose: Approve an initiated refund.
- Parameters:
  - `caller`: admin or merchant.
  - `refund_id`: refund id.
- Returns: `Result<(), PaymentError>`.
- Errors:
  - `RefundNotFound`, `Unauthorized`, `RefundAlreadyApproved`.

### `execute_refund`

- Purpose: Finalise an approved refund.
- Parameters:
  - `caller`: admin or merchant.
  - `refund_id`: refund id.
- Returns: `Result<(), PaymentError>`.
- Errors:
  - `RefundNotFound`, `Unauthorized`, `RefundNotApproved`.

## Multisig functions

### `initiate_multisig_payment`

- Purpose: Create a multisig payment request requiring multiple signatures.
- Parameters:
  - `caller`: initiator.
  - `payment_id`: payment id.
  - `order`: payment payload.
  - `signers`: addresses required to authorise.
  - `threshold`: required number of signatures.
- Returns: `Result<(), PaymentError>`.
- Errors:
  - `InvalidInput`, `Unauthorized`, `PaymentAlreadyExists`.

### `submit_multisig_signature`

- Purpose: Add a signature to a pending multisig payment.
- Parameters:
  - `caller`: signer.
  - `payment_id`: pending payment id.
  - `signature`: signature bytes.
- Returns: `Result<(), PaymentError>`.
- Errors:
  - `InvalidInput`, `PaymentNotFound`, `Unauthorized`.

## Events

The contract emits events for major state transitions:

- `merchant_registered`
- `payment_processed`
- `refund_initiated`
- `refund_approved`
- `refund_executed`
- `multisig_payment_created`

## Storage and state changes

- Merchant records are stored by merchant address.
- Payment records are stored by order id.
- Refund records are stored by refund id.
- Global statistics and merchant payment history are updated whenever payments or refunds change.

## Gas and performance notes

- Signature verification and token transfer are the most expensive operations.
- Keep payload sizes small and avoid unnecessary storage writes.
- Use the cleanup period and pagination to limit on-chain history growth.
