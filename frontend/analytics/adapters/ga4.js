/**
 * Google Analytics 4 adapter for Pulsar Analytics — Issue #269
 *
 * Usage:
 *   import { createGA4Adapter } from './adapters/ga4.js';
 *   import { registerAdapter } from '../tracker.js';
 *   registerAdapter(createGA4Adapter('G-XXXXXXXXXX'));
 *
 * Injects the gtag.js script on first call. Safe to import in SSR
 * environments — the script injection guard checks for `window`.
 */

let scriptInjected = false;

/**
 * Inject the GA4 gtag.js script tag once.
 * @param {string} measurementId
 */
function injectScript(measurementId) {
  if (scriptInjected || typeof document === "undefined") return;
  scriptInjected = true;

  const script = document.createElement("script");
  script.async = true;
  script.src = `https://www.googletagmanager.com/gtag/js?id=${measurementId}`;
  document.head.appendChild(script);

  window.dataLayer = window.dataLayer || [];
  window.gtag = function gtag() {
    window.dataLayer.push(arguments);
  };
  window.gtag("js", new Date());
  window.gtag("config", measurementId, {
    // Disable default PII collection
    send_page_view: false,
    anonymize_ip: true,
  });
}

/**
 * Create a GA4 analytics adapter.
 * @param {string} measurementId  GA4 measurement ID (e.g. "G-XXXXXXXXXX")
 * @returns {import('../tracker.js').AnalyticsAdapter}
 */
function createGA4Adapter(measurementId) {
  if (typeof window !== "undefined") {
    injectScript(measurementId);
  }

  return {
    track({ category, action, properties }) {
      if (typeof window === "undefined" || typeof window.gtag !== "function") return;
      window.gtag("event", action, {
        event_category: category,
        ...properties,
      });
    },

    identify(properties) {
      if (typeof window === "undefined" || typeof window.gtag !== "function") return;
      // GA4 does not have a native identify — set user properties instead.
      window.gtag("set", "user_properties", properties);
    },
  };
}

export { createGA4Adapter };
