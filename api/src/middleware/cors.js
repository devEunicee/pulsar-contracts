/**
 * cors.js — CORS (Cross-Origin Resource Sharing) middleware for the Pulsar API.
 *
 * Resolves #317: implement proper CORS configuration for cross-origin requests
 * from the frontend and third-party applications.
 *
 * Configuration is driven entirely by environment variables so that each
 * deployment environment (local, staging, production) can supply its own
 * origin allow-list without a code change.
 *
 * Environment variables
 * ─────────────────────
 * CORS_ORIGINS          Comma-separated list of allowed origins.
 *                       Use "*" to allow all origins (dev only).
 *                       Default: "http://localhost:5173,http://localhost:3000"
 * CORS_METHODS          Comma-separated HTTP methods.
 *                       Default: "GET,POST,PUT,PATCH,DELETE,OPTIONS"
 * CORS_ALLOWED_HEADERS  Comma-separated request headers the client may send.
 *                       Default: "Content-Type,Authorization,X-Api-Key,X-Idempotency-Key"
 * CORS_EXPOSED_HEADERS  Comma-separated response headers the browser may read.
 *                       Default: "X-Request-Id,X-Rate-Limit-Limit,X-Rate-Limit-Remaining,X-Rate-Limit-Reset"
 * CORS_MAX_AGE          Preflight cache duration in seconds.
 *                       Default: "86400" (24 hours)
 * CORS_CREDENTIALS      Whether to allow credentials (cookies / auth headers).
 *                       "true" | "false". Default: "true"
 *
 * Usage
 * ─────
 * import { corsMiddleware } from "./middleware/cors.js";
 * app.use(corsMiddleware);
 */

"use strict";

// ── Configuration ─────────────────────────────────────────────────────────────

/**
 * Parse a comma-separated environment variable into a trimmed string array,
 * filtering out empty segments.
 *
 * @param {string} envVar - Environment variable value.
 * @param {string} fallback - Default value used when the variable is absent.
 * @returns {string[]}
 */
function parseList(envVar, fallback) {
  const raw = envVar != null ? envVar : fallback;
  return raw
    .split(",")
    .map((s) => s.trim())
    .filter(Boolean);
}

const allowedOrigins = parseList(
  process.env.CORS_ORIGINS,
  "http://localhost:5173,http://localhost:3000"
);

const allowedMethods = parseList(
  process.env.CORS_METHODS,
  "GET,POST,PUT,PATCH,DELETE,OPTIONS"
);

const allowedHeaders = parseList(
  process.env.CORS_ALLOWED_HEADERS,
  "Content-Type,Authorization,X-Api-Key,X-Idempotency-Key"
);

const exposedHeaders = parseList(
  process.env.CORS_EXPOSED_HEADERS,
  "X-Request-Id,X-Rate-Limit-Limit,X-Rate-Limit-Remaining,X-Rate-Limit-Reset"
);

const maxAge = parseInt(process.env.CORS_MAX_AGE ?? "86400", 10);
const credentialsEnabled = (process.env.CORS_CREDENTIALS ?? "true") === "true";

// Wildcard shorthand: a single "*" entry means every origin is allowed.
const allowAll = allowedOrigins.length === 1 && allowedOrigins[0] === "*";

// ── Origin validation ─────────────────────────────────────────────────────────

/**
 * Determine whether an incoming Origin header value is permitted.
 *
 * @param {string | undefined} origin - The request Origin header value.
 * @returns {string | false} The origin to echo back, or false to deny.
 */
function resolveOrigin(origin) {
  // Same-origin / non-browser requests have no Origin header — always allow.
  if (!origin) return "*";

  if (allowAll) return origin;

  if (allowedOrigins.includes(origin)) return origin;

  return false;
}

// ── CORS middleware ───────────────────────────────────────────────────────────

/**
 * Express middleware that applies CORS headers to every response and handles
 * OPTIONS preflight requests.
 *
 * @type {import("express").RequestHandler}
 */
export function corsMiddleware(req, res, next) {
  const requestOrigin = req.headers.origin;
  const resolvedOrigin = resolveOrigin(requestOrigin);

  if (resolvedOrigin !== false) {
    res.setHeader("Access-Control-Allow-Origin", resolvedOrigin);

    if (credentialsEnabled && resolvedOrigin !== "*") {
      // Credentials require an explicit (non-wildcard) origin.
      res.setHeader("Access-Control-Allow-Credentials", "true");
    }

    if (exposedHeaders.length > 0) {
      res.setHeader("Access-Control-Expose-Headers", exposedHeaders.join(", "));
    }
  }

  // ── Preflight (OPTIONS) ──────────────────────────────────────────────────

  if (req.method === "OPTIONS") {
    if (resolvedOrigin !== false) {
      res.setHeader("Access-Control-Allow-Methods", allowedMethods.join(", "));
      res.setHeader("Access-Control-Allow-Headers", allowedHeaders.join(", "));
      res.setHeader("Access-Control-Max-Age", String(maxAge));
    }

    // Preflight responses must be 204 (no content) with no body.
    res.writeHead(204);
    res.end();
    return;
  }

  next();
}

// ── Exported configuration snapshot (useful for tests & health endpoints) ────

/**
 * Read-only view of the resolved CORS configuration.
 */
export const corsConfig = Object.freeze({
  allowedOrigins,
  allowAll,
  allowedMethods,
  allowedHeaders,
  exposedHeaders,
  maxAge,
  credentialsEnabled,
});
