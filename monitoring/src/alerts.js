/**
 * alerts.js — Alert dispatcher for the Pulsar contract monitor.
 *
 * Supported channels:
 *   - stdout (always)
 *   - Webhook (Slack / PagerDuty / generic HTTP POST) when ALERT_WEBHOOK_URL
 *     is configured.
 */

"use strict";

const config = require("./config");

/**
 * Fire an alert.
 *
 * @param {"refund_rate" | "large_payment" | "error_rate" | "info"} type
 * @param {string} message  Human-readable description.
 * @param {object} [data]   Optional structured payload attached to the alert.
 */
async function alert(type, message, data = {}) {
  const payload = {
    source: "pulsar-contract-monitor",
    contract_id: config.contractId,
    alert_type: type,
    message,
    data,
    timestamp: new Date().toISOString(),
  };

  // Always log to stdout.
  const prefix = type === "info" ? "[INFO]" : "[ALERT]";
  console.log(`${prefix} [${type}] ${message}`, JSON.stringify(data));

  // Optionally POST to a webhook.
  if (config.alertWebhookUrl) {
    try {
      const { default: fetch } = await import("node-fetch");
      const res = await fetch(config.alertWebhookUrl, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(payload),
      });
      if (!res.ok) {
        console.error(
          `[alerts] Webhook POST failed: ${res.status} ${res.statusText}`
        );
      }
    } catch (err) {
      console.error("[alerts] Failed to send webhook alert:", err.message);
    }
  }
}

module.exports = { alert };
