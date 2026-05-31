/**
 * monitor.js — Stellar Horizon event stream listener for the Pulsar
 * payment-processing contract.
 *
 * Connects to Horizon, streams contract events, updates in-memory metrics,
 * and fires alerts when anomaly thresholds are breached.
 *
 * Usage:
 *   CONTRACT_ID=<contract-id> node src/monitor.js
 *
 * See README.md in this directory for full configuration options.
 */

"use strict";

const { Horizon } = require("@stellar/stellar-sdk");
const config = require("./config");
const { Metrics } = require("./metrics");
const { alert } = require("./alerts");

// ── Horizon client ─────────────────────────────────────────────────────────

const server = new Horizon.Server(config.horizonUrl);

// ── Metric store ───────────────────────────────────────────────────────────

const metrics = new Metrics();

// ── Cursor tracking ────────────────────────────────────────────────────────

/** Horizon paging token for the last processed event. */
let cursor = "now";

// ── Event handlers ─────────────────────────────────────────────────────────

/**
 * Dispatch a single contract event to the appropriate handler.
 *
 * Pulsar emits the following event topics (first element of the topics array):
 *   - "payment_processed"
 *   - "refund_initiated"
 *   - "refund_approved"
 *   - "refund_executed"
 *   - "refund_rejected"
 *   - "multisig_initiated"
 *   - "multisig_signed"
 *   - "multisig_executed"
 *   - "merchant_registered"
 *   - "merchant_deactivated"
 *   - "admin_set"
 *
 * @param {object} event  Raw Horizon contract event record.
 */
async function handleEvent(event) {
  const topic = event.topic?.[0]?.value ?? "";

  // Record every event as a transaction attempt.
  metrics.recordTransaction(true);

  switch (topic) {
    case "payment_processed": {
      // event.value is (order_id, payer, merchant, amount)
      const amount = parseAmount(event.value);
      metrics.recordPayment(amount);

      // Large-payment alert.
      if (amount > config.largePaymentThreshold) {
        await alert(
          "large_payment",
          `Large payment detected: ${amount.toString()} units`,
          { amount: amount.toString(), event_id: event.id }
        );
      }
      break;
    }

    case "refund_initiated": {
      metrics.recordRefund();
      break;
    }

    default:
      // No special handling needed for other event types.
      break;
  }

  // Update cursor so the next poll starts after this event.
  cursor = event.paging_token;
}

/**
 * Attempt to extract a payment amount from a Horizon event value.
 * The value is a Soroban SCVal tuple; we look for the last i128 element.
 *
 * @param {object} value  Horizon event value object.
 * @returns {bigint}
 */
function parseAmount(value) {
  try {
    // Horizon returns SCVal as JSON; the amount is the last element of the tuple.
    if (Array.isArray(value)) {
      for (let i = value.length - 1; i >= 0; i--) {
        const v = value[i];
        if (v?.type === "i128" || v?.type === "i64") {
          return BigInt(v.value ?? v.lo ?? 0);
        }
      }
    }
  } catch (_) {
    // Ignore parse errors; return 0 so we don't crash the monitor.
  }
  return 0n;
}

// ── Anomaly checks ─────────────────────────────────────────────────────────

/**
 * Run all threshold checks and fire alerts as needed.
 * Called after each poll cycle.
 */
async function checkAnomalies() {
  const windowMs = config.refundRateWindowMs;

  // 1. Refund rate
  const rr = metrics.refundRate(windowMs);
  if (rr > config.refundRateThreshold) {
    await alert(
      "refund_rate",
      `Refund rate ${(rr * 100).toFixed(1)}% exceeds threshold ` +
        `${(config.refundRateThreshold * 100).toFixed(1)}% in the last ` +
        `${windowMs / 60000} minutes`,
      metrics.snapshot(windowMs)
    );
  }

  // 2. Error rate
  const er = metrics.errorRate(windowMs);
  if (er > config.errorRateThreshold) {
    await alert(
      "error_rate",
      `Error rate ${(er * 100).toFixed(1)}% exceeds threshold ` +
        `${(config.errorRateThreshold * 100).toFixed(1)}% in the last ` +
        `${windowMs / 60000} minutes`,
      metrics.snapshot(windowMs)
    );
  }
}

// ── Polling loop ───────────────────────────────────────────────────────────

/**
 * Fetch a page of contract events from Horizon starting at `cursor`,
 * process each event, then schedule the next poll.
 */
async function poll() {
  if (!config.contractId) {
    console.warn("[monitor] CONTRACT_ID not set — skipping poll.");
    scheduleNextPoll();
    return;
  }

  try {
    const response = await server
      .contractEvents(config.contractId)
      .cursor(cursor)
      .limit(200)
      .call();

    const records = response?.records ?? [];

    for (const event of records) {
      await handleEvent(event);
    }

    if (records.length > 0) {
      console.log(`[monitor] Processed ${records.length} event(s). cursor=${cursor}`);
    }

    await checkAnomalies();
  } catch (err) {
    // Record the failed poll as a transaction error.
    metrics.recordTransaction(false);
    console.error("[monitor] Poll error:", err.message);
  }

  scheduleNextPoll();
}

function scheduleNextPoll() {
  setTimeout(poll, config.pollIntervalMs);
}

// ── Entry point ────────────────────────────────────────────────────────────

async function main() {
  await alert(
    "info",
    `Pulsar contract monitor starting. contract=${config.contractId} ` +
      `horizon=${config.horizonUrl} poll_interval=${config.pollIntervalMs}ms`
  );

  // Log initial metric snapshot.
  console.log("[monitor] Initial metrics:", metrics.snapshot(config.refundRateWindowMs));

  poll();
}

main().catch((err) => {
  console.error("[monitor] Fatal error:", err);
  process.exit(1);
});
