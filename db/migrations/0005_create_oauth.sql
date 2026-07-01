-- UP
-- OAuth 2.0 Integration schema (#310)
-- Supports authorization code flow, client credentials flow, and refresh tokens.

-- Registered OAuth clients (third-party applications).
CREATE TABLE oauth_clients (
  client_id       TEXT        PRIMARY KEY,
  client_secret   TEXT        NOT NULL,              -- stored as SHA-256 hash
  client_name     TEXT        NOT NULL,
  description     TEXT,
  redirect_uris   TEXT        NOT NULL,              -- JSON array of allowed redirect URIs
  scopes          TEXT        NOT NULL,              -- space-separated allowed scopes
  grant_types     TEXT        NOT NULL,              -- comma-separated: authorization_code,client_credentials,refresh_token
  owner           TEXT        NOT NULL,              -- Stellar account ID of the registrant
  verified        BOOLEAN     NOT NULL DEFAULT FALSE,
  active          BOOLEAN     NOT NULL DEFAULT TRUE,
  rate_limit      INTEGER     NOT NULL DEFAULT 100,  -- token requests per hour
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_oauth_clients_owner  ON oauth_clients (owner);
CREATE INDEX idx_oauth_clients_active ON oauth_clients (active);

-- Authorization codes (short-lived, used once in authorization code flow).
CREATE TABLE oauth_authorization_codes (
  code            TEXT        PRIMARY KEY,
  client_id       TEXT        NOT NULL REFERENCES oauth_clients (client_id) ON DELETE CASCADE,
  user_id         TEXT        NOT NULL,              -- Stellar account ID of the authorizing user
  redirect_uri    TEXT        NOT NULL,
  scopes          TEXT        NOT NULL,
  code_challenge  TEXT,                              -- PKCE code_challenge
  challenge_method TEXT,                             -- S256 or plain
  expires_at      TIMESTAMPTZ NOT NULL,
  used            BOOLEAN     NOT NULL DEFAULT FALSE,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_auth_codes_client_id  ON oauth_authorization_codes (client_id);
CREATE INDEX idx_auth_codes_expires_at ON oauth_authorization_codes (expires_at);

-- Access tokens.
CREATE TABLE oauth_access_tokens (
  token_hash      TEXT        PRIMARY KEY,           -- SHA-256(token)
  client_id       TEXT        NOT NULL REFERENCES oauth_clients (client_id) ON DELETE CASCADE,
  user_id         TEXT,                              -- NULL for client_credentials grant
  scopes          TEXT        NOT NULL,
  grant_type      TEXT        NOT NULL,              -- authorization_code | client_credentials
  expires_at      TIMESTAMPTZ NOT NULL,
  revoked         BOOLEAN     NOT NULL DEFAULT FALSE,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_access_tokens_client_id  ON oauth_access_tokens (client_id);
CREATE INDEX idx_access_tokens_expires_at ON oauth_access_tokens (expires_at);
CREATE INDEX idx_access_tokens_revoked    ON oauth_access_tokens (revoked);

-- Refresh tokens (long-lived; used to obtain new access tokens).
CREATE TABLE oauth_refresh_tokens (
  token_hash      TEXT        PRIMARY KEY,           -- SHA-256(token)
  client_id       TEXT        NOT NULL REFERENCES oauth_clients (client_id) ON DELETE CASCADE,
  user_id         TEXT        NOT NULL,
  scopes          TEXT        NOT NULL,
  access_token_hash TEXT,                            -- links to the access token it was issued with
  expires_at      TIMESTAMPTZ NOT NULL,
  revoked         BOOLEAN     NOT NULL DEFAULT FALSE,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_refresh_tokens_client_id  ON oauth_refresh_tokens (client_id);
CREATE INDEX idx_refresh_tokens_expires_at ON oauth_refresh_tokens (expires_at);
CREATE INDEX idx_refresh_tokens_revoked    ON oauth_refresh_tokens (revoked);

-- DOWN
DROP INDEX IF EXISTS idx_refresh_tokens_revoked;
DROP INDEX IF EXISTS idx_refresh_tokens_expires_at;
DROP INDEX IF EXISTS idx_refresh_tokens_client_id;
DROP TABLE IF EXISTS oauth_refresh_tokens;
DROP INDEX IF EXISTS idx_access_tokens_revoked;
DROP INDEX IF EXISTS idx_access_tokens_expires_at;
DROP INDEX IF EXISTS idx_access_tokens_client_id;
DROP TABLE IF EXISTS oauth_access_tokens;
DROP INDEX IF EXISTS idx_auth_codes_expires_at;
DROP INDEX IF EXISTS idx_auth_codes_client_id;
DROP TABLE IF EXISTS oauth_authorization_codes;
DROP INDEX IF EXISTS idx_oauth_clients_active;
DROP INDEX IF EXISTS idx_oauth_clients_owner;
DROP TABLE IF EXISTS oauth_clients;
