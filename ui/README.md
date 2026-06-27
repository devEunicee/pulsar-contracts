# Pulsar UI — Component Library

Storybook-documented React component library for the Pulsar payment platform.

## Getting started

```bash
cd ui
npm install
npm run storybook        # dev server at http://localhost:6006
npm run build-storybook  # static build → storybook-static/
npm run test-storybook   # visual regression tests
```

## Components

| Component | Story path | Description |
|---|---|---|
| `Button` | `Components/Button` | CTA buttons with variant, size, loading, disabled states |
| `Badge` | `Components/Badge` | Status badges for payments and refunds |
| `PaymentCard` | `Components/PaymentCard` | Payment summary card with status and details action |

## Adding a new component

1. Create `src/components/<Name>/<Name>.tsx`
2. Create `src/components/<Name>/<Name>.stories.tsx` with `autodocs` tag
3. Export from `src/index.ts`

Stories are auto-discovered by the glob `src/**/*.stories.@(ts|tsx)`.
Props documentation is auto-generated from TypeScript types via `autodocs`.
Interactive controls are enabled for all props via `@storybook/addon-essentials`.
