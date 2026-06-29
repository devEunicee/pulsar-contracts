/**
 * Signature verification service (#316)
 *
 * Off-chain ed25519 verification that mirrors the on-chain logic in helper.rs,
 * used to validate payment requests before submitting them to the contract.
 *
 * Features:
 *   - Ed25519 signature verification (Node.js native crypto)
 *   - Signed payload hash verification (SHA-256 of canonical JSON)
 *   - Replay attack prevention via nonce + timestamp window
 *   - Signature format validation (64-byte hex)
 *   - Public key format validation (32-byte hex)
 *   - Structured error recovery
 */

import { createHash, verify as cryptoVerify } from "node:crypto";

// Maximum age of a signed payload (seconds). Replays older than this are rejected.
const TIMESTAMP_WINDOW_SECS = 5 * 60; // 5 minutes

// In-memory nonce store. Replace with Redis for multi-instance deployments.
const usedNonces = new Map(); // nonce → expiry timestamp (ms)

// ── Public API ────────────────────────────────────────────────────────────────

/**
 * Verify an ed25519 signature over a payment order.
 *
 * @param {object} params
 * @param {object} params.order          - The payment order object
 * @param {string} params.signature      - 64-byte hex-encoded signature
 * @param {string} params.publicKey      - 32-byte hex-encoded ed25519 public key
 * @returns {{ valid: boolean, error?: string }}
 */
export function verifyPaymentSignature({ order, signature, publicKey }) {
  try {
    validateKeyFormat(publicKey, 32, "publicKey");
    validateKeyFormat(signature, 64, "signature");

    const payload = canonicalPayload(order);
    const payloadHash = sha256(payload);

    const pubKeyBuf = Buffer.from(publicKey, "hex");
    const sigBuf    = Buffer.from(signature,  "hex");

    const ok = cryptoVerify(
      null,                         // algorithm null → pure ed25519
      payloadHash,
      { key: pubKeyBuf, format: "raw", type: "public", dsaEncoding: "ieee-p1363" },
      sigBuf
    );

    return ok ? { valid: true } : { valid: false, error: "Signature mismatch" };
  } catch (err) {
    return { valid: false, error: `Verification failed: ${err.message}` };
  }
}

/**
 * Check replay protection: nonce must be unused and timestamp must be fresh.
 *
 * @param {string} nonce      - Unique per-request identifier
 * @param {number} timestamp  - Unix seconds when the request was signed
 * @returns {{ valid: boolean, error?: string }}
 */
export function checkReplayProtection(nonce, timestamp) {
  if (!nonce || typeof nonce !== "string" || nonce.length < 8 || nonce.length > 128) {
    return { valid: false, error: "Invalid nonce format" };
  }

  const nowSecs = Math.floor(Date.now() / 1000);
  const age = nowSecs - timestamp;
  if (age < 0 || age > TIMESTAMP_WINDOW_SECS) {
    return { valid: false, error: `Timestamp outside ${TIMESTAMP_WINDOW_SECS}s window` };
  }

  pruneExpiredNonces();

  if (usedNonces.has(nonce)) {
    return { valid: false, error: "Nonce already used (replay detected)" };
  }

  // Record nonce with its expiry (timestamp + window + small buffer)
  usedNonces.set(nonce, (timestamp + TIMESTAMP_WINDOW_SECS + 60) * 1000);
  return { valid: true };
}

/**
 * Full verification: replay check + signature check.
 *
 * @param {object} params
 * @param {object} params.order
 * @param {string} params.signature
 * @param {string} params.publicKey
 * @param {string} params.nonce
 * @param {number} params.timestamp
 * @returns {{ valid: boolean, error?: string }}
 */
export function verifyRequest({ order, signature, publicKey, nonce, timestamp }) {
  const replayResult = checkReplayProtection(nonce, timestamp);
  if (!replayResult.valid) return replayResult;

  return verifyPaymentSignature({ order, signature, publicKey });
}

/**
 * Compute the canonical SHA-256 payload hash for a payment order.
 * This must match the signing procedure used by the merchant SDK.
 *
 * @param {object} order
 * @returns {Buffer}
 */
export function canonicalPayloadHash(order) {
  return sha256(canonicalPayload(order));
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/**
 * Produce a deterministic JSON representation of the order for signing.
 * Keys are sorted alphabetically to ensure consistency across serializers.
 */
function canonicalPayload(order) {
  const sorted = Object.fromEntries(
    Object.keys(order).sort().map((k) => [k, order[k]])
  );
  return Buffer.from(JSON.stringify(sorted));
}

/** SHA-256 of a Buffer or string, returns a Buffer. */
function sha256(data) {
  return createHash("sha256").update(data).digest();
}

/**
 * Validate that `value` is a hex string encoding exactly `expectedBytes` bytes.
 * @param {string} value
 * @param {number} expectedBytes
 * @param {string} name  - field name for error messages
 */
function validateKeyFormat(value, expectedBytes, name) {
  if (typeof value !== "string") {
    throw new TypeError(`${name} must be a string`);
  }
  if (!/^[0-9a-fA-F]+$/.test(value)) {
    throw new TypeError(`${name} must be a hex string`);
  }
  if (value.length !== expectedBytes * 2) {
    throw new TypeError(`${name} must be ${expectedBytes} bytes (${expectedBytes * 2} hex chars)`);
  }
}

/** Remove expired nonces from the in-memory store. */
function pruneExpiredNonces() {
  const now = Date.now();
  for (const [nonce, expiry] of usedNonces) {
    if (expiry < now) usedNonces.delete(nonce);
  }
}
