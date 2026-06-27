# Analytics Tracking — Pulsar

> Issue #269 | Priority: Medium | Effort: Medium

Privacy-first event tracking for understanding usage patterns.

---

## Quick start

```js
// app.js (entry point)
import { trackPageView, trackAction, trackTransaction } from './frontend/analytics/index.js';

// Track the initial page view
trackPageView();
```

---

## File layout

```
frontend/analytics/
├── index.js           Bootstrap: wires adapters, re-exports API
├── tracker.js         Core: track(), stripPii(), opt-out logic
└── adapters/
    ├── ga4.js         Google Analytics 4 adapter
    └── console.js     Development / debug adapter
```

---

## API

### `trackPageView(pageName?)`

Call on every route change. Defaults to `location.pathname`.

```js
trackPageView('/payments');
```

### `trackAction(action, properties?)`

User interactions: filters applied, forms submitted, buttons clicked, tabs changed.

```js
trackAction('filter_applied', { filter_type: 'date_range' });
trackAction('form_submitted', { form: 'register_merchant' });
trackAction('tab_changed', { tab: 'payment_history' });
```

### `trackTransaction(action, properties?)`

Payment and refund lifecycle events.

```js
trackTransaction('payment_initiated', { amount: 1000, token: 'USDC' });
trackTransaction('refund_requested', { amount: 500 });
trackTransaction('multisig_signed');
```

### `trackError(errorType, properties?)`

Non-fatal UI errors (invalid input, failed fetch, etc.).

```js
trackError('signature_verification_failed');
trackError('network_error', { endpoint: '/api/payments' });
```

### `setUserProperties(properties)`

Set non-identifying attributes that persist across events.
**No PII** — do not pass names, emails, wallet addresses, or keys.

```js
setUserProperties({ role: 'merchant', merchant_type: 'Retail' });
```

### Opt-out

```js
import { optOut, optIn, isOptedOut } from './frontend/analytics/tracker.js';

optOut();       // respects user's privacy choice
optIn();        // re-enables tracking
isOptedOut();   // true/false
```

DNT (Do Not Track) header is also respected automatically.

---

## Adding a custom adapter

```js
import { registerAdapter } from './frontend/analytics/tracker.js';

registerAdapter({
  track({ category, action, properties }) {
    // send to your analytics service
    myService.event(action, { ...properties, category });
  },
  identify(properties) {
    myService.setUser(properties);
  },
});
```

---

## Privacy guarantees

- PII fields (`email`, `name`, `address`, `wallet`, `public_key`, `private_key`, `contact`, `phone`, `ip`, `user_id`, `merchant_id`) are stripped before any event is dispatched.
- DNT header is honoured.
- Explicit opt-out stored in `localStorage`.
- GA4 is configured with `anonymize_ip: true` and `send_page_view: false` (page views are tracked explicitly with `trackPageView()`).
- No user IDs are collected or sent.

---

## Event reference

| Category | Action | Properties |
|---|---|---|
| `page_view` | `page_viewed` | `page` |
| `user_action` | `filter_applied` | `filter_type` |
| `user_action` | `form_submitted` | `form` |
| `user_action` | `tab_changed` | `tab` |
| `transaction` | `payment_initiated` | `amount`, `token` |
| `transaction` | `refund_requested` | `amount` |
| `transaction` | `multisig_signed` | — |
| `error` | `signature_verification_failed` | — |
| `error` | `network_error` | `endpoint` |
