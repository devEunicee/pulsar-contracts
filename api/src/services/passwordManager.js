/**
 * Password management service (#314)
 *
 * Provides:
 *   - hashPassword / verifyPassword (bcrypt)
 *   - validateStrength (12+ chars, upper, lower, digit, special)
 *   - checkHistory (prevent reuse of last N passwords)
 *   - generateResetToken / validateResetToken / consumeResetToken
 *   - checkResetRateLimit
 */

import { createHash, randomBytes } from "node:crypto";
import bcrypt from "bcrypt";
import { pool } from "../db.js";

const BCRYPT_ROUNDS       = 12;
const HISTORY_DEPTH       = 5;          // prevent reuse of last 5 passwords
const RESET_TOKEN_TTL_MS  = 30 * 60 * 1000; // 30 minutes
const RATE_LIMIT_WINDOW   = 60 * 60;    // 1 hour (seconds)
const RATE_LIMIT_MAX      = 5;          // max attempts per window

// ── Hashing ───────────────────────────────────────────────────────────────────

/**
 * Hash a plaintext password with bcrypt.
 * @param {string} plaintext
 * @returns {Promise<string>} bcrypt hash
 */
export async function hashPassword(plaintext) {
  return bcrypt.hash(plaintext, BCRYPT_ROUNDS);
}

/**
 * Verify a plaintext password against a stored bcrypt hash.
 * @param {string} plaintext
 * @param {string} hash
 * @returns {Promise<boolean>}
 */
export async function verifyPassword(plaintext, hash) {
  return bcrypt.compare(plaintext, hash);
}

// ── Strength validation ───────────────────────────────────────────────────────

const STRENGTH_RULES = [
  { re: /.{12,}/, msg: "at least 12 characters" },
  { re: /[A-Z]/,  msg: "at least one uppercase letter" },
  { re: /[a-z]/,  msg: "at least one lowercase letter" },
  { re: /\d/,     msg: "at least one digit" },
  { re: /[^A-Za-z0-9]/, msg: "at least one special character" },
];

/**
 * Validate password strength.
 * @param {string} plaintext
 * @returns {{ valid: boolean, errors: string[] }}
 */
export function validateStrength(plaintext) {
  const errors = STRENGTH_RULES
    .filter(({ re }) => !re.test(plaintext))
    .map(({ msg }) => msg);
  return { valid: errors.length === 0, errors };
}

// ── Password history ──────────────────────────────────────────────────────────

/**
 * Return true if `plaintext` matches any of the user's last N hashes.
 * @param {string} userId
 * @param {string} plaintext
 * @returns {Promise<boolean>}
 */
export async function isPasswordReused(userId, plaintext) {
  const { rows } = await pool.query(
    `SELECT password_hash FROM password_history
      WHERE user_id = $1
      ORDER BY created_at DESC
      LIMIT $2`,
    [userId, HISTORY_DEPTH]
  );
  for (const { password_hash } of rows) {
    if (await bcrypt.compare(plaintext, password_hash)) return true;
  }
  return false;
}

/**
 * Persist the new hash into history, keeping only the last HISTORY_DEPTH entries.
 * @param {string} userId
 * @param {string} hash  - already-hashed value
 */
export async function recordPasswordHistory(userId, hash) {
  const client = await pool.connect();
  try {
    await client.query("BEGIN");
    await client.query(
      `INSERT INTO password_history (user_id, password_hash) VALUES ($1, $2)`,
      [userId, hash]
    );
    // Trim history to HISTORY_DEPTH rows
    await client.query(
      `DELETE FROM password_history
        WHERE id NOT IN (
          SELECT id FROM password_history
           WHERE user_id = $1
           ORDER BY created_at DESC
           LIMIT $2
        ) AND user_id = $1`,
      [userId, HISTORY_DEPTH]
    );
    await client.query("COMMIT");
  } catch (err) {
    await client.query("ROLLBACK");
    throw err;
  } finally {
    client.release();
  }
}

// ── Reset tokens ──────────────────────────────────────────────────────────────

/**
 * Generate a cryptographically secure reset token, store its hash, and return
 * the raw token (to be delivered via email — never stored in plaintext).
 * @param {string} userId
 * @returns {Promise<string>} raw token
 */
export async function generateResetToken(userId) {
  const raw  = randomBytes(32).toString("hex");
  const hash = sha256(raw);
  const expiresAt = new Date(Date.now() + RESET_TOKEN_TTL_MS);

  // Invalidate any existing unused tokens for this user
  await pool.query(
    `UPDATE password_reset_tokens SET used = TRUE
      WHERE user_id = $1 AND used = FALSE`,
    [userId]
  );

  await pool.query(
    `INSERT INTO password_reset_tokens (token_hash, user_id, expires_at)
     VALUES ($1, $2, $3)`,
    [hash, userId, expiresAt]
  );

  return raw;
}

/**
 * Validate a raw reset token — checks existence, expiry, and unused status.
 * Does NOT consume the token; call consumeResetToken after updating the password.
 * @param {string} rawToken
 * @returns {Promise<string|null>} userId if valid, null otherwise
 */
export async function validateResetToken(rawToken) {
  const hash = sha256(rawToken);
  const { rows } = await pool.query(
    `SELECT user_id FROM password_reset_tokens
      WHERE token_hash = $1 AND used = FALSE AND expires_at > NOW()`,
    [hash]
  );
  return rows.length ? rows[0].user_id : null;
}

/**
 * Mark a reset token as used and invalidate all sessions (caller must handle
 * session invalidation at the auth layer).
 * @param {string} rawToken
 */
export async function consumeResetToken(rawToken) {
  const hash = sha256(rawToken);
  await pool.query(
    `UPDATE password_reset_tokens SET used = TRUE WHERE token_hash = $1`,
    [hash]
  );
}

// ── Rate limiting ─────────────────────────────────────────────────────────────

/**
 * Check whether the identifier (userId or IP) has exceeded the reset rate limit.
 * Records the attempt regardless.
 * @param {string} identifier
 * @returns {Promise<boolean>} true if limit exceeded (request should be blocked)
 */
export async function checkResetRateLimit(identifier) {
  await pool.query(
    `INSERT INTO password_reset_attempts (identifier) VALUES ($1)`,
    [identifier]
  );
  const { rows } = await pool.query(
    `SELECT COUNT(*) AS cnt FROM password_reset_attempts
      WHERE identifier = $1
        AND attempted_at > NOW() - ($2 || ' seconds')::interval`,
    [identifier, RATE_LIMIT_WINDOW]
  );
  return parseInt(rows[0].cnt, 10) > RATE_LIMIT_MAX;
}

// ── Helpers ───────────────────────────────────────────────────────────────────

function sha256(str) {
  return createHash("sha256").update(str).digest("hex");
}
