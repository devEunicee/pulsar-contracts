# Pulsar REST API

REST API wrapper for the Pulsar payment-processing smart contract on Stellar Soroban.

## Setup

```bash
cd api
npm install
```

## Configuration

Set environment variables:

```bash
export CONTRACT_ID=<deployed_contract_id>
export STELLAR_NETWORK=testnet   # or mainnet
export STELLAR_RPC_URL=https://soroban-testnet.stellar.org
export SOURCE_SECRET_KEY=<secret_key_with_funds>
```

## Run

```bash
npm start
```

Development mode (auto-restart):

```bash
npm run dev
```

## Endpoints

### Merchants

**POST /api/merchants**

Register a merchant.

Request body:
```json
{
  "merchant_address": "G...",
  "name": "Store Name",
  "description": "Store description",
  "contact_info": "contact@store.com",
  "category": "Retail"
}
```

**GET /api/merchants/:id**

Retrieve merchant information.

### Payments

**POST /api/payments**

Process a payment with signature verification.

Request body:
```json
{
  "payer": "G...",
  "order": {
    "order_id": "ORDER_001",
    "merchant_address": "G...",
    "payer": "G...",
    "token": "C...",
    "amount": "1000",
    "description": "Purchase",
    "expires_at": "0"
  },
  "signature": "hex_encoded_signature"
}
```

**GET /api/payments/:id?caller=G...**

Retrieve a payment record by order ID.

Query parameters:
- `caller` (required): Address requesting the record

**GET /api/payments?merchant=G...**

List merchant payment history with filtering/pagination.

Query parameters:
- `merchant` (required): Merchant address
- `cursor` (optional): Pagination cursor
- `limit` (optional): Max results (default 10, max 100)
- `date_start`, `date_end` (optional): Unix timestamp range
- `amount_min`, `amount_max` (optional): Amount range
- `status` (optional): "Any", "Completed", "PartiallyRefunded", "FullyRefunded"
- `sort_field` (optional): "Date" or "Amount"
- `sort_order` (optional): "Ascending" or "Descending"

## Error Responses

All errors follow the format:

```json
{
  "error": {
    "code": "ErrorCode",
    "message": "Detailed error message"
  }
}
```

HTTP status codes map to contract errors:
- 403: Unauthorized
- 404: Resource not found (Merchant/Payment/Refund/Multisig)
- 409: Resource conflict (AlreadyExists, AlreadyCompleted)
- 422: Invalid input or state (InvalidAmount, PaymentExpired, RefundWindowExpired, etc.)
- 500: Internal server error
