/**
 * Pulsar Analytics — Bootstrap
 * Issue #269
 *
 * Import this file once at your application entry point.
 * It wires up adapters and re-exports the public tracking API.
 *
 * Replace 'G-XXXXXXXXXX' with your real GA4 Measurement ID.
 */

import { registerAdapter } from "./tracker.js";
import { createGA4Adapter } from "./adapters/ga4.js";
import { consoleAdapter } from "./adapters/console.js";

const IS_PROD = typeof window !== "undefined"
  && !["localhost", "127.0.0.1"].includes(location.hostname);

// Register GA4 in production
if (IS_PROD) {
  registerAdapter(createGA4Adapter("G-XXXXXXXXXX"));
} else {
  // Log to console in dev/staging
  registerAdapter(consoleAdapter);
}

// Re-export everything callers need
export {
  trackPageView,
  trackAction,
  trackTransaction,
  trackError,
  setUserProperties,
  isOptedOut,
  optOut,
  optIn,
} from "./tracker.js";
