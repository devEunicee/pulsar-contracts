/**
 * oauthAuth.js — Express middleware for OAuth 2.0 bearer token authentication (#310).
 *
 * Usage:
 *   import { requireBearerToken, requireOAuthScope } from "./middleware/oauthAuth.js";
 *
 *   // Require any valid bearer token:
 *   router.get("/protected", requireBearerToken(), handler);
 *
 *   // Require a specific scope:
 *   router.post("/payments", requireBearerToken(), requireOAuthScope("payments:write"), handler);
 */

import { OAuthService, OAuthError, hasScope } from "../services/oauth.js";

// Lazily-initialised singleton.
let _service = null;
function getService() {
  if (!_service) _service = new OAuthService();
  return _service;
}

/**
 * Parse a Bearer token from the Authorization header.
 * @param {import("express").Request} req
 * @returns {string|null}
 */
function extractBearerToken(req) {
  const authHeader = req.headers.authorization;
  if (!authHeader) return null;
  const parts = authHeader.split(" ");
  if (parts.length !== 2 || parts[0].toLowerCase() !== "bearer") return null;
  return parts[1];
}

/**
 * Authenticate the request using the Authorization: Bearer <token> header.
 * On success, attaches `req.oauthToken` metadata and calls next().
 *
 * @returns {import("express").RequestHandler}
 */
export function requireBearerToken() {
  return async (req, res, next) => {
    const token = extractBearerToken(req);
    if (!token) {
      res.setHeader("WWW-Authenticate", 'Bearer realm="pulsar"');
      return res.status(401).json({
        error: "invalid_token",
        error_description: "Bearer token required",
      });
    }

    const service = getService();
    let metadata;
    try {
      metadata = await service.validateAccessToken(token);
    } catch (_) {
      metadata = null;
    }

    if (!metadata) {
      res.setHeader(
        "WWW-Authenticate",
        'Bearer realm="pulsar", error="invalid_token", error_description="Token is invalid or expired"'
      );
      return res.status(401).json({
        error: "invalid_token",
        error_description: "Access token is invalid, expired, or revoked",
      });
    }

    req.oauthToken = metadata;
    next();
  };
}

/**
 * Enforce a scope requirement on an already-authenticated OAuth request.
 * Must be called after requireBearerToken().
 *
 * @param {string} scope  e.g. "payments:write"
 * @returns {import("express").RequestHandler}
 */
export function requireOAuthScope(scope) {
  return (req, res, next) => {
    if (!req.oauthToken) {
      return res.status(401).json({
        error: "invalid_token",
        error_description: "OAuth token authentication required",
      });
    }
    if (!hasScope(req.oauthToken.scopes, scope)) {
      return res.status(403).json({
        error: "insufficient_scope",
        error_description: `Token does not have the required '${scope}' scope`,
        scope: scope,
      });
    }
    next();
  };
}

/**
 * Express error handler for OAuthError instances.
 * Returns RFC 6749-compliant JSON error responses.
 *
 * @type {import("express").ErrorRequestHandler}
 */
export function oauthErrorHandler(err, _req, res, next) {
  if (err instanceof OAuthError) {
    const status = oauthErrorStatus(err.code);
    return res.status(status).json({
      error: err.code,
      error_description: err.message,
    });
  }
  next(err);
}

/** Map RFC 6749 error codes to HTTP status codes. */
function oauthErrorStatus(code) {
  switch (code) {
    case "invalid_client":
      return 401;
    case "invalid_grant":
    case "invalid_request":
    case "invalid_scope":
    case "unsupported_grant_type":
    case "unsupported_response_type":
      return 400;
    case "unauthorized_client":
      return 403;
    case "slow_down":
      return 429;
    default:
      return 400;
  }
}
