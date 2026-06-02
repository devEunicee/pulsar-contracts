# Pulsar Webhook Service

Off-chain service that listens to Pulsar contract events and delivers HTTP POST notifications to registered merchant webhook URLs.

## Setup

```bash
cd services/webhook
cp .env.example .env    # fill in CONTRACT_ID
npm install
npm run dev             # development
npm run build && npm start  # production
```

## API

### Register a webhook

```http
POST /webhooks
Content-Type: application/json

{
  "merchantAddress": "GXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX",
  "url": "https://your-server.example.com/pulsar-events"
}
```

Response `201`:
```json
{
  "merchantAddress": "GXXX...",
  "url": "https://your-server.example.com/pulsar-events",
  "registeredAt": 1717200000000
}
```

### Get a registration

```http
GET /webhooks/:merchantAddress
```

### Delete a registration

```http
DELETE /webhooks/:merchantAddress
```

---

## Webhook Payload

Every event delivers a `POST` to the registered URL with this JSON body:

```json
{
  "event": "payment_processed",
  "contractId": "CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX",
  "ledger": 1234567,
  "timestamp": 1717200000,
  "data": {
    "id": "0000000000000000-0000000000000000-0",
    "value": { ... }
  }
}
```

### Supported events

| `event` | Trigger |
|---|---|
| `payment_processed` | A payment was successfully processed |
| `refund_initiated` | A refund was initiated |
| `refund_approved` | A refund was approved |
| `refund_rejected` | A refund was rejected |
| `refund_executed` | A refund was executed (funds transferred) |
| `multisig_initiated` | A multisig payment was initiated |
| `multisig_signed` | A signer signed a multisig payment |
| `multisig_executed` | A multisig payment was executed |

---

## Retry Logic

Failed deliveries are retried with exponential backoff:

| Attempt | Delay |
|---|---|
| 1 | immediate |
| 2 | 1 s |
| 3 | 2 s |
| 4 | 4 s |
| 5 | 8 s |

After 5 failed attempts the delivery is dropped and the error is logged.
