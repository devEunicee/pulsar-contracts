import React, { useEffect, useState, useCallback } from 'react';
import './DbMonitoringDashboard.css';

const DASHBOARD_API =
  process.env.REACT_APP_DASHBOARD_URL || 'http://localhost:4000';

const REFRESH_INTERVAL_MS = 30_000;

// ── Helpers ───────────────────────────────────────────────────────────────────

function Badge({ label, value, variant = 'neutral' }) {
  return (
    <div className={`db-badge db-badge--${variant}`}>
      <span className="db-badge__label">{label}</span>
      <span className="db-badge__value">{value}</span>
    </div>
  );
}

function MetricCard({ title, children }) {
  return (
    <section className="db-card" aria-labelledby={`card-${title}`}>
      <h3 id={`card-${title}`} className="db-card__title">
        {title}
      </h3>
      <div className="db-card__body">{children}</div>
    </section>
  );
}

function StatusDot({ ok }) {
  return (
    <span
      className={`db-status-dot ${ok ? 'db-status-dot--ok' : 'db-status-dot--warn'}`}
      role="img"
      aria-label={ok ? 'Healthy' : 'Warning'}
    />
  );
}

// ── Sections ──────────────────────────────────────────────────────────────────

function ConnectionsPanel({ connections }) {
  if (!connections) return <p>No data</p>;
  const { total, active, idle, idle_in_transaction, waiting, other } =
    connections;
  return (
    <div className="db-connections">
      <Badge label="Total" value={total} />
      <Badge label="Active" value={active} variant={active > 50 ? 'warn' : 'ok'} />
      <Badge label="Idle" value={idle} />
      <Badge
        label="Idle in Txn"
        value={idle_in_transaction}
        variant={idle_in_transaction >= 5 ? 'warn' : 'neutral'}
      />
      <Badge
        label="Waiting (Lock)"
        value={waiting}
        variant={waiting > 0 ? 'warn' : 'neutral'}
      />
      <Badge label="Other" value={other} />
    </div>
  );
}

function DatabaseStatsPanel({ stats }) {
  if (!stats) return <p>No data</p>;
  const hitRatio = parseFloat(stats.cache_hit_ratio ?? 0);
  return (
    <div className="db-stats-grid">
      <div className="db-stat">
        <StatusDot ok={hitRatio >= 95} />
        <span>Cache hit ratio</span>
        <strong>{hitRatio}%</strong>
      </div>
      <div className="db-stat">
        <span>Commits</span>
        <strong>{Number(stats.xact_commit ?? 0).toLocaleString()}</strong>
      </div>
      <div className="db-stat">
        <span>Rollbacks</span>
        <strong>{Number(stats.xact_rollback ?? 0).toLocaleString()}</strong>
      </div>
      <div className="db-stat">
        <span>Deadlocks</span>
        <strong className={Number(stats.deadlocks) > 0 ? 'db-text--warn' : ''}>
          {stats.deadlocks ?? 0}
        </strong>
      </div>
      <div className="db-stat">
        <span>Conflicts</span>
        <strong>{stats.conflicts ?? 0}</strong>
      </div>
      <div className="db-stat">
        <span>Temp bytes</span>
        <strong>{formatBytes(Number(stats.temp_bytes ?? 0))}</strong>
      </div>
    </div>
  );
}

function SlowQueriesPanel({ queries, thresholds }) {
  if (queries.length === 0)
    return (
      <p className="db-empty">
        No queries currently running longer than {thresholds?.slow_query_ms} ms.
      </p>
    );
  return (
    <div className="db-table-wrapper" role="region" aria-label="Slow running queries">
      <table className="db-table">
        <thead>
          <tr>
            <th>PID</th>
            <th>State</th>
            <th>Duration (ms)</th>
            <th>Wait event</th>
            <th>Query preview</th>
          </tr>
        </thead>
        <tbody>
          {queries.map((q) => (
            <tr key={q.pid} className="db-table__row--warn">
              <td>{q.pid}</td>
              <td>{q.state}</td>
              <td>{Math.round(parseFloat(q.duration_ms)).toLocaleString()}</td>
              <td>{q.wait_event ?? '—'}</td>
              <td>
                <code className="db-code">{q.query_preview}</code>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function LongTransactionsPanel({ transactions, thresholds }) {
  if (transactions.length === 0)
    return (
      <p className="db-empty">
        No transactions running longer than {thresholds?.long_txn_seconds} s.
      </p>
    );
  return (
    <div className="db-table-wrapper" role="region" aria-label="Long running transactions">
      <table className="db-table">
        <thead>
          <tr>
            <th>PID</th>
            <th>User</th>
            <th>State</th>
            <th>Duration (s)</th>
            <th>Last query</th>
          </tr>
        </thead>
        <tbody>
          {transactions.map((t) => (
            <tr key={t.pid} className="db-table__row--warn">
              <td>{t.pid}</td>
              <td>{t.usename}</td>
              <td>{t.state}</td>
              <td>{Math.round(parseFloat(t.duration_seconds))}</td>
              <td>
                <code className="db-code">{t.last_query}</code>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function ReplicationPanel({ replication }) {
  if (replication.length === 0)
    return <p className="db-empty">No replication standbys detected.</p>;
  return (
    <div className="db-table-wrapper" role="region" aria-label="Replication status">
      <table className="db-table">
        <thead>
          <tr>
            <th>Standby</th>
            <th>Client</th>
            <th>State</th>
            <th>Sync state</th>
            <th>Replay lag</th>
          </tr>
        </thead>
        <tbody>
          {replication.map((r, i) => {
            const lagBytes = parseInt(r.replay_lag_bytes ?? '0', 10);
            const isLagging = lagBytes > 10 * 1024 * 1024;
            return (
              <tr key={i} className={isLagging ? 'db-table__row--warn' : ''}>
                <td>{r.application_name}</td>
                <td>{r.client_addr}</td>
                <td>{r.state}</td>
                <td>{r.sync_state}</td>
                <td>
                  {isLagging && <StatusDot ok={false} />}
                  {formatBytes(lagBytes)}
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}

function TableSizesPanel({ tables }) {
  if (tables.length === 0) return <p className="db-empty">No table data.</p>;
  return (
    <div className="db-table-wrapper" role="region" aria-label="Table disk usage">
      <table className="db-table">
        <thead>
          <tr>
            <th>Table</th>
            <th>Total size</th>
            <th>Table size</th>
            <th>Index size</th>
          </tr>
        </thead>
        <tbody>
          {tables.map((t) => (
            <tr key={`${t.schemaname}.${t.tablename}`}>
              <td>
                <code>{t.schemaname}.{t.tablename}</code>
              </td>
              <td>{t.total_size}</td>
              <td>{t.table_size}</td>
              <td>{t.index_size}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function SlowStatementsPanel({ statements }) {
  if (statements.length === 0)
    return (
      <p className="db-empty">
        pg_stat_statements not available or no data.
      </p>
    );
  return (
    <div className="db-table-wrapper" role="region" aria-label="Slow SQL statements">
      <table className="db-table">
        <thead>
          <tr>
            <th>Mean (ms)</th>
            <th>Total (ms)</th>
            <th>Calls</th>
            <th>Rows</th>
            <th>Query preview</th>
          </tr>
        </thead>
        <tbody>
          {statements.map((s, i) => (
            <tr key={i}>
              <td>{s.mean_exec_ms}</td>
              <td>{Number(s.total_exec_ms).toLocaleString()}</td>
              <td>{Number(s.calls).toLocaleString()}</td>
              <td>{Number(s.rows).toLocaleString()}</td>
              <td>
                <code className="db-code">{s.query_preview}</code>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function AlertsPanel({ alerts }) {
  if (alerts.length === 0)
    return <p className="db-empty">No recent alerts.</p>;
  return (
    <ul className="db-alerts-list" aria-label="Recent database alerts">
      {alerts.slice(0, 20).map((a, i) => (
        <li key={i} className="db-alert-item">
          <span className="db-alert-item__type">{a.type}</span>
          <span className="db-alert-item__time">
            {new Date(a.timestamp).toLocaleTimeString()}
          </span>
          <pre className="db-alert-item__data">
            {JSON.stringify(a.data, null, 2)}
          </pre>
        </li>
      ))}
    </ul>
  );
}

// ── Utility ───────────────────────────────────────────────────────────────────

function formatBytes(bytes) {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`;
}

// ── Main Dashboard ────────────────────────────────────────────────────────────

export default function DbMonitoringDashboard() {
  const [dbSnapshot, setDbSnapshot] = useState(null);
  const [alerts, setAlerts] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [lastRefresh, setLastRefresh] = useState(null);

  const fetchData = useCallback(async () => {
    try {
      const [dbRes, alertRes] = await Promise.all([
        fetch(`${DASHBOARD_API}/metrics/db`),
        fetch(`${DASHBOARD_API}/metrics/alerts`),
      ]);

      if (!dbRes.ok) throw new Error(`DB metrics fetch failed: ${dbRes.status}`);
      if (!alertRes.ok) throw new Error(`Alerts fetch failed: ${alertRes.status}`);

      const [dbData, alertData] = await Promise.all([
        dbRes.json(),
        alertRes.json(),
      ]);

      setDbSnapshot(dbData);
      setAlerts(alertData.alerts ?? []);
      setError(null);
      setLastRefresh(new Date());
    } catch (err) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchData();
    const timer = setInterval(fetchData, REFRESH_INTERVAL_MS);
    return () => clearInterval(timer);
  }, [fetchData]);

  return (
    <div className="db-dashboard">
      <header className="db-dashboard__header">
        <h2 className="db-dashboard__title">Database Monitoring Dashboard</h2>
        <div className="db-dashboard__meta">
          {lastRefresh && (
            <span>
              Last refresh: {lastRefresh.toLocaleTimeString()} &nbsp;
              (auto-refreshes every {REFRESH_INTERVAL_MS / 1000}s)
            </span>
          )}
          <button
            className="db-btn db-btn--secondary"
            onClick={fetchData}
            disabled={loading}
            aria-label="Refresh metrics"
          >
            {loading ? 'Refreshing…' : '↻ Refresh'}
          </button>
        </div>
      </header>

      {error && (
        <p role="alert" className="db-error">
          {error}
        </p>
      )}

      {dbSnapshot?.error && (
        <p role="alert" className="db-error">
          Backend error: {dbSnapshot.error}
        </p>
      )}

      <div className="db-dashboard__grid">
        <MetricCard title="Connections">
          <ConnectionsPanel connections={dbSnapshot?.connections} />
        </MetricCard>

        <MetricCard title="Database Stats">
          <DatabaseStatsPanel stats={dbSnapshot?.database} />
        </MetricCard>

        <MetricCard title="Slow Running Queries">
          <SlowQueriesPanel
            queries={dbSnapshot?.slow_running_queries ?? []}
            thresholds={dbSnapshot?.thresholds}
          />
        </MetricCard>

        <MetricCard title="Long-Running Transactions">
          <LongTransactionsPanel
            transactions={dbSnapshot?.long_running_transactions ?? []}
            thresholds={dbSnapshot?.thresholds}
          />
        </MetricCard>

        <MetricCard title="Replication Lag">
          <ReplicationPanel replication={dbSnapshot?.replication ?? []} />
        </MetricCard>

        <MetricCard title="Disk Usage (Top Tables)">
          <TableSizesPanel tables={dbSnapshot?.table_sizes ?? []} />
        </MetricCard>

        <MetricCard title="Slowest SQL Statements (pg_stat_statements)">
          <SlowStatementsPanel
            statements={dbSnapshot?.slow_statements ?? []}
          />
        </MetricCard>

        <MetricCard title="Recent Alerts">
          <AlertsPanel alerts={alerts} />
        </MetricCard>
      </div>

      <p className="db-dashboard__footer">
        Collected at: {dbSnapshot?.collected_at ?? '—'}
      </p>
    </div>
  );
}
