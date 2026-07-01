/**
 * oauth.js — OAuth 2.0 authorization server routes (#310).
 *
 * Endpoints (RFC 6749):
 *   GET  /oauth/authorize           Authorization endpoint (auth code flow)
 *   POST /oauth/token               Token endpoint
 *   POST /oauth/revoke              Token revocation (RFC 7009)
 *
 * Client management (non-standard, management plane):
 *   POST   /oauth/clients            Register a new OAuth client
 *   GET    /oauth/clients            List clients for the authenticated owner
 *   GET    /oauth/clients/:id        Get a single client
 *   DELETE /oauth/clients/:id        Deactivate a client
 *
 * Metadata:
 *   GET  /oauth/scopes               List valid scopes
 *
 * Authentication for the management routes uses X-Owner (same placeholder
 * pattern as the API key routes; will be replaced by #309 JWT auth).
 */

import { Router } from "express";
import { OAuthService, OAuthError, VALID_SCOPES } from "../services/oauth.js";
import { oauthErrorHandler } from "../middleware/oauthAuth.js";

const router = Router();
const service = new OAuthService();

// ── Helpers ───────────────────────────────────────────────────────────────────

function getOwner(req, res) {
  const owner = req.headers["x-owner"];
  if (!owner) {
    res.status(401).json({
      error: "unauthorized",
      error_description:
        "X-Owner header with a valid Stellar address is required. " +
        "Will be replaced by JWT auth (#309).",
    });
    return null;
  }
  return owner;
}

/**
 * Parse HTTP Basic auth header.
 * @param {string|undefined} header
 * @returns {{ clientId: string, clientSecret: string }|null}
 */
function parseBasicAuth(header) {
  if (!header || !header.startsWith("Basic ")) return null;
  try {
    const decoded = Buffer.from(header.slice(6), "base64").toString("utf8");
    const idx = decoded.indexOf(":");
    if (idx < 0) return null;
    return {
      clientId: decoded.slice(0, idx),
      clientSecret: decoded.slice(idx + 1),
    };
  } catch (_) {
    return null;
  }
}

// ── Authorization endpoint ────────────────────────────────────────────────────

/**
 * GET /oauth/authorize
 *
 * The client redirects the resource owner here. In a full implementation the
 * server would render a consent UI. For this API-first service the endpoint
 * validates the request and auto-approves for authenticated API consumers
 * (consent UI is the responsibility of the client application).
 *
 * Required query params:
 *   response_type  Must be "code"
 *   client_id
 *   redirect_uri
 *   scope          Space-separated scopes
 *   state          CSRF token (passed back on redirect)
 *   code_challenge  (recommended, PKCE)
 *   code_challenge_method  "S256" | "plain"
 *
 * X-Owner header identifies the authorizing user.
 */
router.get("/authorize", async (req, res, next) => {
  try {
    const {
      response_type,
      client_id,
      redirect_uri,
      scope,
      state,
      code_challenge,
      code_challenge_method,
    } = req.query;

    if (response_type !== "code") {
      return res.status(400).json({
        error: "unsupported_response_type",
        error_description: "Only response_type=code is supported",
      });
    }
    if (!client_id || !redirect_uri || !scope) {
      return res.status(400).json({
        error: "invalid_request",
        error_description: "client_id, redirect_uri and scope are required",
      });
    }

    const userId = req.headers["x-owner"];
    if (!userId) {
      return res.status(401).json({
        error: "access_denied",
        error_description: "X-Owner header is required to identify the authorizing user",
      });
    }

    const code = await service.issueAuthCode({
      clientId: client_id,
      redirectUri: redirect_uri,
      scope,
      userId,
      codeChallenge: code_challenge,
      challengeMethod: code_challenge_method,
    });

    // Redirect back to the client with the code.
    const redirectUrl = new URL(redirect_uri);
    redirectUrl.searchParams.set("code", code);
    if (state) redirectUrl.searchParams.set("state", state);

    res.redirect(302, redirectUrl.toString());
  } catch (err) {
    if (err instanceof OAuthError) {
      // For auth errors before redirect: return JSON (no redirect URL confirmed yet).
      return res.status(400).json({
        error: err.code,
        error_description: err.message,
      });
    }
    next(err);
  }
});

// ── Token endpoint ────────────────────────────────────────────────────────────

/**
 * POST /oauth/token
 *
 * Content-Type: application/x-www-form-urlencoded
 * Authorization: Basic <base64(client_id:client_secret)>
 *
 * Supported grant_type values:
 *   authorization_code   — exchange auth code for tokens
 *   client_credentials   — machine-to-machine token
 *   refresh_token        — rotate refresh token
 */
router.post("/token", express_urlencoded, async (req, res, next) => {
  try {
    const {
      grant_type,
      code,
      redirect_uri,
      code_verifier,
      scope = "",
      refresh_token,
    } = req.body;

    // Client authentication via Basic auth or body params.
    let clientId, clientSecret;
    const basicAuth = parseBasicAuth(req.headers.authorization);
    if (basicAuth) {
      ({ clientId, clientSecret } = basicAuth);
    } else {
      clientId = req.body.client_id;
      clientSecret = req.body.client_secret;
    }

    if (!clientId || !clientSecret) {
      return res.status(401).json({
        error: "invalid_client",
        error_description: "Client authentication required (Basic auth or client_id + client_secret in body)",
      });
    }

    let tokenResponse;

    switch (grant_type) {
      case "authorization_code":
        if (!code || !redirect_uri) {
          return res.status(400).json({
            error: "invalid_request",
            error_description: "code and redirect_uri are required for authorization_code grant",
          });
        }
        tokenResponse = await service.exchangeAuthCode({
          clientId,
          clientSecret,
          code,
          redirectUri: redirect_uri,
          codeVerifier: code_verifier,
        });
        break;

      case "client_credentials":
        if (!scope) {
          return res.status(400).json({
            error: "invalid_request",
            error_description: "scope is required for client_credentials grant",
          });
        }
        tokenResponse = await service.clientCredentials({
          clientId,
          clientSecret,
          scope,
        });
        break;

      case "refresh_token":
        if (!refresh_token) {
          return res.status(400).json({
            error: "invalid_request",
            error_description: "refresh_token is required",
          });
        }
        tokenResponse = await service.refreshAccessToken({
          clientId,
          clientSecret,
          refreshToken: refresh_token,
          scope: scope || undefined,
        });
        break;

      default:
        return res.status(400).json({
          error: "unsupported_grant_type",
          error_description: `grant_type '${grant_type}' is not supported. Supported: authorization_code, client_credentials, refresh_token`,
        });
    }

    // RFC 6749 §5.1: cache-control headers on token responses.
    res.setHeader("Cache-Control", "no-store");
    res.setHeader("Pragma", "no-cache");
    res.status(200).json(tokenResponse);
  } catch (err) {
    next(err);
  }
});

// ── Revocation endpoint ───────────────────────────────────────────────────────

/**
 * POST /oauth/revoke  (RFC 7009)
 *
 * Body (form-urlencoded):
 *   token       — the token to revoke
 *   token_type_hint — "access_token" | "refresh_token" (optional)
 */
router.post("/revoke", express_urlencoded, async (req, res, next) => {
  try {
    const { token, token_type_hint } = req.body;
    if (!token) {
      return res.status(400).json({
        error: "invalid_request",
        error_description: "token is required",
      });
    }
    await service.revokeToken(token, token_type_hint);
    // RFC 7009 §2.2: always return 200 even if token was not found.
    res.status(200).json({});
  } catch (err) {
    next(err);
  }
});

// ── Client management routes ──────────────────────────────────────────────────

/**
 * GET /oauth/scopes
 * Returns the list of valid permission scopes.
 */
router.get("/scopes", (_req, res) => {
  res.json({ scopes: VALID_SCOPES });
});

/**
 * POST /oauth/clients
 * Body: { client_name, description?, redirect_uris, scopes, grant_types, rate_limit? }
 * Returns: { client: OAuthClientRecord, clientSecret: string }
 *
 * clientSecret is returned ONCE — store it securely.
 */
router.post("/clients", async (req, res, next) => {
  try {
    const owner = getOwner(req, res);
    if (!owner) return;

    const {
      client_name,
      description,
      redirect_uris,
      scopes,
      grant_types,
      rate_limit,
    } = req.body;

    if (!client_name) {
      return res.status(422).json({
        error: "invalid_request",
        error_description: "client_name is required",
      });
    }
    if (!Array.isArray(redirect_uris) || redirect_uris.length === 0) {
      return res.status(422).json({
        error: "invalid_request",
        error_description: "redirect_uris must be a non-empty array",
      });
    }
    if (!Array.isArray(scopes) || scopes.length === 0) {
      return res.status(422).json({
        error: "invalid_request",
        error_description: "scopes must be a non-empty array",
      });
    }
    if (!Array.isArray(grant_types) || grant_types.length === 0) {
      return res.status(422).json({
        error: "invalid_request",
        error_description: "grant_types must be a non-empty array",
      });
    }

    const result = await service.registerClient({
      owner,
      clientName: client_name,
      description,
      redirectUris: redirect_uris,
      scopes,
      grantTypes: grant_types,
      rateLimit: rate_limit ?? 100,
    });

    res.status(201).json(result);
  } catch (err) {
    if (err.message.startsWith("Unknown scopes") || err.message.startsWith("Unknown grant")) {
      return res.status(422).json({ error: "invalid_request", error_description: err.message });
    }
    next(err);
  }
});

/**
 * GET /oauth/clients
 * Returns all clients registered by the authenticated owner.
 */
router.get("/clients", async (req, res, next) => {
  try {
    const owner = getOwner(req, res);
    if (!owner) return;
    const clients = await service.listClients(owner);
    res.json({ clients });
  } catch (err) {
    next(err);
  }
});

/**
 * GET /oauth/clients/:id
 */
router.get("/clients/:id", async (req, res, next) => {
  try {
    const owner = getOwner(req, res);
    if (!owner) return;
    const client = await service.getClient(req.params.id, owner);
    if (!client) {
      return res.status(404).json({ error: "not_found", error_description: "Client not found" });
    }
    res.json({ client });
  } catch (err) {
    next(err);
  }
});

/**
 * DELETE /oauth/clients/:id
 * Deactivates the client. Returns 204.
 */
router.delete("/clients/:id", async (req, res, next) => {
  try {
    const owner = getOwner(req, res);
    if (!owner) return;
    await service.deactivateClient(req.params.id, owner);
    res.status(204).end();
  } catch (err) {
    if (err.message.includes("not found")) {
      return res.status(404).json({ error: "not_found", error_description: err.message });
    }
    next(err);
  }
});

// ── OAuth error handler ───────────────────────────────────────────────────────

router.use(oauthErrorHandler);

// ── Helpers ───────────────────────────────────────────────────────────────────

/**
 * Middleware to parse application/x-www-form-urlencoded bodies.
 * Only applied to token and revoke endpoints.
 */
function express_urlencoded(req, res, next) {
  if (req.headers["content-type"]?.startsWith("application/x-www-form-urlencoded")) {
    let body = "";
    req.on("data", (chunk) => { body += chunk.toString(); });
    req.on("end", () => {
      req.body = Object.fromEntries(new URLSearchParams(body));
      next();
    });
  } else {
    next();
  }
}

export default router;
