/**
 * Two-Factor Authentication service (Issue #313)
 *
 * Covers:
 *  - TOTP enrollment / verification
 *  - SMS OTP as backup method (stub — wire in real SMS provider via env)
 *  - Backup codes (10 one-time codes)
 *  - Device trust / remember-me for 30 days
 *  - Rate limiting on verification attempts (5 tries per 15 min per user)
 *  - Recovery flow (consume backup code)
 */

import { randomBytes, createHash } from "node:crypto";
import { generateTotpSecret, verifyTotp } from "./totp.js";

// ── in-memory stores (replace with DB/Redis in production) ────────────────────

/** userId → { secret, method, backupCodes: string[], enabled } */
const userTfa = new Map();

/** deviceToken → { userId, expiresAt } */
const trustedDevices = new Map();

/** userId → { attempts: number, windowStart: number } */
const rateLimitBuckets = new Map();

/** userId → pendingSmsCode */
const smsCodes = new Map();

// ── constants ─────────────────────────────────────────────────────────────────

const BACKUP_CODE_COUNT    = 10;
const DEVICE_TRUST_TTL     = 30 * 24 * 60 * 60 * 1000; // 30 days (ms)
const RATE_LIMIT_WINDOW    = 15 * 60 * 1000;            // 15 min (ms)
const RATE_LIMIT_MAX       = 5;
const SMS_CODE_TTL         = 10 * 60 * 1000;            // 10 min (ms)

export class TfaError extends Error {
  constructor(code, message) {
    super(message ?? code);
    this.code   = code;
    this.status = code === "TOO_MANY_ATTEMPTS" ? 429 : 401;
  }
}

// ── rate limiter ──────────────────────────────────────────────────────────────

function checkRateLimit(userId) {
  const now    = Date.now();
  const bucket = rateLimitBuckets.get(userId) ?? { attempts: 0, windowStart: now };
  if (now - bucket.windowStart > RATE_LIMIT_WINDOW) {
    bucket.attempts    = 0;
    bucket.windowStart = now;
  }
  if (bucket.attempts >= RATE_LIMIT_MAX) throw new TfaError("TOO_MANY_ATTEMPTS");
  bucket.attempts++;
  rateLimitBuckets.set(userId, bucket);
}

// ── backup codes ──────────────────────────────────────────────────────────────

function generateBackupCodes() {
  return Array.from({ length: BACKUP_CODE_COUNT }, () =>
    randomBytes(4).toString("hex").toUpperCase()
  );
}

function hashCode(code) {
  return createHash("sha256").update(code).digest("hex");
}

// ── enrollment ────────────────────────────────────────────────────────────────

/**
 * Enroll a user in TOTP 2FA.
 * Returns { secret, uri, backupCodes } — caller must persist these.
 */
export function enrollTotp(userId) {
  const secret      = generateTotpSecret();
  const backupCodes = generateBackupCodes();
  userTfa.set(userId, {
    secret,
    method:      "totp",
    backupCodes: backupCodes.map(hashCode), // store hashed
    enabled:     false, // enable after first successful verify
  });
  // Return plaintext codes once — user must save them
  return { secret, backupCodes };
}

/**
 * Activate TOTP after the user confirms the first code.
 */
export function activateTotp(userId, code) {
  const rec = userTfa.get(userId);
  if (!rec) throw new TfaError("NOT_ENROLLED", "User not enrolled in 2FA");
  if (!verifyTotp(rec.secret, code)) throw new TfaError("INVALID_CODE");
  rec.enabled = true;
}

// ── SMS OTP ───────────────────────────────────────────────────────────────────

/**
 * Send an SMS OTP to the user's phone (stub).
 * Replace body with real provider call (Twilio, AWS SNS, etc.).
 */
export async function sendSmsOtp(userId, phoneNumber) {
  const code    = String(Math.floor(100000 + Math.random() * 900000));
  const expires = Date.now() + SMS_CODE_TTL;
  smsCodes.set(userId, { code, expires, phoneNumber });
  // TODO: await smsProvider.send(phoneNumber, `Your Pulsar code: ${code}`)
  console.log(`[SMS stub] Code for ${userId}: ${code}`);
  return { sent: true };
}

/**
 * Verify an SMS OTP code.
 */
export function verifySmsOtp(userId, code) {
  checkRateLimit(userId);
  const entry = smsCodes.get(userId);
  if (!entry || Date.now() > entry.expires) throw new TfaError("CODE_EXPIRED");
  if (entry.code !== code) throw new TfaError("INVALID_CODE");
  smsCodes.delete(userId);
}

// ── TOTP verification ─────────────────────────────────────────────────────────

export function verifyTotpCode(userId, code) {
  checkRateLimit(userId);
  const rec = userTfa.get(userId);
  if (!rec?.enabled) throw new TfaError("NOT_ENABLED", "2FA not enabled for this user");
  if (!verifyTotp(rec.secret, code)) throw new TfaError("INVALID_CODE");
}

// ── backup code recovery ──────────────────────────────────────────────────────

/**
 * Consume a backup code. Each code can only be used once.
 */
export function consumeBackupCode(userId, code) {
  checkRateLimit(userId);
  const rec = userTfa.get(userId);
  if (!rec?.enabled) throw new TfaError("NOT_ENABLED");
  const hashed = hashCode(code.toUpperCase());
  const idx    = rec.backupCodes.indexOf(hashed);
  if (idx === -1) throw new TfaError("INVALID_CODE", "Invalid or already-used backup code");
  rec.backupCodes.splice(idx, 1); // consume
}

// ── device trust ──────────────────────────────────────────────────────────────

/**
 * Issue a device-trust token valid for 30 days.
 * @returns {string} opaque device token
 */
export function trustDevice(userId) {
  const token   = randomBytes(32).toString("hex");
  const expires = Date.now() + DEVICE_TRUST_TTL;
  trustedDevices.set(token, { userId, expiresAt: expires });
  return token;
}

/**
 * Returns true if the device token is valid and not expired.
 */
export function isDeviceTrusted(userId, deviceToken) {
  const entry = trustedDevices.get(deviceToken);
  if (!entry) return false;
  if (Date.now() > entry.expiresAt) { trustedDevices.delete(deviceToken); return false; }
  return entry.userId === userId;
}

// ── admin enforcement ─────────────────────────────────────────────────────────

/**
 * Returns true if the user is required to complete 2FA.
 * Admins are always required; other roles only if enrolled.
 */
export function requires2FA(role, userId) {
  if (role === "admin") return true;
  return userTfa.get(userId)?.enabled ?? false;
}
