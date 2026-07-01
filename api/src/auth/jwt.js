/**
 * JWT Authentication Service (Issue #309)
 *
 * Signs with RS256 (RSA private key). Keys are loaded from environment:
 *   JWT_PRIVATE_KEY  — PEM-encoded RSA private key (PKCS#8)
 *   JWT_PUBLIC_KEY   — PEM-encoded RSA public key
 *
 * Access tokens expire in 1 hour; refresh tokens expire in 7 days.
 * Refresh token rotation: each use issues a new refresh token and
 * invalidates the old one via an in-memory store (swap for Redis/DB in prod).
 */

import { createSign, createVerify } from "node:crypto";

const ACCESS_TTL  = 60 * 60;          // 1 hour  (seconds)
const REFRESH_TTL = 7 * 24 * 60 * 60; // 7 days  (seconds)

// ── helpers ───────────────────────────────────────────────────────────────────

function b64url(buf) {
  return buf.toString("base64")
    .replace(/\+/g, "-").replace(/\//g, "_").replace(/=/g, "");
}

function encode(obj) {
  return b64url(Buffer.from(JSON.stringify(obj)));
}

function sign(payload, privateKey) {
  const header  = encode({ alg: "RS256", typ: "JWT" });
  const body    = encode(payload);
  const signing = `${header}.${body}`;
  const signer  = createSign("RSA-SHA256");
  signer.update(signing);
  const sig = b64url(signer.sign(privateKey));
  return `${signing}.${sig}`;
}

function verify(token, publicKey) {
  const parts = token.split(".");
  if (parts.length !== 3) throw new JwtError("MALFORMED_TOKEN");
  const [header, body, sig] = parts;
  const verifier = createVerify("RSA-SHA256");
  verifier.update(`${header}.${body}`);
  const ok = verifier.verify(publicKey, sig, "base64");
  if (!ok) throw new JwtError("INVALID_SIGNATURE");
  const payload = JSON.parse(Buffer.from(body, "base64").toString());
  if (payload.exp && Math.floor(Date.now() / 1000) > payload.exp) {
    throw new JwtError("TOKEN_EXPIRED");
  }
  return payload;
}

// ── refresh token store (replace with Redis/DB in production) ─────────────────
const refreshStore = new Map(); // token → { userId, address, role, exp }

// ── exported service ──────────────────────────────────────────────────────────

export class JwtError extends Error {
  constructor(code) { super(code); this.code = code; this.status = 401; }
}

/**
 * Generate an access + refresh token pair.
 * @param {{ userId: string, address: string, role: string }} user
 * @param {{ privateKey: string }} keys
 */
export function issueTokens(user, { privateKey }) {
  const now = Math.floor(Date.now() / 1000);
  const accessPayload = {
    sub:     user.userId,
    address: user.address,
    role:    user.role,
    iat:     now,
    exp:     now + ACCESS_TTL,
  };
  const accessToken = sign(accessPayload, privateKey);

  // Refresh token: opaque random bytes stored server-side
  const refreshToken = b64url(
    Buffer.from(crypto.randomUUID().replace(/-/g, ""), "hex")
  );
  refreshStore.set(refreshToken, {
    userId:  user.userId,
    address: user.address,
    role:    user.role,
    exp:     now + REFRESH_TTL,
  });

  return { accessToken, refreshToken, expiresIn: ACCESS_TTL };
}

/**
 * Rotate a refresh token — invalidate old, issue new pair.
 * @param {string} oldRefreshToken
 * @param {{ privateKey: string }} keys
 */
export function rotateRefreshToken(oldRefreshToken, { privateKey }) {
  const entry = refreshStore.get(oldRefreshToken);
  if (!entry) throw new JwtError("INVALID_REFRESH_TOKEN");
  if (Math.floor(Date.now() / 1000) > entry.exp) {
    refreshStore.delete(oldRefreshToken);
    throw new JwtError("REFRESH_TOKEN_EXPIRED");
  }
  refreshStore.delete(oldRefreshToken);
  return issueTokens({ userId: entry.userId, address: entry.address, role: entry.role }, { privateKey });
}

/**
 * Validate an access token and return its payload.
 * @param {string} token
 * @param {{ publicKey: string }} keys
 */
export function validateAccessToken(token, { publicKey }) {
  return verify(token, publicKey);
}

/**
 * Revoke a refresh token (logout).
 */
export function revokeRefreshToken(token) {
  refreshStore.delete(token);
}
