/**
 * oauth.test.js — Unit tests for OAuth 2.0 service (#310).
 *
 * Run with: node --test src/services/oauth.test.js
 *
 * Tests pure logic (token generation, scope resolution, rate limiting,
 * PKCE challenge computation, OAuthError) without a live database.
 */

import { describe, it } from "node:test";
import assert from "node:assert/strict";
import crypto from "crypto";

import {
  hashToken,
  resolveScopes,
  hasScope,
  checkClientRateLimit,
  VALID_SCOPES,
  OAuthError,
} from "./oauth.js";

// ── Token helpers ─────────────────────────────────────────────────────────────

describe("hashToken", () => {
  it("returns a 64-char hex SHA-256 digest", () => {
    const hash = hashToken("at_abc123");
    assert.match(hash, /^[0-9a-f]{64}$/);
  });

  it("is deterministic for the same input", () => {
    assert.strictEqual(hashToken("at_abc"), hashToken("at_abc"));
  });

  it("produces unique hashes for different tokens", () => {
    assert.notStrictEqual(hashToken("at_aaa"), hashToken("at_bbb"));
  });
});

// ── Scope resolution ──────────────────────────────────────────────────────────

describe("resolveScopes", () => {
  it("returns valid=true when requested scopes are a subset of allowed", () => {
    const { valid, scopes } = resolveScopes(
      "payments:read merchants:read",
      "payments:read payments:write merchants:read"
    );
    assert.strictEqual(valid, true);
    assert.deepStrictEqual(scopes, ["payments:read", "merchants:read"]);
  });

  it("returns valid=false when a requested scope exceeds the allowed set", () => {
    const { valid, unauthorized } = resolveScopes(
      "payments:read admin",
      "payments:read"
    );
    assert.strictEqual(valid, false);
    assert.ok(unauthorized.includes("admin"));
  });

  it("returns valid=false for completely unknown scopes", () => {
    const { valid, unauthorized } = resolveScopes(
      "fly:rockets",
      "payments:read"
    );
    assert.strictEqual(valid, false);
    assert.ok(unauthorized.includes("fly:rockets"));
  });

  it("handles extra whitespace", () => {
    const { valid } = resolveScopes(
      "  payments:read  ",
      "payments:read payments:write"
    );
    assert.strictEqual(valid, true);
  });

  it("treats empty requested scope as valid", () => {
    const { valid, scopes } = resolveScopes("", "payments:read");
    assert.strictEqual(valid, true);
    assert.deepStrictEqual(scopes, []);
  });
});

// ── hasScope ──────────────────────────────────────────────────────────────────

describe("hasScope", () => {
  it("returns true when the required scope is present", () => {
    assert.strictEqual(hasScope("payments:read merchants:read", "payments:read"), true);
  });

  it("returns false when the required scope is absent", () => {
    assert.strictEqual(hasScope("payments:read", "payments:write"), false);
  });

  it("admin scope grants access to any scope", () => {
    assert.strictEqual(hasScope("admin", "payments:write"), true);
    assert.strictEqual(hasScope("admin", "refunds:read"), true);
  });
});

// ── Rate limiting ─────────────────────────────────────────────────────────────

describe("checkClientRateLimit", () => {
  it("allows requests below the limit", () => {
    const { allowed, remaining } = checkClientRateLimit(`oauth-client-${Date.now()}-a`, 10);
    assert.strictEqual(allowed, true);
    assert.strictEqual(remaining, 9);
  });

  it("blocks once limit is exceeded", () => {
    const id = `oauth-client-${Date.now()}-b`;
    for (let i = 0; i < 5; i++) checkClientRateLimit(id, 5);
    const { allowed } = checkClientRateLimit(id, 5);
    assert.strictEqual(allowed, false);
  });

  it("uses separate buckets per client", () => {
    const a = `oauth-client-${Date.now()}-c1`;
    const b = `oauth-client-${Date.now()}-c2`;
    for (let i = 0; i < 3; i++) checkClientRateLimit(a, 3);
    checkClientRateLimit(a, 3); // blocked
    const { allowed } = checkClientRateLimit(b, 3);
    assert.strictEqual(allowed, true, "different client should have its own bucket");
  });
});

// ── VALID_SCOPES ──────────────────────────────────────────────────────────────

describe("VALID_SCOPES", () => {
  it("includes expected values", () => {
    const required = [
      "payments:read",
      "payments:write",
      "merchants:read",
      "merchants:write",
      "refunds:read",
      "refunds:write",
      "profile",
      "admin",
    ];
    for (const scope of required) {
      assert.ok(VALID_SCOPES.includes(scope), `Expected VALID_SCOPES to contain '${scope}'`);
    }
  });

  it("is frozen (immutable)", () => {
    assert.strictEqual(Object.isFrozen(VALID_SCOPES), true);
  });
});

// ── OAuthError ────────────────────────────────────────────────────────────────

describe("OAuthError", () => {
  it("stores error code and message", () => {
    const err = new OAuthError("invalid_grant", "The code has expired");
    assert.strictEqual(err.code, "invalid_grant");
    assert.strictEqual(err.message, "The code has expired");
    assert.strictEqual(err.name, "OAuthError");
    assert.ok(err instanceof Error);
  });

  it("can be caught as an Error instance", () => {
    let caught = null;
    try {
      throw new OAuthError("invalid_client", "Bad creds");
    } catch (e) {
      caught = e;
    }
    assert.ok(caught instanceof Error);
    assert.ok(caught instanceof OAuthError);
  });
});

// ── PKCE S256 challenge verification ──────────────────────────────────────────

describe("PKCE S256 challenge", () => {
  it("base64url(sha256(verifier)) matches challenge", () => {
    const verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
    const challenge = crypto
      .createHash("sha256")
      .update(verifier)
      .digest("base64url");
    // RFC 7636 test vector.
    assert.strictEqual(challenge, "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM");
  });
});
