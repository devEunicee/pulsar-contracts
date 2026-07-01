/**
 * cors.test.js — Unit tests for the CORS middleware (#317).
 *
 * Run with: node --test src/middleware/cors.test.js
 */

import { describe, it, before, after } from "node:test";
import assert from "node:assert/strict";

// ── Helpers ───────────────────────────────────────────────────────────────────

/** Build a minimal mock Express request. */
function mockReq({ method = "GET", origin } = {}) {
  return {
    method,
    headers: origin ? { origin } : {},
  };
}

/** Build a minimal mock Express response that captures set headers. */
function mockRes() {
  const headers = {};
  let statusCode = null;
  let ended = false;
  return {
    headers,
    statusCode,
    ended,
    setHeader(name, value) {
      headers[name] = value;
    },
    writeHead(code) {
      this.statusCode = code;
    },
    end() {
      this.ended = true;
    },
  };
}

// ── Tests ─────────────────────────────────────────────────────────────────────

describe("corsMiddleware — default (localhost) configuration", () => {
  let corsMiddleware;
  let corsConfig;

  before(async () => {
    // Ensure env vars are in default state.
    delete process.env.CORS_ORIGINS;
    delete process.env.CORS_METHODS;
    delete process.env.CORS_ALLOWED_HEADERS;
    delete process.env.CORS_EXPOSED_HEADERS;
    delete process.env.CORS_MAX_AGE;
    delete process.env.CORS_CREDENTIALS;

    // Dynamic import so env vars are read at module load time.
    const mod = await import("./cors.js?v=1");
    corsMiddleware = mod.corsMiddleware;
    corsConfig = mod.corsConfig;
  });

  it("exports corsConfig with expected defaults", () => {
    assert.ok(Array.isArray(corsConfig.allowedOrigins));
    assert.ok(corsConfig.allowedOrigins.includes("http://localhost:5173"));
    assert.ok(corsConfig.allowedOrigins.includes("http://localhost:3000"));
    assert.strictEqual(corsConfig.credentialsEnabled, true);
    assert.strictEqual(corsConfig.maxAge, 86400);
    assert.ok(corsConfig.allowedMethods.includes("GET"));
    assert.ok(corsConfig.allowedMethods.includes("POST"));
  });

  it("allows a known origin and sets ACAO header", (t, done) => {
    const req = mockReq({ origin: "http://localhost:5173" });
    const res = mockRes();
    corsMiddleware(req, res, () => {
      assert.strictEqual(
        res.headers["Access-Control-Allow-Origin"],
        "http://localhost:5173"
      );
      assert.strictEqual(
        res.headers["Access-Control-Allow-Credentials"],
        "true"
      );
      done();
    });
  });

  it("does NOT set ACAO header for an unknown origin", (t, done) => {
    const req = mockReq({ origin: "https://evil.example.com" });
    const res = mockRes();
    corsMiddleware(req, res, () => {
      assert.strictEqual(
        res.headers["Access-Control-Allow-Origin"],
        undefined
      );
      done();
    });
  });

  it("handles requests with no Origin header (same-origin / curl)", (t, done) => {
    const req = mockReq(); // no origin
    const res = mockRes();
    corsMiddleware(req, res, () => {
      // Should pass through without error.
      done();
    });
  });

  it("responds 204 to OPTIONS preflight for an allowed origin", () => {
    const req = mockReq({ method: "OPTIONS", origin: "http://localhost:3000" });
    const res = mockRes();
    let nextCalled = false;
    corsMiddleware(req, res, () => {
      nextCalled = true;
    });

    assert.strictEqual(nextCalled, false, "next() must not be called for OPTIONS");
    assert.strictEqual(res.statusCode, 204);
    assert.strictEqual(res.ended, true);
    assert.ok(
      res.headers["Access-Control-Allow-Methods"],
      "Allow-Methods header must be present"
    );
    assert.ok(
      res.headers["Access-Control-Allow-Headers"],
      "Allow-Headers header must be present"
    );
    assert.ok(
      res.headers["Access-Control-Max-Age"],
      "Max-Age header must be present"
    );
  });

  it("responds 204 to OPTIONS but sets no ACAO for unknown origin", () => {
    const req = mockReq({ method: "OPTIONS", origin: "https://other.com" });
    const res = mockRes();
    corsMiddleware(req, res, () => {});

    assert.strictEqual(res.statusCode, 204);
    assert.strictEqual(res.headers["Access-Control-Allow-Origin"], undefined);
  });
});

describe("corsMiddleware — wildcard configuration", () => {
  let corsMiddleware;

  before(async () => {
    process.env.CORS_ORIGINS = "*";
    process.env.CORS_CREDENTIALS = "false";
    const mod = await import("./cors.js?v=2");
    corsMiddleware = mod.corsMiddleware;
  });

  after(() => {
    delete process.env.CORS_ORIGINS;
    delete process.env.CORS_CREDENTIALS;
  });

  it("reflects back any origin when CORS_ORIGINS=*", (t, done) => {
    const req = mockReq({ origin: "https://any.example.com" });
    const res = mockRes();
    corsMiddleware(req, res, () => {
      assert.strictEqual(
        res.headers["Access-Control-Allow-Origin"],
        "https://any.example.com"
      );
      done();
    });
  });

  it("does NOT set credentials header when CORS_CREDENTIALS=false", (t, done) => {
    const req = mockReq({ origin: "https://any.example.com" });
    const res = mockRes();
    corsMiddleware(req, res, () => {
      assert.strictEqual(
        res.headers["Access-Control-Allow-Credentials"],
        undefined
      );
      done();
    });
  });
});
