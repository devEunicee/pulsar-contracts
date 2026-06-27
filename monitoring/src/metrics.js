/**
 * metrics.js — In-memory metric store for the Pulsar contract monitor.
 *
 * Tracks:
 *   - payment_volume   : total number of payment_processed events seen
 *   - refund_rate      : refunds / payments within a rolling time window
 *   - error_rate       : failed transactions / total transactions
 *
 * All timestamps are Unix milliseconds (Date.now()).
 */

"use strict";

class Metrics {
  constructor() {
    /** @type {{ ts: number, amount: bigint }[]} */
    this._payments = [];

    /** @type {{ ts: number }[]} */
    this._refunds = [];

    /** @type {{ ts: number, success: boolean }[]} */
    this._transactions = [];

    /** Cumulative totals (never reset). */
    this.totalPayments = 0n;
    this.totalPaymentVolume = 0n;
    this.totalRefunds = 0n;
    this.totalErrors = 0n;
  }

  // ── Record events ──────────────────────────────────────────────────────────

  /**
   * Record a successful payment.
   * @param {bigint} amount  Payment amount in token's smallest unit.
   */
  recordPayment(amount) {
    const ts = Date.now();
    this._payments.push({ ts, amount });
    this.totalPayments += 1n;
    this.totalPaymentVolume += amount;
  }

  /** Record a refund initiation. */
  recordRefund() {
    this._refunds.push({ ts: Date.now() });
    this.totalRefunds += 1n;
  }

  /**
   * Record a transaction outcome.
   * @param {boolean} success  true = succeeded, false = failed/errored.
   */
  recordTransaction(success) {
    this._transactions.push({ ts: Date.now(), success });
    if (!success) this.totalErrors += 1n;
  }

  // ── Windowed computations ──────────────────────────────────────────────────

  /**
   * Prune entries older than `windowMs` from all rolling buffers.
   * @param {number} windowMs
   */
  _prune(windowMs) {
    const cutoff = Date.now() - windowMs;
    this._payments = this._payments.filter((e) => e.ts >= cutoff);
    this._refunds = this._refunds.filter((e) => e.ts >= cutoff);
    this._transactions = this._transactions.filter((e) => e.ts >= cutoff);
  }

  /**
   * Compute refund rate within the rolling window.
   * @param {number} windowMs
   * @returns {number}  Value in [0, 1].  Returns 0 when no payments in window.
   */
  refundRate(windowMs) {
    this._prune(windowMs);
    const payments = this._payments.length;
    if (payments === 0) return 0;
    return this._refunds.length / payments;
  }

  /**
   * Compute error rate within the rolling window.
   * @param {number} windowMs
   * @returns {number}  Value in [0, 1].  Returns 0 when no transactions in window.
   */
  errorRate(windowMs) {
    this._prune(windowMs);
    const total = this._transactions.length;
    if (total === 0) return 0;
    const errors = this._transactions.filter((t) => !t.success).length;
    return errors / total;
  }

  /**
   * Payment volume (count) within the rolling window.
   * @param {number} windowMs
   * @returns {number}
   */
  paymentVolume(windowMs) {
    this._prune(windowMs);
    return this._payments.length;
  }

  /** Return a plain-object snapshot of all current metrics. */
  snapshot(windowMs) {
    return {
      window_ms: windowMs,
      payment_volume_in_window: this.paymentVolume(windowMs),
      refund_rate_in_window: this.refundRate(windowMs),
      error_rate_in_window: this.errorRate(windowMs),
      total_payments: this.totalPayments.toString(),
      total_payment_volume: this.totalPaymentVolume.toString(),
      total_refunds: this.totalRefunds.toString(),
      total_errors: this.totalErrors.toString(),
    };
  }
}

module.exports = { Metrics };
