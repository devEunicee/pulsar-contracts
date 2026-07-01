/**
 * oauth.js — OAuth 2.0 Authorization Server service (#310).
 *
 * Implements:
 *   - Authorization Code flow (with PKCE)
 *   - Client Credentials flow
 *   - Token endpoint (issue access + refresh tokens)
 *   - Refresh token grant
 *   - Scope-based permissions
 *   - Client registration and management
 *   - Per-client rate limiting
 *   - Token revocation (RFC 7009)
 *   - Verified third-party app flag
 *
 * Token format: <prefix>_<48-hex-chars>
 *   Access token prefix:  "at"   → at_<48-hex>
 *   Refresh token prefix: "rt"   → rt_<48-hex>
 *   Auth code prefix:     "ac"   → ac_<48-hex>
 *
 * Only SHA-256 hashes of tokens are persisted.
 *
 * Scopes (space-separated string, RFC 6749 §3.3):
 *   payments:read    payments:write
 *   merchants:read   merchants:write
 *   refunds:read     refunds:write
 *   profile          (basic user info)
 *   admin
 *
 * Dependencies: #309 (JWT) — access tokens are opaque bearer tokens in this
 * implementation.  JWT signing is left as a hook (see signAccessToken) so the
 * JWT infrastructure from #309 can be dropped in without changing the rest of
 * the module.
 */

import crypto from "crypto";
import { Client } from "pg";

// ── Constants ─────────────────────────────────────────────────────────────────

export const VALID_SCOPES = Object.freeze([
  "payments:read",
  "payments:write",
  "merchants:read",
  "merchants:write",
  "refunds:read",
  "refunds:write",
  "profile",
  "admin",
]);

const ACCESS_TOKEN_TTL_SECONDS = parseInt(
  process.env.OAUTH_ACCESS_TOKEN_TTL ?? "3600",
  10
); // 1 hour
const REFRESH_TOKEN_TTL_SECONDS = parseInt(
  process.env.OAUTH_REFRESH_TOKEN_TTL ?? "2592000",
  10
); // 30 days
const AUTH_CODE_TTL_SECONDS = 600; // 10 minutes (RFC recommends ≤ 10 min)
const RATE_LIMIT_WINDOW_MS = 60 * 60 * 1000; // 1 hour

// ── In-memory rate-limit store ────────────────────────────────────────────────

const rateLimitStore = new Map();

/**
 * Consume one token from a client's rate-limit bucket.
 *
 * @param {string} clientId
 * @param {number} limit  Requests per hour.
 * @returns {{ allowed: boolean, remaining: number, resetAt: number }}
 */
export function checkClientRateLimit(clientId, limit) {
  const now = Date.now();
  let bucket = rateLimitStore.get(clientId);
  if (!bucket || now - bucket.windowStart >= RATE_LIMIT_WINDOW_MS) {
    bucket = { count: 0, windowStart: now };
    rateLimitStore.set(clientId, bucket);
  }
  bucket.count += 1;
  const remaining = Math.max(0, limit - bucket.count);
  return {
    allowed: bucket.count <= limit,
    remaining,
    resetAt: bucket.windowStart + RATE_LIMIT_WINDOW_MS,
  };
}

// ── Token helpers ─────────────────────────────────────────────────────────────

/**
 * Generate a secure opaque token with the given prefix.
 * @param {"at"|"rt"|"ac"} prefix
 * @returns {{ value: string, hash: string }}
 */
function generateToken(prefix) {
  const value = `${prefix}_${crypto.randomBytes(24).toString("hex")}`;
  const hash = crypto.createHash("sha256").update(value).digest("hex");
  return { value, hash };
}

/**
 * Hash a user-supplied token for DB lookup.
 * @param {string} token
 * @returns {string}
 */
export function hashToken(token) {
  return crypto.createHash("sha256").update(token).digest("hex");
}

/**
 * Hook for JWT signing (#309). For now returns the opaque token value.
 * Replace with JWT signing once #309 is merged.
 *
 * @param {object} payload  Token payload (not used for opaque tokens).
 * @param {string} value    The raw opaque token.
 * @returns {string}
 */
function signAccessToken(_payload, value) {
  return value;
}

// ── Scope helpers ─────────────────────────────────────────────────────────────

/**
 * Validate and normalise a space-separated scope string.
 *
 * @param {string} requestedScopeStr  Scope string from the request.
 * @param {string} allowedScopeStr    Scope string from the client registration.
 * @returns {{ valid: boolean, scopes: string[], unauthorized: string[] }}
 */
export function resolveScopes(requestedScopeStr, allowedScopeStr) {
  const requested = requestedScopeStr.trim().split(/\s+/).filter(Boolean);
  const allowed = allowedScopeStr.trim().split(/\s+/).filter(Boolean);

  const unauthorized = requested.filter((s) => !allowed.includes(s));
  const unknown = requested.filter((s) => !VALID_SCOPES.includes(s));

  return {
    valid: unauthorized.length === 0 && unknown.length === 0,
    scopes: requested,
    unauthorized: [...new Set([...unauthorized, ...unknown])],
  };
}

/**
 * Check whether a token's scopes include the required scope.
 * @param {string} tokenScopes  Space-separated scope list.
 * @param {string} required
 * @returns {boolean}
 */
export function hasScope(tokenScopes, required) {
  const scopes = tokenScopes.trim().split(/\s+/);
  return scopes.includes("admin") || scopes.includes(required);
}

// ── Client secret helpers ─────────────────────────────────────────────────────

function hashSecret(secret) {
  return crypto.createHash("sha256").update(secret).digest("hex");
}

function generateClientCredentials() {
  const clientId = `client_${crypto.randomBytes(12).toString("hex")}`;
  const clientSecret = `cs_${crypto.randomBytes(24).toString("hex")}`;
  return { clientId, clientSecret, secretHash: hashSecret(clientSecret) };
}

// ── OAuthService ──────────────────────────────────────────────────────────────

export class OAuthService {
  /**
   * @param {{ connectionString?: string } | import("pg").Client} dbOrOpts
   */
  constructor(dbOrOpts = {}) {
    if (dbOrOpts && typeof dbOrOpts.query === "function") {
      this._db = dbOrOpts;
      this._ownsDb = false;
    } else {
      this._db = new Client({
        connectionString:
          (dbOrOpts && dbOrOpts.connectionString) ?? process.env.DATABASE_URL,
      });
      this._ownsDb = true;
    }
    this._connected = false;
  }

  async _ensureConnected() {
    if (!this._connected) {
      await this._db.connect();
      this._connected = true;
    }
  }

  async close() {
    if (this._ownsDb && this._connected) {
      await this._db.end();
      this._connected = false;
    }
  }

  // ── Client registration ───────────────────────────────────────────────────

  /**
   * Register a new OAuth 2.0 client application.
   *
   * @param {object} opts
   * @param {string}   opts.owner         Stellar address of the registrant.
   * @param {string}   opts.clientName    Human-readable app name.
   * @param {string}   [opts.description] Optional description.
   * @param {string[]} opts.redirectUris  Allowed redirect URIs.
   * @param {string[]} opts.scopes        Requested permission scopes.
   * @param {string[]} opts.grantTypes    e.g. ["authorization_code", "refresh_token"]
   * @param {number}   [opts.rateLimit]   Token requests per hour.
   * @returns {Promise<{ client: OAuthClientRecord, clientSecret: string }>}
   */
  async registerClient({
    owner,
    clientName,
    description,
    redirectUris,
    scopes,
    grantTypes,
    rateLimit = 100,
  }) {
    const unknownScopes = scopes.filter((s) => !VALID_SCOPES.includes(s));
    if (unknownScopes.length) {
      throw new Error(`Unknown scopes: ${unknownScopes.join(", ")}`);
    }
    const validGrants = ["authorization_code", "client_credentials", "refresh_token"];
    const unknownGrants = grantTypes.filter((g) => !validGrants.includes(g));
    if (unknownGrants.length) {
      throw new Error(`Unknown grant types: ${unknownGrants.join(", ")}`);
    }

    await this._ensureConnected();
    const { clientId, clientSecret, secretHash } = generateClientCredentials();

    const { rows } = await this._db.query(
      `INSERT INTO oauth_clients
         (client_id, client_secret, client_name, description, redirect_uris,
          scopes, grant_types, owner, rate_limit)
       VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
       RETURNING *`,
      [
        clientId,
        secretHash,
        clientName,
        description ?? null,
        JSON.stringify(redirectUris),
        scopes.join(" "),
        grantTypes.join(","),
        owner,
        rateLimit,
      ]
    );

    return { client: sanitizeClient(rows[0]), clientSecret };
  }

  /**
   * List clients registered by an owner.
   * @param {string} owner
   * @returns {Promise<OAuthClientRecord[]>}
   */
  async listClients(owner) {
    await this._ensureConnected();
    const { rows } = await this._db.query(
      `SELECT * FROM oauth_clients WHERE owner = $1 ORDER BY created_at DESC`,
      [owner]
    );
    return rows.map(sanitizeClient);
  }

  /**
   * Get a single client by ID (verifies ownership).
   * @param {string} clientId
   * @param {string} owner
   * @returns {Promise<OAuthClientRecord|null>}
   */
  async getClient(clientId, owner) {
    await this._ensureConnected();
    const { rows } = await this._db.query(
      `SELECT * FROM oauth_clients WHERE client_id = $1 AND owner = $2`,
      [clientId, owner]
    );
    return rows.length ? sanitizeClient(rows[0]) : null;
  }

  /**
   * Authenticate a client using its client_id + client_secret.
   * @param {string} clientId
   * @param {string} clientSecret  Plaintext secret from Basic auth.
   * @returns {Promise<OAuthClientRecord|null>}
   */
  async authenticateClient(clientId, clientSecret) {
    await this._ensureConnected();
    const { rows } = await this._db.query(
      `SELECT * FROM oauth_clients WHERE client_id = $1 AND active = true`,
      [clientId]
    );
    if (!rows.length) return null;
    const client = rows[0];
    const expectedHash = hashSecret(clientSecret);
    if (!crypto.timingSafeEqual(Buffer.from(client.client_secret), Buffer.from(expectedHash))) {
      return null;
    }
    return sanitizeClient(client);
  }

  /**
   * Delete / deactivate a client.
   * @param {string} clientId
   * @param {string} owner
   */
  async deactivateClient(clientId, owner) {
    await this._ensureConnected();
    const { rowCount } = await this._db.query(
      `UPDATE oauth_clients SET active = false, updated_at = now()
       WHERE client_id = $1 AND owner = $2`,
      [clientId, owner]
    );
    if (rowCount === 0) throw new Error("Client not found or not owned by caller");
  }

  // ── Authorization code flow ───────────────────────────────────────────────

  /**
   * Validate an authorization request and issue an auth code.
   *
   * @param {object} opts
   * @param {string}  opts.clientId
   * @param {string}  opts.redirectUri
   * @param {string}  opts.scope
   * @param {string}  opts.userId         Stellar address of the authorizing user.
   * @param {string}  [opts.codeChallenge] PKCE code_challenge.
   * @param {string}  [opts.challengeMethod] "S256" | "plain"
   * @returns {Promise<string>}  The authorization code (plaintext).
   */
  async issueAuthCode({
    clientId,
    redirectUri,
    scope,
    userId,
    codeChallenge,
    challengeMethod,
  }) {
    await this._ensureConnected();

    // Load client.
    const { rows: clientRows } = await this._db.query(
      `SELECT * FROM oauth_clients WHERE client_id = $1 AND active = true`,
      [clientId]
    );
    if (!clientRows.length) throw new OAuthError("invalid_client", "Unknown client");

    const client = clientRows[0];
    const allowedUris = JSON.parse(client.redirect_uris);
    if (!allowedUris.includes(redirectUri)) {
      throw new OAuthError("invalid_request", "redirect_uri not registered for this client");
    }

    const grantTypes = client.grant_types.split(",");
    if (!grantTypes.includes("authorization_code")) {
      throw new OAuthError(
        "unauthorized_client",
        "Client is not authorized for authorization_code grant"
      );
    }

    // Validate scopes.
    const { valid, unauthorized } = resolveScopes(scope, client.scopes);
    if (!valid) {
      throw new OAuthError("invalid_scope", `Unauthorized scopes: ${unauthorized.join(" ")}`);
    }

    const { value: code, hash } = generateToken("ac");
    const expiresAt = new Date(Date.now() + AUTH_CODE_TTL_SECONDS * 1000);

    await this._db.query(
      `INSERT INTO oauth_authorization_codes
         (code, client_id, user_id, redirect_uri, scopes, code_challenge, challenge_method, expires_at)
       VALUES ($1, $2, $3, $4, $5, $6, $7, $8)`,
      [hash, clientId, userId, redirectUri, scope, codeChallenge ?? null, challengeMethod ?? null, expiresAt]
    );

    return code; // plaintext returned to the redirect URI
  }

  // ── Token endpoint ────────────────────────────────────────────────────────

  /**
   * Exchange an authorization code for an access + refresh token pair.
   *
   * @param {object} opts
   * @param {string}  opts.clientId
   * @param {string}  opts.clientSecret
   * @param {string}  opts.code           Plaintext authorization code.
   * @param {string}  opts.redirectUri
   * @param {string}  [opts.codeVerifier]  PKCE code_verifier.
   * @returns {Promise<TokenResponse>}
   */
  async exchangeAuthCode({ clientId, clientSecret, code, redirectUri, codeVerifier }) {
    await this._ensureConnected();

    const client = await this.authenticateClient(clientId, clientSecret);
    if (!client) throw new OAuthError("invalid_client", "Invalid client credentials");

    // Check rate limit.
    const { allowed } = checkClientRateLimit(clientId, client.rate_limit ?? 100);
    if (!allowed) throw new OAuthError("slow_down", "Rate limit exceeded");

    const codeHash = hashToken(code);
    const { rows } = await this._db.query(
      `SELECT * FROM oauth_authorization_codes
       WHERE code = $1 AND used = false AND expires_at > now()`,
      [codeHash]
    );

    if (!rows.length) {
      throw new OAuthError("invalid_grant", "Authorization code is invalid or expired");
    }
    const authCode = rows[0];

    if (authCode.client_id !== clientId) {
      throw new OAuthError("invalid_grant", "Authorization code was not issued to this client");
    }
    if (authCode.redirect_uri !== redirectUri) {
      throw new OAuthError("invalid_grant", "redirect_uri mismatch");
    }

    // PKCE verification.
    if (authCode.code_challenge) {
      if (!codeVerifier) {
        throw new OAuthError("invalid_grant", "code_verifier is required");
      }
      const method = authCode.challenge_method ?? "plain";
      let computedChallenge;
      if (method === "S256") {
        computedChallenge = crypto
          .createHash("sha256")
          .update(codeVerifier)
          .digest("base64url");
      } else {
        computedChallenge = codeVerifier;
      }
      if (computedChallenge !== authCode.code_challenge) {
        throw new OAuthError("invalid_grant", "PKCE code_verifier does not match challenge");
      }
    }

    // Mark code as used (one-time use).
    await this._db.query(
      `UPDATE oauth_authorization_codes SET used = true WHERE code = $1`,
      [codeHash]
    );

    return this._issueTokens({
      clientId,
      userId: authCode.user_id,
      scopes: authCode.scopes,
      grantType: "authorization_code",
    });
  }

  /**
   * Issue tokens via Client Credentials grant.
   *
   * @param {object} opts
   * @param {string}  opts.clientId
   * @param {string}  opts.clientSecret
   * @param {string}  opts.scope
   * @returns {Promise<TokenResponse>}
   */
  async clientCredentials({ clientId, clientSecret, scope }) {
    await this._ensureConnected();

    const client = await this.authenticateClient(clientId, clientSecret);
    if (!client) throw new OAuthError("invalid_client", "Invalid client credentials");

    const { allowed } = checkClientRateLimit(clientId, client.rate_limit ?? 100);
    if (!allowed) throw new OAuthError("slow_down", "Rate limit exceeded");

    const grantTypes = client.grant_types.split(",");
    if (!grantTypes.includes("client_credentials")) {
      throw new OAuthError(
        "unauthorized_client",
        "Client is not authorized for client_credentials grant"
      );
    }

    // Validate scopes.
    const { valid, unauthorized } = resolveScopes(scope, client.scopes);
    if (!valid) {
      throw new OAuthError("invalid_scope", `Unauthorized scopes: ${unauthorized.join(" ")}`);
    }

    const response = await this._issueTokens({
      clientId,
      userId: null, // no user context for machine-to-machine flow
      scopes: scope,
      grantType: "client_credentials",
    });

    // Client credentials grant does not return a refresh token.
    delete response.refresh_token;
    return response;
  }

  /**
   * Refresh an access token using a refresh token.
   *
   * @param {object} opts
   * @param {string}  opts.clientId
   * @param {string}  opts.clientSecret
   * @param {string}  opts.refreshToken  Plaintext refresh token.
   * @param {string}  [opts.scope]       Optional scope downgrade.
   * @returns {Promise<TokenResponse>}
   */
  async refreshAccessToken({ clientId, clientSecret, refreshToken, scope }) {
    await this._ensureConnected();

    const client = await this.authenticateClient(clientId, clientSecret);
    if (!client) throw new OAuthError("invalid_client", "Invalid client credentials");

    const { allowed } = checkClientRateLimit(clientId, client.rate_limit ?? 100);
    if (!allowed) throw new OAuthError("slow_down", "Rate limit exceeded");

    const rtHash = hashToken(refreshToken);
    const { rows } = await this._db.query(
      `SELECT * FROM oauth_refresh_tokens
       WHERE token_hash = $1 AND revoked = false AND expires_at > now()`,
      [rtHash]
    );

    if (!rows.length) {
      throw new OAuthError("invalid_grant", "Refresh token is invalid, expired, or revoked");
    }

    const rt = rows[0];
    if (rt.client_id !== clientId) {
      throw new OAuthError("invalid_grant", "Refresh token was not issued to this client");
    }

    // Honour optional scope downgrade (can only reduce, never expand).
    let effectiveScope = rt.scopes;
    if (scope) {
      const { valid, unauthorized } = resolveScopes(scope, rt.scopes);
      if (!valid) {
        throw new OAuthError("invalid_scope", `Requested scopes exceed original grant: ${unauthorized.join(" ")}`);
      }
      effectiveScope = scope;
    }

    // Revoke old refresh token (rotation).
    await this._db.query(
      `UPDATE oauth_refresh_tokens SET revoked = true WHERE token_hash = $1`,
      [rtHash]
    );

    return this._issueTokens({
      clientId,
      userId: rt.user_id,
      scopes: effectiveScope,
      grantType: "authorization_code",
    });
  }

  /**
   * Validate an access token and return its metadata.
   *
   * @param {string} token  Plaintext bearer token.
   * @returns {Promise<AccessTokenMetadata|null>}
   */
  async validateAccessToken(token) {
    await this._ensureConnected();
    const hash = hashToken(token);
    const { rows } = await this._db.query(
      `SELECT * FROM oauth_access_tokens
       WHERE token_hash = $1 AND revoked = false AND expires_at > now()`,
      [hash]
    );
    if (!rows.length) return null;
    return {
      clientId: rows[0].client_id,
      userId: rows[0].user_id,
      scopes: rows[0].scopes,
      grantType: rows[0].grant_type,
      expiresAt: rows[0].expires_at,
    };
  }

  /**
   * Revoke an access or refresh token (RFC 7009).
   *
   * @param {string} token  Plaintext token to revoke.
   * @param {string} hint   "access_token" | "refresh_token"
   */
  async revokeToken(token, hint) {
    await this._ensureConnected();
    const hash = hashToken(token);
    if (hint === "refresh_token") {
      await this._db.query(
        `UPDATE oauth_refresh_tokens SET revoked = true WHERE token_hash = $1`,
        [hash]
      );
    } else {
      // Try both tables — spec says ignore hint if wrong.
      await Promise.all([
        this._db.query(
          `UPDATE oauth_access_tokens SET revoked = true WHERE token_hash = $1`,
          [hash]
        ),
        this._db.query(
          `UPDATE oauth_refresh_tokens SET revoked = true WHERE token_hash = $1`,
          [hash]
        ),
      ]);
    }
  }

  // ── Internal helpers ──────────────────────────────────────────────────────

  /**
   * Persist and return a new access + refresh token pair.
   * @private
   */
  async _issueTokens({ clientId, userId, scopes, grantType }) {
    const accessToken = generateToken("at");
    const refreshToken = grantType !== "client_credentials" ? generateToken("rt") : null;

    const accessExpiresAt = new Date(Date.now() + ACCESS_TOKEN_TTL_SECONDS * 1000);
    const refreshExpiresAt = refreshToken
      ? new Date(Date.now() + REFRESH_TOKEN_TTL_SECONDS * 1000)
      : null;

    // Persist access token.
    await this._db.query(
      `INSERT INTO oauth_access_tokens
         (token_hash, client_id, user_id, scopes, grant_type, expires_at)
       VALUES ($1, $2, $3, $4, $5, $6)`,
      [accessToken.hash, clientId, userId, scopes, grantType, accessExpiresAt]
    );

    // Persist refresh token (not for client_credentials).
    if (refreshToken) {
      await this._db.query(
        `INSERT INTO oauth_refresh_tokens
           (token_hash, client_id, user_id, scopes, access_token_hash, expires_at)
         VALUES ($1, $2, $3, $4, $5, $6)`,
        [
          refreshToken.hash,
          clientId,
          userId,
          scopes,
          accessToken.hash,
          refreshExpiresAt,
        ]
      );
    }

    /** @type {TokenResponse} */
    const response = {
      access_token: signAccessToken({ clientId, userId, scopes }, accessToken.value),
      token_type: "Bearer",
      expires_in: ACCESS_TOKEN_TTL_SECONDS,
      scope: scopes,
    };

    if (refreshToken) {
      response.refresh_token = refreshToken.value;
    }

    return response;
  }
}

// ── OAuthError ────────────────────────────────────────────────────────────────

/**
 * RFC 6749-compliant error with an error code.
 */
export class OAuthError extends Error {
  /**
   * @param {string} code     RFC 6749 error code (e.g. "invalid_grant").
   * @param {string} message  Human-readable description.
   */
  constructor(code, message) {
    super(message);
    this.code = code;
    this.name = "OAuthError";
  }
}

// ── Sanitizers ────────────────────────────────────────────────────────────────

/** Strip the stored secret hash before returning a client record. */
function sanitizeClient(row) {
  const { client_secret, ...safe } = row;  // eslint-disable-line no-unused-vars
  return {
    ...safe,
    redirect_uris: Array.isArray(safe.redirect_uris)
      ? safe.redirect_uris
      : JSON.parse(safe.redirect_uris ?? "[]"),
    scopes: safe.scopes ? safe.scopes.split(" ") : [],
    grant_types: safe.grant_types ? safe.grant_types.split(",") : [],
  };
}

/**
 * @typedef {object} TokenResponse
 * @property {string} access_token
 * @property {string} token_type  "Bearer"
 * @property {number} expires_in  Seconds until expiry.
 * @property {string} scope
 * @property {string} [refresh_token]
 */

/**
 * @typedef {object} AccessTokenMetadata
 * @property {string}      clientId
 * @property {string|null} userId
 * @property {string}      scopes
 * @property {string}      grantType
 * @property {Date}        expiresAt
 */

/**
 * @typedef {object} OAuthClientRecord
 * @property {string}   client_id
 * @property {string}   client_name
 * @property {string}   description
 * @property {string[]} redirect_uris
 * @property {string[]} scopes
 * @property {string[]} grant_types
 * @property {string}   owner
 * @property {boolean}  verified
 * @property {boolean}  active
 * @property {number}   rate_limit
 * @property {string}   created_at
 * @property {string}   updated_at
 */
