# Off-Chain API — Pagination Guide

> **Issue BE-008** — Off-chain API wrappers must expose the contract's
> cursor-based pagination to API consumers.

---

## Overview

The Pulsar payment-processing contract returns paginated results via a
`PaymentPage` response that contains:

| Field | Type | Description |
|---|---|---|
| `records` | array | Payment records for this page |
| `next_cursor` | string \| null | Opaque cursor to fetch the next page; `null` when no more pages |
| `total` | number | Total matching records across **all** pages |

An off-chain API layer (REST, GraphQL, gRPC, etc.) **must** surface these
fields directly to consumers rather than hiding them behind a different
abstraction.

---

## Query Parameters

Every paginated endpoint must accept the following query parameters:

| Parameter | Type | Default | Description |
|---|---|---|---|
| `cursor` | string | *(absent)* | Opaque cursor returned by a previous response. Omit on the first request. |
| `limit` | integer | `10` | Number of records per page. Capped at `100` by the contract. |

### Optional filter parameters

| Parameter | Type | Description |
|---|---|---|
| `date_start` | Unix timestamp (u64) | Lower bound on `paid_at` |
| `date_end` | Unix timestamp (u64) | Upper bound on `paid_at` |
| `amount_min` | integer (i128) | Minimum payment amount |
| `amount_max` | integer (i128) | Maximum payment amount |
| `token` | address string | Filter by token contract address |
| `status` | `any` \| `completed` \| `partially_refunded` \| `fully_refunded` | Payment status filter |
| `sort_field` | `date` \| `amount` | Field to sort by (default: `date`) |
| `sort_order` | `asc` \| `desc` | Sort direction (default: `desc`) |

---

## Response Shape

```json
{
  "records": [
    {
      "order_id": "ORDER_001",
      "merchant_address": "G...",
      "payer": "G...",
      "token": "C...",
      "amount": 1000,
      "refunded_amount": 0,
      "pending_refund_amount": 0,
      "status": "completed",
      "paid_at": 1700000000,
      "description": "Coffee"
    }
  ],
  "next_cursor": "T1JERVJFX0FCQw==",
  "total": 42
}
```

- `next_cursor` is `null` when the current page is the last page.
- `total` reflects the count of all records matching the filter, not just
  the current page.

---

## Pagination Examples

### First page (no cursor)

```
GET /api/v1/merchants/{address}/payments?limit=10&sort_field=date&sort_order=desc
```

Response:

```json
{
  "records": [ ... ],
  "next_cursor": "T1JERVJFX0FCQw==",
  "total": 42
}
```

### Subsequent page (pass cursor from previous response)

```
GET /api/v1/merchants/{address}/payments?limit=10&cursor=T1JERVJFX0FCQw==&sort_field=date&sort_order=desc
```

Response:

```json
{
  "records": [ ... ],
  "next_cursor": "T1JERVJFX0ZHSg==",
  "total": 42
}
```

### Last page

```json
{
  "records": [ ... ],
  "next_cursor": null,
  "total": 42
}
```

### With filters

```
GET /api/v1/merchants/{address}/payments?limit=5&amount_min=100&amount_max=5000&status=completed
```

---

## Payer History Endpoint

```
GET /api/v1/payers/{address}/payments?cursor=...&limit=10
```

Same response shape and query parameters as the merchant endpoint.

---

## Implementation Notes for API Authors

1. **Pass `cursor` directly to the contract** — do not attempt to decode or
   re-encode it. The cursor is an opaque base64-encoded order ID.
2. **Enforce the `limit` cap** — reject requests with `limit > 100` at the
   API layer before calling the contract.
3. **Propagate `next_cursor` verbatim** — return it exactly as received from
   the contract. Do not transform it.
4. **`total` is a snapshot** — it reflects the count at query time. Concurrent
   writes may cause slight inconsistencies across pages; this is expected
   behaviour.
5. **Cursor stability** — cursors are stable as long as the underlying payment
   record exists. Archived or cleaned-up payments may invalidate a cursor.

---

## Error Responses

| HTTP Status | Condition |
|---|---|
| `400 Bad Request` | `limit` out of range (≤ 0 or > 100), invalid `cursor` format |
| `401 Unauthorized` | Caller is not the merchant / payer / admin |
| `404 Not Found` | Merchant or payer address not registered |
| `500 Internal Server Error` | Contract invocation failed |
