/**
 * apiKeys.test.js — Unit tests for API Key Management (#315).
 *
 * Run with: node --test src/services/apiKeys.test.js
 *
 * Tests the pure logic in the service (key generation, hashing, scope
 * validation, rate limiting, masking) without a live database.
 */

import { describe, it } from "node:test";
import assert from "node:assert/strict";

import {
  generateApiKey,
  hashApiKey,
  validateScopes,
  hasScope,
  checkRateLimit,
  VALID_SCOPES,
} from "./apiKeys.js";

// ── Key generation ────────────────────────────────────────────────────────────

describe("generateApiKey", () => {
  it("returns a plaintext key starting with psk_", () => {
    const { plaintext } = generateApiKey();
    assert.ok(plaintext.startsWith("psk_"), `expected prefix, got ${plaintext}`);
  });

  it("returns a hash that is a 64-char hex string (SHA-256)", () => {
    const { hash } = generateApiKey();
    assert.match(hash, /^[0-9a-f]{64}$/);
  });

  it("returns a unique key on every call", () => {
    const a = generateApiKey();
    const b = generateApiKey();
    assert.notStrictEqual(a.plaintext, b.plaintext);
    assert.notStrictEqual(a.hash, b.hash);
  });

  it("prefix is the first 8 chars of the plaintext", () => {
    const { plaintext, prefix } = generateApiKey();
    assert.strictEqual(prefix, plaintext.slice(0, 8));
  });
});

// ── Hashing ───────────────────────────────────────────────────────────────────

describe("hashApiKey", () => {
  it("produces the same hash for the same key", () => {
    const { plaintext, hash } = generateApiKey();
    assert.strictEqual(hashApiKey(plaintext), hash);
  });

  it("produces different hashes for different keys", () => {
    const a = generateApiKey();
    const b = generateApiKey();
    assert.notStrictEqual(hashApiKey(a.plaintext), hashApiKey(b.plaintext));
  });
});

// ── Scope validation ──────────────────────────────────────────────────────────

describe("validateScopes", () => {
  it("returns valid=true for all known scopes", () => {
    const { valid } = validateScopes(VALID_SCOPES);
    assert.strictEqual(valid, true);
  });

  it("returns valid=false and lists unknown scopes", () => {
    const { valid, unknown } = validateScopes(["payments:read", "fly:rockets"]);
    assert.strictEqual(valid, false);
    assert.deepStrictEqual(unknown, ["fly:rockets"]);
  });

  it("accepts an empty array as valid (no scopes granted)", () => {
    const { valid } = validateScopes([]);
    assert.strictEqual(valid, true);
  });
});

// ── Scope checking ────────────────────────────────────────────────────────────

describe("hasScope", () => {
  it("returns true when the required scope is present", () => {
    assert.strictEqual(hasScope("payments:read,merchants:read", "payments:read"), true);
  });

  it("returns false when the required scope is absent", () => {
    assert.strictEqual(hasScope("payments:read", "payments:write"), false);
  });

  it("returns true for any scope when admin is granted", () => {
    assert.strictEqual(hasScope("admin", "payments:write"), true);
    assert.strictEqual(hasScope("admin", "refunds:read"), true);
  });

  it("is case-sensitive", () => {
    assert.strictEqual(hasScope("Payments:Read", "payments:read"), false);
  });

  it("handles extra whitespace around scopes", () => {
    assert.strictEqual(hasScope("payments:read , merchants:read", "merchants:read"), true);
  });
});

// ── Rate limiting ─────────────────────────────────────────────────────────────

describe("checkRateLimit", () => {
  it("allows requests below the limit", () => {
    const id = `test-key-${Date.now()}-a`;
    const result = checkRateLimit(id, 10);
    assert.strictEqual(result.allowed, true);
    assert.strictEqual(result.remaining, 9);
  });

  it("blocks once limit is exceeded", () => {
    const id = `test-key-${Date.now()}-b`;
    // Exhaust the limit.
    for (let i = 0; i < 3; i++) checkRateLimit(id, 3);
    // Next call should be blocked.
    const { allowed, remaining } = checkRateLimit(id, 3);
    assert.strictEqual(allowed, false);
    assert.strictEqual(remaining, 0);
  });

  it("provides a resetAt timestamp in the future", () => {
    const id = `test-key-${Date.now()}-c`;
    const { resetAt } = checkRateLimit(id, 100);
    assert.ok(resetAt > Date.now(), "resetAt should be in the future");
  });

  it("uses separate buckets per key ID", () => {
    const idA = `test-key-${Date.now()}-d1`;
    const idB = `test-key-${Date.now()}-d2`;
    // Exhaust key A.
    for (let i = 0; i < 2; i++) checkRateLimit(idA, 2);
    checkRateLimit(idA, 2); // blocked
    // Key B should be unaffected.
    const { allowed } = checkRateLimit(idB, 2);
    assert.strictEqual(allowed, true);
  });
});
