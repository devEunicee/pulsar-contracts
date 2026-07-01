-- UP
-- API Key Management schema (#315)
-- Stores API keys with scoped permissions, expiration, and activity logging.

CREATE TABLE api_keys (
  id              TEXT        PRIMARY KEY,         -- opaque UUID
  name            TEXT        NOT NULL,            -- human label
  owner           TEXT        NOT NULL,            -- Stellar account ID (G...)
  key_prefix      TEXT        NOT NULL,            -- first 8 chars of key (shown in UI)
  key_hash        TEXT        NOT NULL UNIQUE,     -- SHA-256(api_key), never store plaintext
  scopes          TEXT        NOT NULL,            -- comma-separated permission scopes
  rate_limit      INTEGER     NOT NULL DEFAULT 1000, -- requests per hour
  expires_at      TIMESTAMPTZ,                     -- NULL = never expires
  last_used_at    TIMESTAMPTZ,
  revoked         BOOLEAN     NOT NULL DEFAULT false,
  revoked_at      TIMESTAMPTZ,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_api_keys_owner      ON api_keys (owner);
CREATE INDEX idx_api_keys_key_hash   ON api_keys (key_hash);
CREATE INDEX idx_api_keys_revoked    ON api_keys (revoked);
CREATE INDEX idx_api_keys_expires_at ON api_keys (expires_at);

-- Activity log — one row per authenticated API request.
CREATE TABLE api_key_activity (
  id          BIGSERIAL   PRIMARY KEY,
  key_id      TEXT        NOT NULL REFERENCES api_keys (id) ON DELETE CASCADE,
  ip_address  TEXT,
  method      TEXT,
  path        TEXT,
  status_code INTEGER,
  duration_ms INTEGER,
  created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_api_key_activity_key_id    ON api_key_activity (key_id);
CREATE INDEX idx_api_key_activity_created_at ON api_key_activity (created_at);

-- DOWN
DROP INDEX IF EXISTS idx_api_key_activity_created_at;
DROP INDEX IF EXISTS idx_api_key_activity_key_id;
DROP TABLE IF EXISTS api_key_activity;
DROP INDEX IF EXISTS idx_api_keys_expires_at;
DROP INDEX IF EXISTS idx_api_keys_revoked;
DROP INDEX IF EXISTS idx_api_keys_key_hash;
DROP INDEX IF EXISTS idx_api_keys_owner;
DROP TABLE IF EXISTS api_keys;
