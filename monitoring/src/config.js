/**
 * config.js — Runtime configuration for the Pulsar contract monitor.
 *
 * All values can be overridden via environment variables so that no secrets
 * are hard-coded in source.
 */

"use strict";

const config = {
  // ── Stellar network ────────────────────────────────────────────────────────

  /** Horizon base URL. Use https://horizon.stellar.org for Mainnet. */
  horizonUrl: process.env.HORIZON_URL || "https://horizon-testnet.stellar.org",

  /** The deployed contract ID to monitor. */
  contractId: process.env.CONTRACT_ID || "",

  // ── Polling ────────────────────────────────────────────────────────────────

  /**
   * How often (ms) to poll Horizon for new contract events.
   * Horizon supports SSE streaming; we use polling as a simpler fallback.
   */
  pollIntervalMs: parseInt(process.env.POLL_INTERVAL_MS || "15000", 10),

  // ── Alert thresholds ───────────────────────────────────────────────────────

  /**
   * Refund rate alert: if (refunds / payments) > this fraction within the
   * rolling window, fire an alert.  Default: 0.20 (20 %).
   */
  refundRateThreshold: parseFloat(process.env.REFUND_RATE_THRESHOLD || "0.20"),

  /**
   * Rolling window (ms) used to compute the refund rate.
   * Default: 3 600 000 ms = 1 hour.
   */
  refundRateWindowMs: parseInt(
    process.env.REFUND_RATE_WINDOW_MS || "3600000",
    10
  ),

  /**
   * Large-payment alert: fire when a single payment exceeds this amount
   * (in the token's smallest unit, e.g. stroops for XLM).
   * Default: 1 000 000 000 (1 000 XLM at 7 decimal places).
   */
  largePaymentThreshold: BigInt(
    process.env.LARGE_PAYMENT_THRESHOLD || "1000000000"
  ),

  /**
   * Error-rate alert: if (failed_txs / total_txs) > this fraction within the
   * rolling window, fire an alert.  Default: 0.05 (5 %).
   */
  errorRateThreshold: parseFloat(process.env.ERROR_RATE_THRESHOLD || "0.05"),

  // ── Alerting ───────────────────────────────────────────────────────────────

  /**
   * Webhook URL to POST alert payloads to (Slack, PagerDuty, etc.).
   * Leave empty to only log alerts to stdout.
   */
  alertWebhookUrl: process.env.ALERT_WEBHOOK_URL || "",
};

// Validate required fields at startup.
if (!config.contractId) {
  console.warn(
    "[config] WARNING: CONTRACT_ID is not set. " +
      "Set the CONTRACT_ID environment variable to the deployed contract address."
  );
}

module.exports = config;
