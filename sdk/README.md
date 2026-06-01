# @pulsar-contracts/sdk

TypeScript/JavaScript SDK for creating and signing Pulsar `PaymentOrder` structs before submitting them to the Soroban smart contract.

## Installation

```bash
npm install @pulsar-contracts/sdk
```

## Usage

```typescript
import { createOrder, signOrder, getPublicKey } from '@pulsar-contracts/sdk';

// 1. Build the order
const order = createOrder({
  orderId: 'ORDER_001',
  merchantAddress: 'G...',
  payer: 'G...',
  token: 'C...',
  amount: 1000n,
  description: 'Coffee',
  expiresAt: 0n, // 0 = no expiry
});

// 2. Sign with merchant ed25519 private key (32-byte seed as Uint8Array)
const signature = signOrder(order, privateKeySeed);

// 3. Get the matching public key (pass to process_payment_with_signature)
const merchantPublicKey = getPublicKey(privateKeySeed);

// 4. Submit to contract via Stellar CLI or stellar-sdk
```

## API

### `createOrder(params): PaymentOrder`

Constructs a `PaymentOrder` object.

| Param             | Type     | Description                        |
|-------------------|----------|------------------------------------|
| `orderId`         | `string` | Unique order identifier            |
| `merchantAddress` | `string` | Merchant Stellar address           |
| `payer`           | `string` | Payer Stellar address              |
| `token`           | `string` | Token contract address             |
| `amount`          | `bigint` | Payment amount (smallest unit)     |
| `description`     | `string` | Human-readable description         |
| `expiresAt`       | `bigint` | Unix timestamp; `0n` = no expiry   |

### `signOrder(order, privateKey): string`

Signs the order with an ed25519 private key seed. Returns a 64-byte hex signature.

### `getPublicKey(privateKey): string`

Derives the 32-byte ed25519 public key hex from a private key seed.

### `serializeOrder(order): Uint8Array`

Returns the canonical byte representation used for signing (useful for custom signing flows).

## Building from source

```bash
cd sdk
npm install
npm run build
```
