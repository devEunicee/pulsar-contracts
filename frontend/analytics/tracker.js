/**
 * Pulsar Analytics — Issue #269
 *
 * Privacy-first event tracking that:
 *  - Tracks page views, user actions, and transaction events
 *  - Strips all PII before sending (no names, emails, addresses, keys)
 *  - Supports Google Analytics 4 (gtag) and a generic adapter interface
 *  - Gracefully no-ops when the user has opted out or analytics is absent
 */

// ── Types ─────────────────────────────────────────────────────────────────────

/**
 * @typedef {"page_view"|"user_action"|"transaction"|"error"} EventCategory
 * @typedef {Record<string, string|number|boolean>} EventProperties
 */

/**
 * @typedef {Object} AnalyticsEvent
 * @property {EventCategory} category
 * @property {string} action
 * @property {EventProperties} [properties]
 */

/**
 * @typedef {Object} AnalyticsAdapter
 * @property {(event: AnalyticsEvent) => void} track
 * @property {(properties: EventProperties) => void} [identify]
 */

// ── PII fields to strip from any event payload ────────────────────────────────

const PII_FIELDS = [
  "email", "name", "address", "wallet", "public_key", "private_key",
  "contact", "phone", "ip", "user_id", "merchant_id",
];

/**
 * Remove PII keys from a properties object.
 * @param {EventProperties} props
 * @returns {EventProperties}
 */
function stripPii(props) {
  if (!props) return {};
  return Object.fromEntries(
    Object.entries(props).filter(
      ([k]) => !PII_FIELDS.some((pii) => k.toLowerCase().includes(pii))
    )
  );
}

// ── Opt-out / consent ─────────────────────────────────────────────────────────

const OPT_OUT_KEY = "pulsar-analytics-opt-out";

/** Return true if the user has opted out of analytics. */
function isOptedOut() {
  try {
    return (
      localStorage.getItem(OPT_OUT_KEY) === "true" ||
      navigator.doNotTrack === "1" ||
      window.doNotTrack === "1"
    );
  } catch {
    return false;
  }
}

/** Opt the current user out of analytics tracking. */
function optOut() {
  try {
    localStorage.setItem(OPT_OUT_KEY, "true");
  } catch { /* storage unavailable */ }
}

/** Opt the current user back into analytics tracking. */
function optIn() {
  try {
    localStorage.removeItem(OPT_OUT_KEY);
  } catch { /* storage unavailable */ }
}

// ── Adapter registry ──────────────────────────────────────────────────────────

/** @type {AnalyticsAdapter[]} */
const adapters = [];

/**
 * Register an analytics adapter (GA4, Mixpanel, custom, etc.).
 * @param {AnalyticsAdapter} adapter
 */
function registerAdapter(adapter) {
  adapters.push(adapter);
}

// ── Core track function ───────────────────────────────────────────────────────

/**
 * Dispatch an analytics event to all registered adapters.
 * No-ops when opted out or no adapters are registered.
 *
 * @param {EventCategory} category
 * @param {string} action
 * @param {EventProperties} [properties]
 */
function track(category, action, properties) {
  if (isOptedOut() || adapters.length === 0) return;

  const safeProps = stripPii(properties || {});
  const event = { category, action, properties: safeProps };

  for (const adapter of adapters) {
    try {
      adapter.track(event);
    } catch { /* never let analytics errors break the app */ }
  }
}

// ── High-level helpers ────────────────────────────────────────────────────────

/** Track a page view. Automatically captures the current pathname. */
function trackPageView(pageName) {
  track("page_view", "page_viewed", {
    page: pageName || (typeof location !== "undefined" ? location.pathname : "unknown"),
  });
}

/**
 * Track a user action (button click, filter applied, form submitted, etc.).
 * @param {string} action  e.g. "filter_applied", "form_submitted"
 * @param {EventProperties} [properties]
 */
function trackAction(action, properties) {
  track("user_action", action, properties);
}

/**
 * Track a transaction or payment event. Amounts are kept; PII is stripped.
 * @param {string} action  e.g. "payment_initiated", "refund_requested"
 * @param {{ amount?: number, token?: string, status?: string } & EventProperties} [properties]
 */
function trackTransaction(action, properties) {
  track("transaction", action, properties);
}

/** Track a non-fatal UI error. */
function trackError(errorType, properties) {
  track("error", errorType, properties);
}

// ── Set non-identifying user properties ──────────────────────────────────────

/**
 * Set persistent user properties (role, merchant_type) — no PII.
 * @param {{ role?: string, merchant_type?: string } & EventProperties} properties
 */
function setUserProperties(properties) {
  if (isOptedOut()) return;
  const safeProps = stripPii(properties);
  for (const adapter of adapters) {
    try {
      adapter.identify?.(safeProps);
    } catch { /* ignore */ }
  }
}

export {
  registerAdapter,
  track,
  trackPageView,
  trackAction,
  trackTransaction,
  trackError,
  setUserProperties,
  isOptedOut,
  optOut,
  optIn,
};
