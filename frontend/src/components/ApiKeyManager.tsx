import React, { useEffect, useState, useCallback } from 'react';
import './ApiKeyManager.css';

const API = process.env.REACT_APP_API_URL || 'http://localhost:3000';

// ── Helpers ───────────────────────────────────────────────────────────────────

function apiFetch(path, opts = {}, owner) {
  return fetch(`${API}${path}`, {
    headers: {
      'Content-Type': 'application/json',
      'X-Owner': owner,
      ...opts.headers,
    },
    ...opts,
  });
}

// ── Sub-components ────────────────────────────────────────────────────────────

function ScopePicker({ selected, onChange, validScopes }) {
  function toggle(scope) {
    onChange(
      selected.includes(scope)
        ? selected.filter((s) => s !== scope)
        : [...selected, scope]
    );
  }
  return (
    <fieldset className="ak-scopes-picker">
      <legend>Scopes</legend>
      {validScopes.map((scope) => (
        <label key={scope} className="ak-scope-label">
          <input
            type="checkbox"
            checked={selected.includes(scope)}
            onChange={() => toggle(scope)}
          />
          <code>{scope}</code>
        </label>
      ))}
    </fieldset>
  );
}

function CreateKeyForm({ validScopes, onCreated, owner }) {
  const [name, setName] = useState('');
  const [scopes, setScopes] = useState(['payments:read']);
  const [rateLimit, setRateLimit] = useState(1000);
  const [expiresAt, setExpiresAt] = useState('');
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState(null);

  async function handleSubmit(e) {
    e.preventDefault();
    setSaving(true);
    setError(null);
    try {
      const res = await apiFetch(
        '/api/keys',
        {
          method: 'POST',
          body: JSON.stringify({
            name,
            scopes,
            rate_limit: rateLimit,
            expires_at: expiresAt || null,
          }),
        },
        owner
      );
      const data = await res.json();
      if (!res.ok) throw new Error(data.error?.message ?? 'Create failed');
      onCreated(data);
      setName('');
      setScopes(['payments:read']);
      setRateLimit(1000);
      setExpiresAt('');
    } catch (err) {
      setError(err.message);
    } finally {
      setSaving(false);
    }
  }

  return (
    <form className="ak-form" onSubmit={handleSubmit} aria-label="Create API key">
      <h3 className="ak-form__title">Create New API Key</h3>

      {error && <p role="alert" className="ak-error">{error}</p>}

      <label className="ak-label">
        Name
        <input
          className="ak-input"
          value={name}
          onChange={(e) => setName(e.target.value)}
          required
          placeholder="e.g. Production webhook integration"
          maxLength={120}
        />
      </label>

      <ScopePicker
        selected={scopes}
        onChange={setScopes}
        validScopes={validScopes}
      />

      <label className="ak-label">
        Rate limit (requests / hour)
        <input
          className="ak-input"
          type="number"
          min={1}
          max={100000}
          value={rateLimit}
          onChange={(e) => setRateLimit(Number(e.target.value))}
        />
      </label>

      <label className="ak-label">
        Expires at (optional)
        <input
          className="ak-input"
          type="datetime-local"
          value={expiresAt}
          onChange={(e) => setExpiresAt(e.target.value)}
        />
      </label>

      <button className="ak-btn ak-btn--primary" type="submit" disabled={saving || scopes.length === 0}>
        {saving ? 'Creating…' : 'Create key'}
      </button>
    </form>
  );
}

function NewKeyBanner({ plaintext, onDismiss }) {
  const [copied, setCopied] = useState(false);

  function copy() {
    navigator.clipboard.writeText(plaintext).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    });
  }

  return (
    <div role="alert" className="ak-new-key-banner">
      <strong>✓ API key created.</strong> Copy it now — it will not be shown again.
      <div className="ak-new-key-row">
        <code className="ak-plaintext">{plaintext}</code>
        <button className="ak-btn ak-btn--secondary" onClick={copy}>
          {copied ? 'Copied!' : 'Copy'}
        </button>
      </div>
      <button className="ak-btn ak-btn--ghost" onClick={onDismiss}>
        I've saved it — dismiss
      </button>
    </div>
  );
}

function KeyRow({ apiKey, owner, onRefresh }) {
  const [showActivity, setShowActivity] = useState(false);
  const [activity, setActivity] = useState(null);
  const [activityLoading, setActivityLoading] = useState(false);
  const [rotating, setRotating] = useState(false);
  const [revoking, setRevoking] = useState(false);
  const [newPlaintext, setNewPlaintext] = useState(null);
  const [error, setError] = useState(null);

  async function rotate() {
    if (!window.confirm('Rotate this key? The old key will stop working immediately.')) return;
    setRotating(true);
    setError(null);
    try {
      const res = await apiFetch(`/api/keys/${apiKey.id}/rotate`, { method: 'POST' }, owner);
      const data = await res.json();
      if (!res.ok) throw new Error(data.error?.message ?? 'Rotate failed');
      setNewPlaintext(data.plaintext);
      onRefresh();
    } catch (err) {
      setError(err.message);
    } finally {
      setRotating(false);
    }
  }

  async function revoke() {
    if (!window.confirm('Permanently revoke this key? This cannot be undone.')) return;
    setRevoking(true);
    setError(null);
    try {
      const res = await apiFetch(`/api/keys/${apiKey.id}`, { method: 'DELETE' }, owner);
      if (!res.ok) {
        const data = await res.json();
        throw new Error(data.error?.message ?? 'Revoke failed');
      }
      onRefresh();
    } catch (err) {
      setError(err.message);
      setRevoking(false);
    }
  }

  async function loadActivity() {
    if (showActivity) { setShowActivity(false); return; }
    setActivityLoading(true);
    try {
      const res = await apiFetch(`/api/keys/${apiKey.id}/activity?limit=20`, {}, owner);
      const data = await res.json();
      if (!res.ok) throw new Error(data.error?.message ?? 'Fetch failed');
      setActivity(data.activity);
      setShowActivity(true);
    } catch (err) {
      setError(err.message);
    } finally {
      setActivityLoading(false);
    }
  }

  const isExpired = apiKey.expires_at && new Date(apiKey.expires_at) < new Date();

  return (
    <li className={`ak-key-row ${apiKey.revoked ? 'ak-key-row--revoked' : ''} ${isExpired ? 'ak-key-row--expired' : ''}`}>
      <div className="ak-key-row__main">
        <div className="ak-key-row__info">
          <strong className="ak-key-name">{apiKey.name}</strong>
          <code className="ak-key-masked" title="Full key is masked for security">
            {apiKey.key_masked}
          </code>
          <div className="ak-key-meta">
            <span className={`ak-tag ${apiKey.revoked ? 'ak-tag--error' : isExpired ? 'ak-tag--warn' : 'ak-tag--ok'}`}>
              {apiKey.revoked ? 'Revoked' : isExpired ? 'Expired' : 'Active'}
            </span>
            {apiKey.scopes.map((s) => (
              <span key={s} className="ak-tag ak-tag--scope">{s}</span>
            ))}
            <span className="ak-key-detail">
              {apiKey.rate_limit.toLocaleString()} req/hr
            </span>
            {apiKey.expires_at && (
              <span className="ak-key-detail">
                Expires: {new Date(apiKey.expires_at).toLocaleDateString()}
              </span>
            )}
            {apiKey.last_used_at && (
              <span className="ak-key-detail">
                Last used: {new Date(apiKey.last_used_at).toLocaleString()}
              </span>
            )}
          </div>
        </div>

        {!apiKey.revoked && (
          <div className="ak-key-row__actions">
            <button
              className="ak-btn ak-btn--secondary"
              onClick={loadActivity}
              disabled={activityLoading}
              aria-expanded={showActivity}
            >
              {activityLoading ? '…' : showActivity ? 'Hide log' : 'Activity'}
            </button>
            <button
              className="ak-btn ak-btn--secondary"
              onClick={rotate}
              disabled={rotating}
            >
              {rotating ? 'Rotating…' : 'Rotate'}
            </button>
            <button
              className="ak-btn ak-btn--danger"
              onClick={revoke}
              disabled={revoking}
            >
              {revoking ? 'Revoking…' : 'Revoke'}
            </button>
          </div>
        )}
      </div>

      {error && <p role="alert" className="ak-error">{error}</p>}

      {newPlaintext && (
        <NewKeyBanner
          plaintext={newPlaintext}
          onDismiss={() => setNewPlaintext(null)}
        />
      )}

      {showActivity && activity !== null && (
        <div className="ak-activity">
          <h4>Recent Activity (last 20)</h4>
          {activity.length === 0 ? (
            <p className="ak-empty">No activity recorded yet.</p>
          ) : (
            <div className="ak-activity-table-wrapper" role="region" aria-label="API key activity">
              <table className="ak-activity-table">
                <thead>
                  <tr>
                    <th>Time</th>
                    <th>Method</th>
                    <th>Path</th>
                    <th>Status</th>
                    <th>Duration</th>
                    <th>IP</th>
                  </tr>
                </thead>
                <tbody>
                  {activity.map((a) => (
                    <tr key={a.id}>
                      <td>{new Date(a.created_at).toLocaleTimeString()}</td>
                      <td><code>{a.method}</code></td>
                      <td><code>{a.path}</code></td>
                      <td className={a.status_code >= 400 ? 'ak-status--error' : ''}>{a.status_code}</td>
                      <td>{a.duration_ms}ms</td>
                      <td>{a.ip_address}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>
      )}
    </li>
  );
}

// ── Main component ────────────────────────────────────────────────────────────

export default function ApiKeyManager({ owner }) {
  const [keys, setKeys] = useState([]);
  const [validScopes, setValidScopes] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [newKeyData, setNewKeyData] = useState(null); // { key, plaintext }

  const fetchKeys = useCallback(async () => {
    if (!owner) return;
    setLoading(true);
    try {
      const res = await apiFetch('/api/keys', {}, owner);
      const data = await res.json();
      if (!res.ok) throw new Error(data.error?.message ?? 'Fetch failed');
      setKeys(data.keys);
      setError(null);
    } catch (err) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  }, [owner]);

  useEffect(() => {
    // Load valid scopes once.
    fetch(`${API}/api/keys/scopes`)
      .then((r) => r.json())
      .then((data) => setValidScopes(data.scopes ?? []))
      .catch(() => {});
  }, []);

  useEffect(() => {
    fetchKeys();
  }, [fetchKeys]);

  if (!owner) {
    return (
      <div className="ak-manager">
        <p className="ak-empty">Connect your wallet to manage API keys.</p>
      </div>
    );
  }

  return (
    <div className="ak-manager">
      <h2 className="ak-manager__title">API Key Management</h2>

      {newKeyData && (
        <NewKeyBanner
          plaintext={newKeyData.plaintext}
          onDismiss={() => setNewKeyData(null)}
        />
      )}

      <CreateKeyForm
        validScopes={validScopes}
        owner={owner}
        onCreated={(data) => {
          setNewKeyData(data);
          fetchKeys();
        }}
      />

      <section className="ak-keys-section" aria-labelledby="ak-keys-heading">
        <h3 id="ak-keys-heading" className="ak-keys-heading">
          Your API Keys {!loading && `(${keys.length})`}
        </h3>

        {error && <p role="alert" className="ak-error">{error}</p>}
        {loading && <p aria-live="polite">Loading keys…</p>}

        {!loading && keys.length === 0 && (
          <p className="ak-empty">No API keys yet. Create one above.</p>
        )}

        <ul className="ak-keys-list" aria-label="API keys">
          {keys.map((k) => (
            <KeyRow
              key={k.id}
              apiKey={k}
              owner={owner}
              onRefresh={fetchKeys}
            />
          ))}
        </ul>
      </section>
    </div>
  );
}
