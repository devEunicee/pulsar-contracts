/**
 * TOTP (Time-based One-Time Password) service (Issue #313)
 *
 * Compatible with Google Authenticator and any RFC 6238-compliant app.
 * No external dependencies — uses only Node.js built-in `crypto`.
 */

import { createHmac, randomBytes } from "node:crypto";

const STEP      = 30;   // seconds per TOTP window
const DIGITS    = 6;
const TOLERANCE = 1;    // accept ±1 window for clock drift

// ── base32 helpers ────────────────────────────────────────────────────────────

const BASE32_CHARS = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

function base32Encode(buf) {
  let bits = 0, value = 0, output = "";
  for (const byte of buf) {
    value = (value << 8) | byte;
    bits += 8;
    while (bits >= 5) {
      output += BASE32_CHARS[(value >>> (bits - 5)) & 31];
      bits -= 5;
    }
  }
  if (bits > 0) output += BASE32_CHARS[(value << (5 - bits)) & 31];
  return output;
}

function base32Decode(str) {
  const s = str.toUpperCase().replace(/=+$/, "");
  let bits = 0, value = 0;
  const out = [];
  for (const ch of s) {
    const idx = BASE32_CHARS.indexOf(ch);
    if (idx === -1) throw new Error(`Invalid base32 char: ${ch}`);
    value = (value << 5) | idx;
    bits += 5;
    if (bits >= 8) {
      out.push((value >>> (bits - 8)) & 0xff);
      bits -= 8;
    }
  }
  return Buffer.from(out);
}

// ── HOTP core ─────────────────────────────────────────────────────────────────

function hotp(secret, counter) {
  const key     = base32Decode(secret);
  const counter8 = Buffer.alloc(8);
  // write 64-bit big-endian counter
  counter8.writeBigInt64BE(BigInt(counter));
  const hmac  = createHmac("sha1", key).update(counter8).digest();
  const offset = hmac[hmac.length - 1] & 0x0f;
  const code  = ((hmac[offset] & 0x7f) << 24)
              | (hmac[offset + 1] << 16)
              | (hmac[offset + 2] << 8)
              |  hmac[offset + 3];
  return String(code % Math.pow(10, DIGITS)).padStart(DIGITS, "0");
}

// ── TOTP ──────────────────────────────────────────────────────────────────────

/** Generate a random base32 TOTP secret. */
export function generateTotpSecret() {
  return base32Encode(randomBytes(20));
}

/**
 * Build the otpauth:// URI for QR-code enrollment.
 * @param {string} secret
 * @param {string} label   e.g. "Pulsar:user@example.com"
 */
export function getTotpUri(secret, label) {
  const params = new URLSearchParams({ secret, issuer: "Pulsar", digits: DIGITS, period: STEP });
  return `otpauth://totp/${encodeURIComponent(label)}?${params}`;
}

/**
 * Verify a TOTP code (accepts ±TOLERANCE windows for clock drift).
 * @param {string} secret
 * @param {string} code     6-digit string
 */
export function verifyTotp(secret, code) {
  const counter = Math.floor(Date.now() / 1000 / STEP);
  for (let delta = -TOLERANCE; delta <= TOLERANCE; delta++) {
    if (hotp(secret, counter + delta) === code) return true;
  }
  return false;
}
