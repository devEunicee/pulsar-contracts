/**
 * apiKeyAuth.js — Express middleware for API key authentication (#315).
 *
 * Validates the X-Api-Key header, checks rate limits, enforces scope
 * requirements, and logs activity for every authenticated request.
 *
 * Usage:
 *   import { requireApiKey, requireScope } from "./middleware/apiKeyAuth.js";
 *
 *   // Require any valid API key:
 *   router.get("/protected", requireApiKey(), handler);
 *
 *   // Require a valid key AND a specific scope:
 *   router.post("/payments", requireApiKey(), requireScope("payments:write"), handler);
 */

import { ApiKeyService, checkRateLimit, hasScope } from "../services/apiKeys.js";

// Lazily-initialised service singleton. Avoids a DB connection at import time.
let _service = null;
function getService() {
  if (!_service) _service = new ApiKeyService();
  return _service;
}

// ── Rate-limit headers ────────────────────────────────────────────────────────

function setRateLimitHeaders(res, rateLimit, remaining, resetAt) {
  res.setHeader("X-Rate-Limit-Limit", String(rateLimit));
  res.setHeader("X-Rate-Limit-Remaining", String(remaining));
  res.setHeader(
    "X-Rate-Limit-Reset",
    String(Math.ceil(resetAt / 1000)) // Unix seconds
  );
}

// ── Middleware ────────────────────────────────────────────────────────────────

/**
 * Authenticate the request using the X-Api-Key header.
 * On success, attaches `req.apiKey` (ApiKeyRecord) and calls next().
 * On failure, responds with 401 or 429.
 *
 * @returns {import("express").RequestHandler}
 */
export function requireApiKey() {
  return async (req, res, next) => {
    const raw = req.headers["x-api-key"];

    if (!raw || typeof raw !== "string") {
      return res.status(401).json({
        error: {
          code: "MissingApiKey",
          message: "X-Api-Key header is required",
        },
      });
    }

    const service = getService();
    const keyRecord = await service.authenticate(raw).catch(() => null);

    if (!keyRecord) {
      return res.status(401).json({
        error: {
          code: "InvalidApiKey",
          message: "API key is invalid, expired, or revoked",
        },
      });
    }

    // ── Rate limiting ──────────────────────────────────────────────────────
    const { allowed, remaining, resetAt } = checkRateLimit(
      keyRecord.id,
      keyRecord.rate_limit
    );

    setRateLimitHeaders(res, keyRecord.rate_limit, remaining, resetAt);

    if (!allowed) {
      return res.status(429).json({
        error: {
          code: "RateLimitExceeded",
          message: "API key rate limit exceeded. Check X-Rate-Limit-Reset.",
          reset_at: new Date(resetAt).toISOString(),
        },
      });
    }

    // Attach key record for downstream handlers.
    req.apiKey = keyRecord;

    // ── Activity logging (fire-and-forget) ─────────────────────────────────
    const startMs = Date.now();
    res.on("finish", () => {
      service
        .logActivity(keyRecord.id, {
          ip: req.ip,
          method: req.method,
          path: req.path,
          statusCode: res.statusCode,
          durationMs: Date.now() - startMs,
        })
        .catch(() => {/* best-effort */});
    });

    next();
  };
}

/**
 * Enforce a scope requirement on an already-authenticated request.
 * Must be used AFTER requireApiKey().
 *
 * @param {string} scope  e.g. "payments:write"
 * @returns {import("express").RequestHandler}
 */
export function requireScope(scope) {
  return (req, res, next) => {
    if (!req.apiKey) {
      return res.status(401).json({
        error: {
          code: "Unauthenticated",
          message: "API key authentication required",
        },
      });
    }

    const scopesStr = Array.isArray(req.apiKey.scopes)
      ? req.apiKey.scopes.join(",")
      : req.apiKey.scopes ?? "";

    if (!hasScope(scopesStr, scope)) {
      return res.status(403).json({
        error: {
          code: "InsufficientScope",
          message: `This API key does not have the '${scope}' scope`,
          required: scope,
          granted: req.apiKey.scopes,
        },
      });
    }

    next();
  };
}
