/**
 * Security headers middleware (#318)
 *
 * Adds the following headers to every response:
 *   Content-Security-Policy
 *   Strict-Transport-Security (HSTS)
 *   X-Frame-Options
 *   X-Content-Type-Options
 *   X-XSS-Protection
 *   Referrer-Policy
 *   Permissions-Policy
 */

const DEFAULT_OPTIONS = {
  /** Only enforce HSTS in production to avoid local dev pain. */
  hsts: process.env.NODE_ENV === "production",
  /** Allow callers to supply a custom CSP directive string. */
  csp: "default-src 'none'; script-src 'self'; connect-src 'self'; img-src 'self'; style-src 'self'; frame-ancestors 'none'",
};

/**
 * @param {typeof DEFAULT_OPTIONS} [options]
 * @returns {import('express').RequestHandler}
 */
export function securityHeaders(options = {}) {
  const opts = { ...DEFAULT_OPTIONS, ...options };

  return function securityHeadersMiddleware(_req, res, next) {
    // Content-Security-Policy – restrict resource origins
    res.setHeader("Content-Security-Policy", opts.csp);

    // HSTS – force HTTPS; include subdomains; allow preloading
    if (opts.hsts) {
      res.setHeader(
        "Strict-Transport-Security",
        "max-age=31536000; includeSubDomains; preload"
      );
    }

    // Clickjacking protection
    res.setHeader("X-Frame-Options", "DENY");

    // Prevent MIME-type sniffing
    res.setHeader("X-Content-Type-Options", "nosniff");

    // Legacy XSS filter (still respected by older browsers)
    res.setHeader("X-XSS-Protection", "1; mode=block");

    // Referrer leakage control
    res.setHeader("Referrer-Policy", "strict-origin-when-cross-origin");

    // Permissions / Feature policy – disable sensitive browser APIs
    res.setHeader(
      "Permissions-Policy",
      "geolocation=(), microphone=(), camera=(), payment=(), usb=()"
    );

    next();
  };
}
