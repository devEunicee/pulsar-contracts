/**
 * apiKeys.js — API Key Management service (#315).
 *
 * Provides:
 *   - Key generation (with secure random bytes + SHA-256 hashing)
 *   - Key rotation (atomically revoke old + issue new)
 *   - Scope-based permission model
 *   - Per-key rate limiting (token-bucket via in-memory store)
 *   - Key expiration checking
 *   - Revocation
 *   - Activity logging
 *
 * The plaintext key is returned ONCE at creation/rotation time and is never
 * stored.  The storage layer holds only a SHA-256 hash of the key.
 *
 * Key format:  psk_<32-hex-chars>
 *              └─ prefix  └─ 128 bits of entropy (always stored hashed)
 *
 * Scopes (comma-separated list stored in `api_keys.scopes`):
 *   payments:read    GET payment records
 *   payments:write   POST / mutate payments
 *   merchants:read   GET merchant records
 *   merchants:write  POST / mutate merchants
 *   refunds:read     GET refund records
 *   refunds:write    POST / mutate refunds
 *   admin            Unrestricted (admin only)
 */

import crypto from "crypto";
import { Client } from "pg";

// ── Constants ─────────────────────────────────────────────────────────────────

export const VALID_SCOPES = Object.freeze([
  "payments:read",
  "payments:write",
  "merchants:read",
  "merchants:write",
  "refunds:read",
  "refunds:write",
  "admin",
]);

const KEY_PREFIX = "psk_";
const KEY_ENTROPY_BYTES = 16; // 128-bit key body → 32 hex chars
const RATE_LIMIT_WINDOW_MS = 60 * 60 * 1000; // 1 hour

// ── In-memory rate-limit store ────────────────────────────────────────────────

/**
 * Map<keyId, { count: number, windowStart: number }>
 * Resets whenever the current window expires.
 */
const rateLimitStore = new Map();

/**
 * Check and consume one token from a key's rate-limit bucket.
 *
 * @param {string} keyId   The api_keys.id value.
 * @param {number} limit   Requests allowed per hour.
 * @returns {{ allowed: boolean, remaining: number, resetAt: number }}
 */
export function checkRateLimit(keyId, limit) {
  const now = Date.now();
  let bucket = rateLimitStore.get(keyId);

  if (!bucket || now - bucket.windowStart >= RATE_LIMIT_WINDOW_MS) {
    bucket = { count: 0, windowStart: now };
    rateLimitStore.set(keyId, bucket);
  }

  bucket.count += 1;
  const remaining = Math.max(0, limit - bucket.count);
  const resetAt = bucket.windowStart + RATE_LIMIT_WINDOW_MS;

  return {
    allowed: bucket.count <= limit,
    remaining,
    resetAt,
  };
}

// ── Key generation ────────────────────────────────────────────────────────────

/**
 * Generate a new API key (plaintext) and its storage hash.
 *
 * @returns {{ plaintext: string, hash: string, prefix: string }}
 */
export function generateApiKey() {
  const body = crypto.randomBytes(KEY_ENTROPY_BYTES).toString("hex");
  const plaintext = `${KEY_PREFIX}${body}`;
  const hash = crypto.createHash("sha256").update(plaintext).digest("hex");
  // prefix = first 8 visible chars (shown in UI as psk_XXXX)
  const prefix = plaintext.slice(0, 8);
  return { plaintext, hash, prefix };
}

/**
 * Hash a user-supplied key for comparison against stored hashes.
 *
 * @param {string} key  Plaintext API key.
 * @returns {string}    SHA-256 hex digest.
 */
export function hashApiKey(key) {
  return crypto.createHash("sha256").update(key).digest("hex");
}

// ── Scope helpers ─────────────────────────────────────────────────────────────

/**
 * Validate that all requested scopes are recognised.
 *
 * @param {string[]} scopes
 * @returns {{ valid: boolean, unknown: string[] }}
 */
export function validateScopes(scopes) {
  const unknown = scopes.filter((s) => !VALID_SCOPES.includes(s));
  return { valid: unknown.length === 0, unknown };
}

/**
 * Check whether a key's scopes include the required scope.
 *
 * @param {string} storedScopes  Comma-separated scope list from DB.
 * @param {string} required      The scope to check for.
 * @returns {boolean}
 */
export function hasScope(storedScopes, required) {
  const scopes = storedScopes.split(",").map((s) => s.trim());
  return scopes.includes("admin") || scopes.includes(required);
}

// ── Database service ──────────────────────────────────────────────────────────

export class ApiKeyService {
  /**
   * @param {{ connectionString?: string } | import("pg").Client} dbOrOpts
   */
  constructor(dbOrOpts = {}) {
    if (dbOrOpts instanceof Client) {
      this._db = dbOrOpts;
      this._ownsDb = false;
    } else {
      this._db = new Client({
        connectionString:
          dbOrOpts.connectionString ?? process.env.DATABASE_URL,
      });
      this._ownsDb = true;
    }
    this._connected = false;
  }

  async _ensureConnected() {
    if (!this._connected) {
      await this._db.connect();
      this._connected = true;
    }
  }

  async close() {
    if (this._ownsDb && this._connected) {
      await this._db.end();
      this._connected = false;
    }
  }

  // ── Create ──────────────────────────────────────────────────────────────────

  /**
   * Create a new API key.
   *
   * @param {object} opts
   * @param {string}   opts.owner       Stellar account address of the key owner.
   * @param {string}   opts.name        Human-readable label.
   * @param {string[]} opts.scopes      Permission scopes.
   * @param {number}   [opts.rateLimit] Requests per hour (default 1000).
   * @param {Date|null}[opts.expiresAt] Expiry date, null = never.
   * @returns {Promise<{ key: ApiKeyRecord, plaintext: string }>}
   */
  async create({ owner, name, scopes, rateLimit = 1000, expiresAt = null }) {
    const scopeValidation = validateScopes(scopes);
    if (!scopeValidation.valid) {
      throw new Error(
        `Unknown scopes: ${scopeValidation.unknown.join(", ")}. ` +
          `Valid scopes: ${VALID_SCOPES.join(", ")}`
      );
    }

    await this._ensureConnected();
    const id = crypto.randomUUID();
    const { plaintext, hash, prefix } = generateApiKey();
    const scopeStr = scopes.join(",");

    const { rows } = await this._db.query(
      `INSERT INTO api_keys
         (id, name, owner, key_prefix, key_hash, scopes, rate_limit, expires_at)
       VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
       RETURNING *`,
      [id, name, owner, prefix, hash, scopeStr, rateLimit, expiresAt]
    );

    return { key: maskKey(rows[0]), plaintext };
  }

  // ── List ────────────────────────────────────────────────────────────────────

  /**
   * List all API keys for an owner (sensitive fields masked).
   *
   * @param {string} owner
   * @returns {Promise<ApiKeyRecord[]>}
   */
  async listByOwner(owner) {
    await this._ensureConnected();
    const { rows } = await this._db.query(
      `SELECT * FROM api_keys WHERE owner = $1 ORDER BY created_at DESC`,
      [owner]
    );
    return rows.map(maskKey);
  }

  // ── Get by ID ───────────────────────────────────────────────────────────────

  /**
   * Get a single API key record by ID (sensitive fields masked).
   *
   * @param {string} id
   * @param {string} owner  Caller's address — must match key owner.
   * @returns {Promise<ApiKeyRecord|null>}
   */
  async getById(id, owner) {
    await this._ensureConnected();
    const { rows } = await this._db.query(
      `SELECT * FROM api_keys WHERE id = $1 AND owner = $2`,
      [id, owner]
    );
    return rows.length ? maskKey(rows[0]) : null;
  }

  // ── Authenticate ────────────────────────────────────────────────────────────

  /**
   * Validate an API key and return the key record if valid.
   * Updates last_used_at in the background (fire-and-forget).
   *
   * @param {string} plaintext  The Bearer key from the request.
   * @returns {Promise<ApiKeyRecord|null>}
   */
  async authenticate(plaintext) {
    await this._ensureConnected();
    const hash = hashApiKey(plaintext);

    const { rows } = await this._db.query(
      `SELECT * FROM api_keys WHERE key_hash = $1`,
      [hash]
    );

    if (!rows.length) return null;
    const record = rows[0];

    // Reject revoked keys.
    if (record.revoked) return null;

    // Reject expired keys.
    if (record.expires_at && new Date(record.expires_at) < new Date()) return null;

    // Update last_used_at asynchronously (non-blocking).
    this._db
      .query(`UPDATE api_keys SET last_used_at = now() WHERE id = $1`, [
        record.id,
      ])
      .catch(() => {/* best-effort */});

    return maskKey(record);
  }

  // ── Rotate ──────────────────────────────────────────────────────────────────

  /**
   * Rotate an API key: revoke the old key and issue a new one with the same
   * name, scopes, rate limit, and expiry.
   *
   * @param {string} id     Key ID to rotate.
   * @param {string} owner  Caller's address — must match key owner.
   * @returns {Promise<{ key: ApiKeyRecord, plaintext: string }>}
   */
  async rotate(id, owner) {
    await this._ensureConnected();

    const { rows } = await this._db.query(
      `SELECT * FROM api_keys WHERE id = $1 AND owner = $2 AND revoked = false`,
      [id, owner]
    );
    if (!rows.length) throw new Error("API key not found or already revoked");

    const old = rows[0];
    const newId = crypto.randomUUID();
    const { plaintext, hash, prefix } = generateApiKey();

    // Revoke old + insert new in a transaction.
    await this._db.query("BEGIN");
    try {
      await this._db.query(
        `UPDATE api_keys SET revoked = true, revoked_at = now(), updated_at = now() WHERE id = $1`,
        [old.id]
      );
      const { rows: newRows } = await this._db.query(
        `INSERT INTO api_keys
           (id, name, owner, key_prefix, key_hash, scopes, rate_limit, expires_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         RETURNING *`,
        [
          newId,
          old.name,
          old.owner,
          prefix,
          hash,
          old.scopes,
          old.rate_limit,
          old.expires_at,
        ]
      );
      await this._db.query("COMMIT");
      return { key: maskKey(newRows[0]), plaintext };
    } catch (err) {
      await this._db.query("ROLLBACK");
      throw err;
    }
  }

  // ── Revoke ──────────────────────────────────────────────────────────────────

  /**
   * Revoke an API key immediately.
   *
   * @param {string} id     Key ID to revoke.
   * @param {string} owner  Caller's address — must match key owner.
   * @returns {Promise<void>}
   */
  async revoke(id, owner) {
    await this._ensureConnected();
    const { rowCount } = await this._db.query(
      `UPDATE api_keys
       SET revoked = true, revoked_at = now(), updated_at = now()
       WHERE id = $1 AND owner = $2 AND revoked = false`,
      [id, owner]
    );
    if (rowCount === 0)
      throw new Error("API key not found, already revoked, or not owned by caller");
  }

  // ── Update ──────────────────────────────────────────────────────────────────

  /**
   * Update mutable fields (name, scopes, rateLimit, expiresAt).
   *
   * @param {string} id
   * @param {string} owner
   * @param {Partial<{ name: string, scopes: string[], rateLimit: number, expiresAt: Date|null }>} updates
   * @returns {Promise<ApiKeyRecord>}
   */
  async update(id, owner, updates) {
    await this._ensureConnected();

    const sets = [];
    const values = [];
    let idx = 1;

    if (updates.name !== undefined) {
      sets.push(`name = $${idx++}`);
      values.push(updates.name);
    }
    if (updates.scopes !== undefined) {
      const { valid, unknown } = validateScopes(updates.scopes);
      if (!valid) throw new Error(`Unknown scopes: ${unknown.join(", ")}`);
      sets.push(`scopes = $${idx++}`);
      values.push(updates.scopes.join(","));
    }
    if (updates.rateLimit !== undefined) {
      sets.push(`rate_limit = $${idx++}`);
      values.push(updates.rateLimit);
    }
    if (updates.expiresAt !== undefined) {
      sets.push(`expires_at = $${idx++}`);
      values.push(updates.expiresAt);
    }

    if (sets.length === 0) throw new Error("No fields to update");

    sets.push(`updated_at = now()`);
    values.push(id, owner);

    const { rows } = await this._db.query(
      `UPDATE api_keys SET ${sets.join(", ")}
       WHERE id = $${idx++} AND owner = $${idx++} AND revoked = false
       RETURNING *`,
      values
    );

    if (!rows.length) throw new Error("API key not found or not owned by caller");
    return maskKey(rows[0]);
  }

  // ── Activity log ────────────────────────────────────────────────────────────

  /**
   * Append one activity log entry for the given key.
   *
   * @param {string} keyId
   * @param {{ ip: string, method: string, path: string, statusCode: number, durationMs: number }} entry
   */
  async logActivity(keyId, { ip, method, path, statusCode, durationMs }) {
    try {
      await this._ensureConnected();
      await this._db.query(
        `INSERT INTO api_key_activity (key_id, ip_address, method, path, status_code, duration_ms)
         VALUES ($1, $2, $3, $4, $5, $6)`,
        [keyId, ip, method, path, statusCode, durationMs]
      );
    } catch (_) {
      // Activity logging is best-effort; never crash the request.
    }
  }

  /**
   * Get the last N activity entries for a key.
   *
   * @param {string} keyId
   * @param {string} owner  Must match key owner.
   * @param {number} [limit=50]
   * @returns {Promise<object[]>}
   */
  async getActivity(keyId, owner, limit = 50) {
    await this._ensureConnected();
    // Verify ownership first.
    const { rows: keyRows } = await this._db.query(
      `SELECT id FROM api_keys WHERE id = $1 AND owner = $2`,
      [keyId, owner]
    );
    if (!keyRows.length) throw new Error("API key not found or not owned by caller");

    const { rows } = await this._db.query(
      `SELECT id, ip_address, method, path, status_code, duration_ms, created_at
       FROM api_key_activity
       WHERE key_id = $1
       ORDER BY created_at DESC
       LIMIT $2`,
      [keyId, limit]
    );
    return rows;
  }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/**
 * Mask sensitive fields before returning a key record to the caller.
 * - key_hash is removed entirely (never exposed via API)
 * - key_prefix shows only last-4 style: "psk_****...XXXX"
 *
 * @param {object} row  Raw DB row.
 * @returns {ApiKeyRecord}
 */
function maskKey(row) {
  const { key_hash, ...safe } = row;  // eslint-disable-line no-unused-vars
  return {
    ...safe,
    // Show only last 4 chars of the key prefix per acceptance criteria.
    key_masked: `psk_****${row.key_prefix.slice(-4)}`,
    scopes: row.scopes ? row.scopes.split(",") : [],
  };
}

/**
 * @typedef {object} ApiKeyRecord
 * @property {string}    id
 * @property {string}    name
 * @property {string}    owner
 * @property {string}    key_masked    "psk_****XXXX"
 * @property {string[]}  scopes
 * @property {number}    rate_limit
 * @property {string|null} expires_at
 * @property {string|null} last_used_at
 * @property {boolean}   revoked
 * @property {string|null} revoked_at
 * @property {string}    created_at
 * @property {string}    updated_at
 */
