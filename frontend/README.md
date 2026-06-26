# Pulsar Frontend — Payer Dashboard

React + Vite UI for payers to view payment history and track refund status.

## Setup

```bash
cd frontend
cp .env.example .env        # fill in CONTRACT_ID
npm install
npm run dev
```

## Features
- Freighter wallet connection
- Paginated payment history table (cursor-based)
- Filter by status, amount range
- Sort by date or amount (ascending/descending)
- Refund status displayed inline per payment
