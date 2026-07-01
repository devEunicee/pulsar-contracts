/**
 * dashboard-server.js — HTTP server that exposes database and contract metrics
 * for the Pulsar monitoring dashboard (#307).
 *
 * Endpoints:
 *   GET /health                  — Liveness check.
 *   GET /metrics/db              — Latest database metrics snapshot (JSON).
 *   GET /metrics/contract        — Latest contract event metrics snapshot (JSON).
 *   GET /metrics/alerts          — Recent alert history (JSON).
 *
 * Usage:
 *   DASHBOARD_PORT=4000 DATABASE_URL=postgres://... node src/dashboard-server.js
 *
 * Environment variables:
 *   DASHBOARD_PORT   HTTP port for the dashboard server. Default: 4000.
 *   DATABASE_URL     PostgreSQL connection string.
 *   DB_METRICS_POLL_MS  How often to refresh DB metrics (ms). Default: 30000.
 */

"use strict";

const http = require("http");
const { DbMetrics } = require("./db-metrics");
const config = require("./config");
const { Metrics } = require("./metrics");

const PORT = parseInt(process.env.DASHBOARD_PORT ?? "4000", 10);
const DB_POLL_MS = parseInt(process.env.DB_METRICS_POLL_MS ?? "30000", 10);

// ── State ─────────────────────────────────────────────────────────────────────

const dbMetrics = new DbMetrics();
const contractMetrics = new Metrics();

/** Latest cached DB metrics snapshot. */
let latestDbSnapshot = null;

/** Ring buffer of the last 100 alert events. */
const alertHistory = [];

// ── DB polling ────────────────────────────────────────────────────────────────

async function refreshDbMetrics() {
  try {
    latestDbSnapshot = await dbMetrics.collect();
    checkDbAnomalies(latestDbSnapshot);
  } catch (err) {
    console.error("[dashboard-server] DB metrics refresh failed:", err.message);
  }
}

/**
 * Inspect a freshly-collected DB snapshot and push alerts for anomalies.
 *
 * @param {import("./db-metrics").DbMetricsSnapshot} snapshot
 */
function checkDbAnomalies(snapshot) {
  if (!snapshot || snapshot.error) return;

  // Alert on long-running transactions.
  for (const txn of snapshot.long_running_transactions) {
    const dur = parseFloat(txn.duration_seconds);
    pushAlert("long_running_transaction", {
      pid: txn.pid,
      duration_seconds: dur,
      state: txn.state,
      last_query: txn.last_query,
    });
  }

  // Alert on many idle-in-transaction connections.
  const idleTxnCount = snapshot.connections?.idle_in_transaction ?? 0;
  if (idleTxnCount >= 5) {
    pushAlert("high_idle_in_transaction", {
      count: idleTxnCount,
      threshold: 5,
    });
  }

  // Alert on replication lag > 10 MB.
  for (const replica of snapshot.replication) {
    const lagBytes = parseInt(replica.replay_lag_bytes ?? "0", 10);
    if (lagBytes > 10 * 1024 * 1024) {
      pushAlert("replication_lag", {
        application_name: replica.application_name,
        lag_bytes: lagBytes,
        lag_mb: (lagBytes / 1024 / 1024).toFixed(2),
      });
    }
  }
}

/**
 * Push an alert entry into the ring buffer.
 * @param {string} type
 * @param {object} data
 */
function pushAlert(type, data) {
  alertHistory.push({ type, data, timestamp: new Date().toISOString() });
  if (alertHistory.length > 100) alertHistory.shift();
  console.log(`[dashboard-server] ALERT ${type}:`, JSON.stringify(data));
}

// ── HTTP server ───────────────────────────────────────────────────────────────

function jsonResponse(res, statusCode, body) {
  const payload = JSON.stringify(body, null, 2);
  res.writeHead(statusCode, {
    "Content-Type": "application/json",
    "Content-Length": Buffer.byteLength(payload),
    "Cache-Control": "no-store",
  });
  res.end(payload);
}

const server = http.createServer(async (req, res) => {
  const url = new URL(req.url, `http://localhost:${PORT}`);

  if (req.method !== "GET") {
    return jsonResponse(res, 405, { error: "Method Not Allowed" });
  }

  switch (url.pathname) {
    case "/health":
      return jsonResponse(res, 200, {
        status: "ok",
        uptime_seconds: Math.floor(process.uptime()),
      });

    case "/metrics/db":
      // Return the cached snapshot; if not yet available, collect on-demand.
      if (!latestDbSnapshot) {
        latestDbSnapshot = await dbMetrics.collect();
      }
      return jsonResponse(res, 200, latestDbSnapshot);

    case "/metrics/contract":
      return jsonResponse(
        res,
        200,
        contractMetrics.snapshot(config.refundRateWindowMs)
      );

    case "/metrics/alerts":
      return jsonResponse(res, 200, {
        count: alertHistory.length,
        alerts: alertHistory.slice().reverse(), // newest first
      });

    default:
      return jsonResponse(res, 404, { error: "Not Found" });
  }
});

// ── Bootstrap ─────────────────────────────────────────────────────────────────

async function main() {
  // Initial DB metrics collection before starting to serve.
  await refreshDbMetrics();

  // Schedule periodic DB refresh.
  setInterval(refreshDbMetrics, DB_POLL_MS);

  server.listen(PORT, () => {
    console.log(
      `[dashboard-server] Pulsar DB monitoring dashboard running on port ${PORT}`
    );
    console.log(
      `[dashboard-server] DB metrics refreshed every ${DB_POLL_MS / 1000}s`
    );
  });
}

main().catch((err) => {
  console.error("[dashboard-server] Fatal error:", err);
  process.exit(1);
});

// Graceful shutdown.
process.on("SIGTERM", async () => {
  console.log("[dashboard-server] Shutting down…");
  await dbMetrics.close();
  server.close(() => process.exit(0));
});
