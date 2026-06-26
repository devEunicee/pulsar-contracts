# Pulsar Merchant Dashboard

React web app for merchants to view payment history, manage refunds, and view their profile. Connects to Stellar Freighter wallet.

## Features

- Payment history table (sourced from the event indexer)
- Refund initiation form
- Merchant profile view
- Freighter wallet connection
- Responsive design (mobile + desktop)
- WCAG 2.1 AA accessible (semantic HTML, ARIA roles, focus styles, live regions)

## Setup

```bash
cd dashboard
cp .env.example .env   # set REACT_APP_INDEXER_URL
npm install
npm start
```

## Environment

| Variable                  | Default                  | Description              |
|---------------------------|--------------------------|--------------------------|
| `REACT_APP_INDEXER_URL`   | `http://localhost:3001`  | Pulsar event indexer URL |

## Wallet Support

- [Freighter](https://www.freighter.app/) (primary)
- Albedo can be added via `@albedo-link/intent` following the same pattern in `useWallet.js`
