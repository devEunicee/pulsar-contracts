# Design Tokens — Pulsar Theme/Branding System

> Issue #270 | Priority: Low | Effort: Large

All design tokens are defined as [CSS custom properties](https://developer.mozilla.org/en-US/docs/Web/CSS/Using_CSS_custom_properties) in `frontend/theme/tokens.css` and can be consumed by any CSS file or CSS-in-JS solution.

---

## Usage

```html
<!-- 1. Import the token file once in your HTML or root CSS -->
<link rel="stylesheet" href="/frontend/theme/tokens.css" />

<!-- 2. Optionally bootstrap the theme before first paint to avoid FOUC -->
<script type="module">
  import { initTheme } from '/frontend/theme/theme-switcher.js';
  initTheme();
</script>
```

```css
/* 3. Reference tokens anywhere in your styles */
.button-primary {
  background: var(--color-action-primary);
  color:      var(--color-action-primary-text);
  padding:    var(--spacing-2) var(--spacing-4);
  border-radius: var(--radius-md);
  font-size:  var(--font-size-sm);
  font-weight: var(--font-weight-semibold);
  box-shadow: var(--shadow-sm);
  transition: background var(--transition-fast);
}
.button-primary:hover {
  background: var(--color-action-primary-hover);
}
```

---

## Token Categories

### Colors

| Token | Light value | Description |
|---|---|---|
| `--color-brand-500` | `#2e4fe0` | Primary brand / CTA |
| `--color-bg-primary` | `#ffffff` | Page background |
| `--color-bg-secondary` | `#f8f9fb` | Card / sidebar background |
| `--color-text-primary` | `#0d1117` | Default body text |
| `--color-text-secondary` | `#4a5261` | De-emphasised text |
| `--color-border-default` | `#dde1e8` | Default border |
| `--color-success` | `#10b981` | Success state |
| `--color-warning` | `#f59e0b` | Warning state |
| `--color-error` | `#ef4444` | Error state |
| `--color-info` | `#3b82f6` | Informational state |

Full palette: `--color-brand-{50–900}`, `--color-neutral-{0–900}`, plus `-light` / `-dark` variants for each semantic colour.

### Typography

| Token | Value |
|---|---|
| `--text-h1-size` | `3rem` (48px) |
| `--text-h2-size` | `2.25rem` (36px) |
| `--text-h3-size` | `1.875rem` (30px) |
| `--text-h4-size` | `1.5rem` (24px) |
| `--text-h5-size` | `1.25rem` (20px) |
| `--text-h6-size` | `1.125rem` (18px) |
| `--text-body-size` | `1rem` (16px) |
| `--text-caption-size` | `0.875rem` (14px) |
| `--font-family-sans` | Inter, system-ui, … |
| `--font-family-mono` | JetBrains Mono, … |

### Spacing

8-point grid: `--spacing-{1–24}` maps to `4 px – 96 px`.

### Border Radius

`--radius-{none|sm|base|md|lg|xl|2xl|3xl|full}` — `0` to `9999px`.

### Shadows

`--shadow-{xs|sm|md|lg|xl|2xl|inner|none}`

### Z-index

`--z-{base|raised|dropdown|sticky|overlay|modal|toast|tooltip}`

---

## Theme Switching

```js
import { applyTheme, toggleTheme, initTheme } from '/frontend/theme/theme-switcher.js';

// Apply on load (reads localStorage + OS preference):
initTheme();

// Toggle light ↔ dark:
toggleTheme();

// Explicit set:
applyTheme('dark');
```

The switcher writes `data-theme="dark"` on `<html>`, which triggers the `[data-theme="dark"]` overrides in `tokens.css`.

---

## Adding a Custom Theme

```css
[data-theme="high-contrast"] {
  --color-bg-primary:   #000000;
  --color-text-primary: #ffffff;
  --color-action-primary: #ffff00;
  /* override only the tokens you need */
}
```
