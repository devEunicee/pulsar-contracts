# Pulsar Analytics Aggregation Service

A lightweight analytics service for Pulsar payment and refund data.

## Endpoints

- `GET /analytics/summary`
  - Returns total payment volume, total payment count, total refund volume, refund count, refund rate, average transaction value, and top merchants by volume.

- `GET /analytics/trends?startDate=YYYY-MM-DD&endDate=YYYY-MM-DD`
  - Returns day-over-day payment volume and count for the given date range.

- `GET /analytics/segments?startDate=YYYY-MM-DD&endDate=YYYY-MM-DD`
  - Returns customer segmentation by total spend and payment count.

- `GET /analytics/custom-report?metric=payment_count|refund_rate|payments|refunds&merchant_id=...&startDate=...&endDate=...`
  - Generates a custom report for payments or refunds.

## Setup

```bash
cd services/analytics-service
npm install
cp .env.example .env
```

Set `DATABASE_URL` to the analytics database connection string.

## Run

```bash
npm run dev
```

## Notes

This service expects a relational analytics database with `payments` and `refunds` tables. See `schema.sql` for the minimal schema.
