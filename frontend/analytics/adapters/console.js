/**
 * Console (debug) adapter for Pulsar Analytics — Issue #269
 *
 * Use during local development to inspect events without sending real data.
 *
 * Usage:
 *   import { consoleAdapter } from './adapters/console.js';
 *   import { registerAdapter } from '../tracker.js';
 *   if (process.env.NODE_ENV !== 'production') {
 *     registerAdapter(consoleAdapter);
 *   }
 */

/** @type {import('../tracker.js').AnalyticsAdapter} */
const consoleAdapter = {
  track({ category, action, properties }) {
    console.debug("[Pulsar Analytics]", category, action, properties);
  },
  identify(properties) {
    console.debug("[Pulsar Analytics] identify", properties);
  },
};

export { consoleAdapter };
