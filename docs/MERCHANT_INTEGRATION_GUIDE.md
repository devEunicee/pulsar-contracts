# Pulsar — Merchant Integration Guide

> Step-by-step guide for integrating Pulsar payment processing into your application.

---

## Table of Contents

1. [Overview](#1-overview)
2. [Prerequisites](#2-prerequisites)
3. [Step-by-Step Integration](#3-step-by-step-integration)
4. [SDK Usage Examples](#4-sdk-usage-examples)
5. [Code Samples](#5-code-samples)
6. [Error Handling](#6-error-handling)
7. [Event Handling](#7-event-handling)
8. [Testing in Sandbox](#8-testing-in-sandbox)
9. [Go-Live Checklist](#9-go-live-checklist)
10. [Support](#10-support)

---

## 1. Overview

Pulsar is a Soroban smart contract on the Stellar network that handles payment processing, refunds, and multi-signature payments for merchants. Integration involves:

1. Registering your merchant account on-chain
2. Generating and signing payment orders
3. Submitting payments via the contract
4. Handling events and refunds

---

## 2. Prerequisites

| Requirement | Notes |
|---|---|
| Stellar keypair | Your merchant public/private key pair |
| Funded account | Minimum ~5 XLM for storage reserves and fees |
| Contract ID | Provided at deployment; saved as `CONTRACT_ID` |
| Stellar SDK | JavaScript, Python, Go, or Rust (see §5) |

**Generate a Stellar keypair:**

```bash
stellar keys generate --global merchant
stellar keys address merchant   # your public key (G...)
stellar keys show merchant      # your secret key (S...) — keep private
```

**Fund testnet account:**

```bash
curl "https://friendbot.stellar.org?addr=$(stellar keys address merchant)"
```

---

## 3. Step-by-Step Integration

### Step 1 — Register as a Merchant

Call `register_merchant` once to create your on-chain merchant profile.

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source-account <MERCHANT_SECRET_KEY> \
  --network testnet \
  -- register_merchant \
  --merchant_address <MERCHANT_ADDRESS> \
  --name "Your Store Name" \
  --description "Brief store description" \
  --contact_info "support@yourstore.com" \
  --category Retail
```

**Categories**: `Retail` | `Food` | `Services` | `Digital` | `Other`

### Step 2 — Construct a Payment Order

A payment order is a JSON object signed by your merchant private key. Build it server-side and never expose the private key to clients.

```json
{
  "order_id": "ORDER_20240101_001",
  "merchant_address": "<MERCHANT_ADDRESS>",
  "payer": "<PAYER_ADDRESS>",
  "token": "<TOKEN_CONTRACT_ADDRESS>",
  "amount": 1000,
  "description": "Product purchase",
  "expires_at": 1735689600
}
```

| Field | Type | Notes |
|---|---|---|
| `order_id` | string | Unique per payment; used as replay-attack tombstone |
| `merchant_address` | address | Your registered merchant address |
| `payer` | address | Customer's Stellar address |
| `token` | address | Token contract (e.g., USDC on testnet) |
| `amount` | i128 | Amount in token's smallest unit (stroops for XLM) |
| `description` | string | Human-readable description |
| `expires_at` | u64 | Unix timestamp; `0` means never expires |

### Step 3 — Sign the Order

Serialize the order to bytes (canonical JSON, UTF-8 encoded) and sign with your ed25519 private key.

```javascript
// Node.js example
const { Keypair } = require('@stellar/stellar-sdk');
const crypto = require('crypto');

const keypair = Keypair.fromSecret('<MERCHANT_SECRET_KEY>');
const orderBytes = Buffer.from(JSON.stringify(order), 'utf8');
const signature = keypair.sign(orderBytes);  // returns 64-byte Buffer
const signatureHex = signature.toString('hex');
```

### Step 4 — Submit the Payment

The payer (or your backend on their behalf) calls `process_payment_with_signature`.

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source-account <PAYER_SECRET_KEY> \
  --network testnet \
  -- process_payment_with_signature \
  --payer <PAYER_ADDRESS> \
  --order '{"order_id":"ORDER_20240101_001","merchant_address":"...","payer":"...","token":"...","amount":1000,"description":"Purchase","expires_at":0}' \
  --signature <64_BYTE_HEX_SIGNATURE> \
  --merchant_public_key <32_BYTE_HEX_PUBLIC_KEY>
```

### Step 5 — Verify the Payment

Query payment status by order ID:

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source-account <MERCHANT_SECRET_KEY> \
  --network testnet \
  -- get_payment_by_id \
  --caller <MERCHANT_ADDRESS> \
  --order_id "ORDER_20240101_001"
```

---

## 4. SDK Usage Examples

### JavaScript / TypeScript (`@stellar/stellar-sdk`)

```typescript
import { Contract, Keypair, Networks, TransactionBuilder, BASE_FEE, nativeToScVal } from '@stellar/stellar-sdk';
import { Server } from '@stellar/stellar-sdk/rpc';

const server = new Server('https://soroban-testnet.stellar.org');
const merchantKeypair = Keypair.fromSecret(process.env.MERCHANT_SECRET!);
const contract = new Contract(process.env.CONTRACT_ID!);

async function registerMerchant() {
  const account = await server.getAccount(merchantKeypair.publicKey());

  const tx = new TransactionBuilder(account, {
    fee: BASE_FEE,
    networkPassphrase: Networks.TESTNET,
  })
    .addOperation(
      contract.call(
        'register_merchant',
        nativeToScVal(merchantKeypair.publicKey(), { type: 'address' }),
        nativeToScVal('My Store'),
        nativeToScVal('Store description'),
        nativeToScVal('support@store.com'),
        nativeToScVal({ Retail: null })
      )
    )
    .setTimeout(30)
    .build();

  tx.sign(merchantKeypair);
  const result = await server.sendTransaction(tx);
  return result;
}
```

### Python (`stellar-sdk`)

```python
from stellar_sdk import Keypair, Network, SorobanServer, TransactionBuilder
from stellar_sdk.soroban_rpc import SendTransactionStatus

server = SorobanServer("https://soroban-testnet.stellar.org")
keypair = Keypair.from_secret(os.environ["MERCHANT_SECRET"])
contract_id = os.environ["CONTRACT_ID"]

def register_merchant(name: str, description: str, contact: str):
    account = server.load_account(keypair.public_key)
    tx = (
        TransactionBuilder(account, network_passphrase=Network.TESTNET_NETWORK_PASSPHRASE)
        .append_invoke_contract_function_op(
            contract_id=contract_id,
            function_name="register_merchant",
            parameters=[
                scval.to_address(keypair.public_key),
                scval.to_string(name),
                scval.to_string(description),
                scval.to_string(contact),
                scval.to_enum("Retail", None),
            ],
        )
        .set_timeout(30)
        .build()
    )
    tx.sign(keypair)
    response = server.send_transaction(tx)
    return response
```

### Go (`stellar/go`)

```go
package main

import (
    "github.com/stellar/go/clients/horizonclient"
    "github.com/stellar/go/keypair"
    "github.com/stellar/go/network"
    "github.com/stellar/go/txnbuild"
)

func registerMerchant(secretKey, contractID, name string) error {
    kp := keypair.MustParseFull(secretKey)
    client := horizonclient.DefaultTestNetClient

    ar := horizonclient.AccountRequest{AccountID: kp.Address()}
    account, err := client.AccountDetail(ar)
    if err != nil {
        return err
    }

    tx, err := txnbuild.NewTransaction(txnbuild.TransactionParams{
        SourceAccount:        &account,
        IncrementSequenceNum: true,
        BaseFee:              txnbuild.MinBaseFee,
        Timebounds:           txnbuild.NewTimeout(30),
        Operations: []txnbuild.Operation{
            &txnbuild.InvokeHostFunction{
                HostFunction: xdr.HostFunction{
                    // ... contract invocation
                },
            },
        },
    })
    // sign and submit ...
    return nil
}
```

---

## 5. Code Samples

### Generate and Sign a Payment Order

```typescript
import { Keypair } from '@stellar/stellar-sdk';

interface PaymentOrder {
  order_id: string;
  merchant_address: string;
  payer: string;
  token: string;
  amount: number;
  description: string;
  expires_at: number;
}

function signOrder(order: PaymentOrder, merchantSecret: string): string {
  const keypair = Keypair.fromSecret(merchantSecret);
  const orderBytes = Buffer.from(JSON.stringify(order), 'utf8');
  const signature = keypair.sign(orderBytes);
  return signature.toString('hex');
}

// Usage
const order: PaymentOrder = {
  order_id: `ORDER_${Date.now()}`,
  merchant_address: 'G...',
  payer: 'G...',
  token: 'C...',
  amount: 1000,
  description: 'Product purchase',
  expires_at: Math.floor(Date.now() / 1000) + 3600, // 1 hour from now
};

const signature = signOrder(order, process.env.MERCHANT_SECRET!);
```

### Initiate a Refund

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source-account <MERCHANT_OR_PAYER_KEY> \
  --network testnet \
  -- initiate_refund \
  --caller <CALLER_ADDRESS> \
  --refund_id "REFUND_001" \
  --order_id "ORDER_20240101_001" \
  --amount 500 \
  --reason "Customer request"
```

### Query Payment History

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source-account <MERCHANT_KEY> \
  --network testnet \
  -- get_merchant_payment_history \
  --merchant <MERCHANT_ADDRESS> \
  --cursor null \
  --limit 20 \
  --filter null \
  --sort_field Date \
  --sort_order Descending
```

---

## 6. Error Handling

Map contract error codes to user-friendly messages in your application:

| Code | Name | Recommended Action |
|---|---|---|
| 1 | `Unauthorized` | Verify caller address and permissions |
| 10 | `MerchantNotFound` | Register merchant first |
| 11 | `MerchantAlreadyRegistered` | Skip registration; proceed to payment |
| 12 | `MerchantInactive` | Contact support to reactivate |
| 20 | `PaymentNotFound` | Check `order_id` is correct |
| 21 | `PaymentAlreadyExists` | Generate a new unique `order_id` |
| 22 | `InvalidAmount` | Ensure amount > 0 |
| 23 | `InvalidSignature` | Re-sign the order bytes; verify key pair matches |
| 24 | `PaymentExpired` | Generate a new order with updated `expires_at` |
| 25 | `InsufficientBalance` | Merchant needs more token balance |
| 32 | `RefundWindowExpired` | Refund past 30-day window; cannot proceed |
| 33 | `RefundAmountExceedsPayment` | Reduce refund amount |
| 43 | `InsufficientSignatures` | All required signers must sign first |
| 50 | `InvalidInput` | Validate all input fields before submitting |

### Error Handling Pattern (TypeScript)

```typescript
async function processPayment(order: PaymentOrder, signature: string): Promise<string> {
  try {
    const result = await invokeContract('process_payment_with_signature', order, signature);
    return result.orderId;
  } catch (err: any) {
    const code = extractContractErrorCode(err);
    switch (code) {
      case 21:
        // Regenerate order_id and retry
        return processPayment({ ...order, order_id: generateOrderId() }, signature);
      case 23:
        throw new Error('Payment signature invalid. Please contact support.');
      case 24:
        throw new Error('Payment order expired. Please initiate a new payment.');
      default:
        throw new Error(`Payment failed with error ${code}: ${err.message}`);
    }
  }
}
```

---

## 7. Event Handling

The contract emits events for every state change. Subscribe via Horizon event stream or your own indexer.

| Event | Trigger | Key Fields |
|---|---|---|
| `payment_processed` | Successful payment | `order_id`, `merchant`, `payer`, `amount`, `token` |
| `refund_initiated` | Refund request created | `refund_id`, `order_id`, `amount` |
| `refund_approved` | Refund approved | `refund_id` |
| `refund_rejected` | Refund rejected | `refund_id` |
| `refund_executed` | Funds transferred back | `refund_id`, `amount` |
| `merchant_registered` | New merchant | `merchant_address`, `name` |
| `merchant_deactivated` | Merchant deactivated | `merchant_address` |
| `multisig_executed` | Multi-sig payment done | `payment_id` |

### Listening for Events (JavaScript)

```javascript
const { Horizon } = require('@stellar/stellar-sdk');

const server = new Horizon.Server('https://horizon-testnet.stellar.org');

// Poll for payment events on your merchant account
server.payments()
  .forAccount('<MERCHANT_ADDRESS>')
  .cursor('now')
  .stream({
    onmessage: (payment) => {
      if (payment.type === 'payment') {
        console.log(`Received payment: ${payment.amount} ${payment.asset_code}`);
        // Update your database, fulfill order, etc.
      }
    },
    onerror: (error) => console.error('Stream error:', error),
  });
```

### Webhook Pattern

For production, implement a webhook dispatcher:

1. Run an indexer service that subscribes to Soroban contract events
2. Filter events by `CONTRACT_ID` and event type
3. POST to your registered webhook URLs with event payload
4. Implement idempotency using `order_id` / `refund_id` as deduplication keys

---

## 8. Testing in Sandbox

### Local Network Setup

```bash
# Start local Stellar network
docker compose up -d

# Build contract
cargo build --target wasm32-unknown-unknown --release

# Deploy to local
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/payment_processing_contract.wasm \
  --source-account <SECRET_KEY> \
  --network local

export CONTRACT_ID="<returned contract ID>"
```

### Seed Test Data

```bash
bash scripts/seed.sh config/local.toml
```

This creates 3 test merchants, 10 payments, and 2 refunds for end-to-end testing.

### Test Accounts

Use Friendbot for testnet accounts:

```bash
# Create and fund test accounts
for role in merchant payer admin; do
  stellar keys generate --global test_$role
  curl -s "https://friendbot.stellar.org?addr=$(stellar keys address test_$role)" > /dev/null
  echo "$role: $(stellar keys address test_$role)"
done
```

### Testnet Endpoints

| Service | URL |
|---|---|
| Horizon (testnet) | `https://horizon-testnet.stellar.org` |
| Soroban RPC (testnet) | `https://soroban-testnet.stellar.org` |
| Friendbot | `https://friendbot.stellar.org` |
| Block explorer | `https://stellar.expert/explorer/testnet` |

### Verifying Integration

```bash
# Check merchant registered
stellar contract invoke --id $CONTRACT_ID --source-account <KEY> --network testnet \
  -- get_merchant --merchant_address <MERCHANT_ADDRESS>

# Check payment recorded
stellar contract invoke --id $CONTRACT_ID --source-account <KEY> --network testnet \
  -- get_payment_by_id --caller <ADDRESS> --order_id "ORDER_001"

# Check merchant stats
stellar contract invoke --id $CONTRACT_ID --source-account <MERCHANT_KEY> --network testnet \
  -- get_merchant_stats \
  --merchant <MERCHANT_ADDRESS> \
  --date_start null \
  --date_end null
```

---

## 9. Go-Live Checklist

### Security

- [ ] Merchant private key stored in a secrets manager (AWS Secrets Manager, HashiCorp Vault, etc.)
- [ ] Private key never logged or exposed in API responses
- [ ] Order signing happens server-side only
- [ ] `order_id` generation is unique and collision-resistant (UUID v4 or similar)
- [ ] `expires_at` set to a reasonable window (e.g., 15–60 minutes)
- [ ] Input validation on all fields before signing

### Infrastructure

- [ ] Merchant account funded with sufficient XLM for storage reserves (~5 XLM minimum)
- [ ] Token allowance granted for expected payment volumes
- [ ] RPC endpoint configured for mainnet (`https://soroban-testnet.stellar.org` → mainnet equivalent)
- [ ] Network passphrase updated to `Public Global Stellar Network ; September 2015`
- [ ] Retry logic implemented for transient RPC failures (exponential backoff)
- [ ] Rate limiting in place on your API gateway to prevent spam

### Testing

- [ ] All happy-path flows tested on testnet (register, pay, refund)
- [ ] Error codes handled gracefully in UI/API
- [ ] Event stream / webhook integration verified
- [ ] Multi-sig flow tested if applicable
- [ ] Load test completed for expected peak transaction volume

### Monitoring

- [ ] Alerts configured for failed payment events
- [ ] Merchant balance monitored (low XLM triggers alert)
- [ ] Refund queue monitored (pending refunds older than X days)
- [ ] On-call runbook documented

### Compliance

- [ ] KYC/AML requirements reviewed with legal team
- [ ] Refund policy communicated to customers (30-day window)
- [ ] Transaction records archived per regulatory requirements

---

## 10. Support

| Channel | Details |
|---|---|
| GitHub Issues | [devEunicee/pulsar-contracts/issues](https://github.com/devEunicee/pulsar-contracts/issues) |
| Contributing Guide | [CONTRIBUTING.md](../CONTRIBUTING.md) |
| Security Reports | See [SECURITY.md](SECURITY.md) for the vulnerability disclosure policy |
| Contract API Reference | [smart-contract-api.md](smart-contract-api.md) |
| Seeding Guide | [SEEDING_GUIDE.md](SEEDING_GUIDE.md) |
| Analytics Guide | [ANALYTICS_GUIDE.md](ANALYTICS_GUIDE.md) |

When filing a bug report, include:
- Network (local / testnet / mainnet)
- Contract ID
- Transaction hash (if available)
- Error code and full error message
- Steps to reproduce
