/**
 * JWT validation middleware (Issue #309)
 *
 * Reads the Bearer token from Authorization header, validates it,
 * and attaches the decoded payload to req.user.
 *
 * Usage:
 *   import { jwtMiddleware } from './auth/jwtMiddleware.js';
 *   app.use(jwtMiddleware);
 */

import { validateAccessToken, JwtError } from "./jwt.js";

const PUBLIC_KEY = process.env.JWT_PUBLIC_KEY ?? "";

export function jwtMiddleware(req, res, next) {
  const authHeader = req.headers["authorization"] ?? "";
  if (!authHeader.startsWith("Bearer ")) {
    return res.status(401).json({ error: { code: "MISSING_TOKEN", message: "Authorization header required" } });
  }
  const token = authHeader.slice(7);
  try {
    req.user = validateAccessToken(token, { publicKey: PUBLIC_KEY });
    next();
  } catch (err) {
    if (err instanceof JwtError) {
      return res.status(401).json({ error: { code: err.code, message: err.message } });
    }
    next(err);
  }
}

/** Middleware variant that allows unauthenticated requests (optional auth). */
export function optionalJwtMiddleware(req, res, next) {
  const authHeader = req.headers["authorization"] ?? "";
  if (!authHeader.startsWith("Bearer ")) return next();
  const token = authHeader.slice(7);
  try {
    req.user = validateAccessToken(token, { publicKey: PUBLIC_KEY });
  } catch {
    // ignore — leave req.user undefined
  }
  next();
}
