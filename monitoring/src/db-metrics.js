/**
 * db-metrics.js — Database metrics collector for the Pulsar monitoring service.
 *
 * Resolves #307: database monitoring dashboard.
 *
 * Collects the following metrics from PostgreSQL by querying the built-in
 * system catalog views (pg_stat_activity, pg_stat_database, pg_stat_bgwriter,
 * pg_stat_replication, pg_locks):
 *
 *   - Real-time connection count (active / idle / idle-in-transaction / waiting)
 *   - Query performance (calls, mean exec time, total exec time) per statement
 *   - Replication lag (bytes behind primary per standby)
 *   - Disk / relation usage (table sizes)
 *   - Slow query identification (queries running > threshold)
 *   - Long-running transaction detection (transactions open > threshold)
 *   - Cache hit ratio
 *   - Lock contention overview
 *
 * Usage:
 *   const { DbMetrics } = require("./db-metrics");
 *   const dbMetrics = new DbMetrics({ connectionString: process.env.DATABASE_URL });
 *   const snapshot = await dbMetrics.collect();
 *
 * Environment variables (all optional):
 *   DATABASE_URL              - PostgreSQL connection string.
 *   DB_SLOW_QUERY_MS          - Threshold in ms above which a query is "slow". Default: 1000.
 *   DB_LONG_TXN_SECONDS       - Threshold in seconds above which a txn is "long-running". Default: 30.
 *   DB_METRICS_POLL_MS        - How often the server polls db metrics (ms). Default: 30000.
 */

"use strict";

const { Client } = require("pg");

const SLOW_QUERY_MS = parseInt(process.env.DB_SLOW_QUERY_MS ?? "1000", 10);
const LONG_TXN_SECONDS = parseInt(process.env.DB_LONG_TXN_SECONDS ?? "30", 10);

// ── Queries ───────────────────────────────────────────────────────────────────

/** Connection counts grouped by state. */
const SQL_CONNECTIONS = `
SELECT
  state,
  wait_event_type,
  count(*) AS count
FROM pg_stat_activity
WHERE pid <> pg_backend_pid()
GROUP BY state, wait_event_type
ORDER BY state NULLS LAST;
`;

/** Per-database statistics (cache hits, transactions, conflicts). */
const SQL_DB_STATS = `
SELECT
  datname,
  numbackends,
  xact_commit,
  xact_rollback,
  blks_read,
  blks_hit,
  CASE
    WHEN (blks_read + blks_hit) > 0
    THEN round(blks_hit::numeric / (blks_read + blks_hit) * 100, 2)
    ELSE 100
  END AS cache_hit_ratio,
  tup_returned,
  tup_fetched,
  tup_inserted,
  tup_updated,
  tup_deleted,
  temp_bytes,
  deadlocks,
  conflicts,
  stats_reset
FROM pg_stat_database
WHERE datname = current_database();
`;

/** Top-N slowest statements (requires pg_stat_statements). */
const SQL_SLOW_STATEMENTS = `
SELECT
  left(query, 200)                      AS query_preview,
  calls,
  round(mean_exec_time::numeric, 2)     AS mean_exec_ms,
  round(total_exec_time::numeric, 2)    AS total_exec_ms,
  round(stddev_exec_time::numeric, 2)   AS stddev_exec_ms,
  rows
FROM pg_stat_statements
ORDER BY mean_exec_time DESC
LIMIT 10;
`;

/** Currently running queries that exceed the slow-query threshold. */
const SQL_SLOW_RUNNING = `
SELECT
  pid,
  usename,
  application_name,
  client_addr::text,
  state,
  wait_event_type,
  wait_event,
  extract(epoch from (now() - query_start)) * 1000  AS duration_ms,
  left(query, 300)                                   AS query_preview
FROM pg_stat_activity
WHERE state = 'active'
  AND query_start < now() - ($1 || ' milliseconds')::interval
  AND pid <> pg_backend_pid()
ORDER BY duration_ms DESC;
`;

/** Long-running open transactions. */
const SQL_LONG_TRANSACTIONS = `
SELECT
  pid,
  usename,
  application_name,
  state,
  extract(epoch from (now() - xact_start)) AS duration_seconds,
  left(query, 300)                          AS last_query
FROM pg_stat_activity
WHERE xact_start IS NOT NULL
  AND xact_start < now() - ($1 || ' seconds')::interval
  AND pid <> pg_backend_pid()
ORDER BY duration_seconds DESC;
`;

/** Replication lag per standby (primary only). */
const SQL_REPLICATION = `
SELECT
  application_name,
  client_addr::text,
  state,
  sync_state,
  sent_lsn::text,
  write_lsn::text,
  flush_lsn::text,
  replay_lsn::text,
  pg_wal_lsn_diff(sent_lsn, replay_lsn) AS replay_lag_bytes
FROM pg_stat_replication;
`;

/** Table disk usage (top 20 by total size). */
const SQL_TABLE_SIZES = `
SELECT
  schemaname,
  tablename,
  pg_size_pretty(pg_total_relation_size(schemaname || '.' || tablename)) AS total_size,
  pg_total_relation_size(schemaname || '.' || tablename)                  AS total_bytes,
  pg_size_pretty(pg_relation_size(schemaname || '.' || tablename))        AS table_size,
  pg_size_pretty(
    pg_total_relation_size(schemaname || '.' || tablename)
    - pg_relation_size(schemaname || '.' || tablename)
  )                                                                        AS index_size
FROM pg_tables
WHERE schemaname NOT IN ('pg_catalog', 'information_schema')
ORDER BY total_bytes DESC
LIMIT 20;
`;

/** Lock contention overview. */
const SQL_LOCKS = `
SELECT
  locktype,
  relation::regclass::text AS relation,
  mode,
  granted,
  count(*) AS count
FROM pg_locks
GROUP BY locktype, relation, mode, granted
ORDER BY granted, count DESC
LIMIT 30;
`;

// ── DbMetrics class ───────────────────────────────────────────────────────────

class DbMetrics {
  /**
   * @param {{ connectionString?: string }} options
   */
  constructor(options = {}) {
    this._connectionString =
      options.connectionString ?? process.env.DATABASE_URL ?? "";
    this._client = null;
    this._hasPgStatStatements = null; // lazily detected
  }

  /** Acquire a client (reuse existing open connection). */
  async _connect() {
    if (this._client) return;
    this._client = new Client({ connectionString: this._connectionString });
    await this._client.connect();
  }

  /** Gracefully close the database connection. */
  async close() {
    if (this._client) {
      await this._client.end();
      this._client = null;
    }
  }

  /** Detect whether pg_stat_statements is available. */
  async _detectPgStatStatements() {
    if (this._hasPgStatStatements !== null) return this._hasPgStatStatements;
    try {
      const { rows } = await this._client.query(
        "SELECT 1 FROM pg_extension WHERE extname = 'pg_stat_statements' LIMIT 1;"
      );
      this._hasPgStatStatements = rows.length > 0;
    } catch (_) {
      this._hasPgStatStatements = false;
    }
    return this._hasPgStatStatements;
  }

  /**
   * Collect all database metrics and return a structured snapshot.
   *
   * @returns {Promise<DbMetricsSnapshot>}
   */
  async collect() {
    if (!this._connectionString) {
      return buildEmptySnapshot("DATABASE_URL not configured");
    }

    try {
      await this._connect();
      const hasPss = await this._detectPgStatStatements();

      const [
        connResult,
        dbStatsResult,
        slowRunningResult,
        longTxnResult,
        replicationResult,
        tableSizesResult,
        locksResult,
      ] = await Promise.all([
        this._client.query(SQL_CONNECTIONS),
        this._client.query(SQL_DB_STATS),
        this._client.query(SQL_SLOW_RUNNING, [String(SLOW_QUERY_MS)]),
        this._client.query(SQL_LONG_TRANSACTIONS, [String(LONG_TXN_SECONDS)]),
        this._client.query(SQL_REPLICATION).catch(() => ({ rows: [] })), // not available on standby
        this._client.query(SQL_TABLE_SIZES),
        this._client.query(SQL_LOCKS),
      ]);

      let slowStatements = [];
      if (hasPss) {
        const r = await this._client.query(SQL_SLOW_STATEMENTS).catch(() => ({ rows: [] }));
        slowStatements = r.rows;
      }

      return {
        collected_at: new Date().toISOString(),
        error: null,
        connections: parseConnections(connResult.rows),
        database: dbStatsResult.rows[0] ?? null,
        slow_statements: slowStatements,
        slow_running_queries: slowRunningResult.rows,
        long_running_transactions: longTxnResult.rows,
        replication: replicationResult.rows,
        table_sizes: tableSizesResult.rows,
        locks: locksResult.rows,
        thresholds: {
          slow_query_ms: SLOW_QUERY_MS,
          long_txn_seconds: LONG_TXN_SECONDS,
        },
      };
    } catch (err) {
      console.error("[db-metrics] Collection error:", err.message);
      return buildEmptySnapshot(err.message);
    }
  }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/**
 * Aggregate raw pg_stat_activity rows into a structured summary.
 * @param {object[]} rows
 */
function parseConnections(rows) {
  const summary = {
    active: 0,
    idle: 0,
    idle_in_transaction: 0,
    idle_in_transaction_aborted: 0,
    fastpath_function_call: 0,
    waiting: 0,
    other: 0,
    total: 0,
  };
  for (const row of rows) {
    const count = parseInt(row.count, 10);
    summary.total += count;
    switch (row.state) {
      case "active":
        summary.active += count;
        break;
      case "idle":
        summary.idle += count;
        break;
      case "idle in transaction":
        summary.idle_in_transaction += count;
        break;
      case "idle in transaction (aborted)":
        summary.idle_in_transaction_aborted += count;
        break;
      case "fastpath function call":
        summary.fastpath_function_call += count;
        break;
      default:
        summary.other += count;
    }
    if (row.wait_event_type === "Lock") {
      summary.waiting += count;
    }
  }
  return summary;
}

/** @returns {DbMetricsSnapshot} */
function buildEmptySnapshot(errorMsg) {
  return {
    collected_at: new Date().toISOString(),
    error: errorMsg,
    connections: null,
    database: null,
    slow_statements: [],
    slow_running_queries: [],
    long_running_transactions: [],
    replication: [],
    table_sizes: [],
    locks: [],
    thresholds: {
      slow_query_ms: SLOW_QUERY_MS,
      long_txn_seconds: LONG_TXN_SECONDS,
    },
  };
}

/**
 * @typedef {object} DbMetricsSnapshot
 * @property {string} collected_at
 * @property {string|null} error
 * @property {object|null} connections
 * @property {object|null} database
 * @property {object[]} slow_statements
 * @property {object[]} slow_running_queries
 * @property {object[]} long_running_transactions
 * @property {object[]} replication
 * @property {object[]} table_sizes
 * @property {object[]} locks
 * @property {{ slow_query_ms: number, long_txn_seconds: number }} thresholds
 */

module.exports = { DbMetrics };
