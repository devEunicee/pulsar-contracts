/**
 * JWT service tests (Issue #309)
 * Run with: node --test api/src/auth/jwt.test.js
 *
 * Generates a throwaway RSA key-pair so no environment variables are needed.
 */

import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { generateKeyPairSync } from "node:crypto";
import {
  issueTokens, rotateRefreshToken, revokeRefreshToken,
  validateAccessToken, JwtError,
} from "./jwt.js";

const { privateKey, publicKey } = generateKeyPairSync("rsa", {
  modulusLength: 2048,
  publicKeyEncoding:  { type: "pkcs1", format: "pem" },
  privateKeyEncoding: { type: "pkcs8", format: "pem" },
});

const user = { userId: "u1", address: "GABC", role: "pulsar_customer" };
const keys = { privateKey, publicKey };

describe("issueTokens", () => {
  it("returns accessToken, refreshToken, expiresIn", () => {
    const { accessToken, refreshToken, expiresIn } = issueTokens(user, keys);
    assert.ok(typeof accessToken  === "string");
    assert.ok(typeof refreshToken === "string");
    assert.equal(expiresIn, 3600);
  });

  it("access token contains correct claims", () => {
    const { accessToken } = issueTokens(user, keys);
    const payload = validateAccessToken(accessToken, keys);
    assert.equal(payload.sub,     "u1");
    assert.equal(payload.address, "GABC");
    assert.equal(payload.role,    "pulsar_customer");
    assert.ok(payload.exp > payload.iat);
  });
});

describe("validateAccessToken", () => {
  it("throws INVALID_SIGNATURE on tampered token", () => {
    const { accessToken } = issueTokens(user, keys);
    const parts = accessToken.split(".");
    parts[1] = Buffer.from(JSON.stringify({ sub: "hacker" })).toString("base64");
    assert.throws(
      () => validateAccessToken(parts.join("."), keys),
      (err) => err.code === "INVALID_SIGNATURE"
    );
  });

  it("throws TOKEN_EXPIRED on expired token", () => {
    // build a token with exp in the past
    const { privateKey: pk } = keys;
    const now = Math.floor(Date.now() / 1000);
    const { issueTokens: _ , ...rest } = await import("./jwt.js");
    // craft payload directly
    const { createSign } = await import("node:crypto");
    const header  = Buffer.from(JSON.stringify({ alg: "RS256", typ: "JWT" })).toString("base64url");
    const body    = Buffer.from(JSON.stringify({ sub: "u1", exp: now - 10 })).toString("base64url");
    const signer  = createSign("RSA-SHA256");
    signer.update(`${header}.${body}`);
    const sig = signer.sign(pk, "base64url");
    assert.throws(
      () => validateAccessToken(`${header}.${body}.${sig}`, keys),
      (err) => err.code === "TOKEN_EXPIRED"
    );
  });
});

describe("rotateRefreshToken", () => {
  it("invalidates old token and issues new pair", () => {
    const { refreshToken: old } = issueTokens(user, keys);
    const next = rotateRefreshToken(old, keys);
    assert.ok(next.accessToken);
    assert.ok(next.refreshToken !== old);
    // old token must be invalid now
    assert.throws(() => rotateRefreshToken(old, keys), (err) => err.code === "INVALID_REFRESH_TOKEN");
  });
});

describe("revokeRefreshToken", () => {
  it("makes the token unusable", () => {
    const { refreshToken } = issueTokens(user, keys);
    revokeRefreshToken(refreshToken);
    assert.throws(
      () => rotateRefreshToken(refreshToken, keys),
      (err) => err.code === "INVALID_REFRESH_TOKEN"
    );
  });
});
