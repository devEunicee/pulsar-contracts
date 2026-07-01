-- UP
-- Issue #314: Password management tables

CREATE TABLE user_credentials (
    user_id        VARCHAR(56)  PRIMARY KEY,           -- Stellar address or internal user ID
    password_hash  TEXT         NOT NULL,              -- bcrypt/argon2 hash
    created_at     TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at     TIMESTAMPTZ  NOT NULL DEFAULT now()
);

-- Last N hashes retained to prevent password reuse
CREATE TABLE password_history (
    id             BIGSERIAL    PRIMARY KEY,
    user_id        VARCHAR(56)  NOT NULL REFERENCES user_credentials (user_id) ON DELETE CASCADE,
    password_hash  TEXT         NOT NULL,
    created_at     TIMESTAMPTZ  NOT NULL DEFAULT now()
);

CREATE INDEX idx_pw_history_user ON password_history (user_id, created_at DESC);

-- Time-limited, single-use reset tokens
CREATE TABLE password_reset_tokens (
    token_hash     VARCHAR(64)  PRIMARY KEY,           -- SHA-256 of the raw token
    user_id        VARCHAR(56)  NOT NULL REFERENCES user_credentials (user_id) ON DELETE CASCADE,
    expires_at     TIMESTAMPTZ  NOT NULL,
    used           BOOLEAN      NOT NULL DEFAULT FALSE,
    created_at     TIMESTAMPTZ  NOT NULL DEFAULT now()
);

CREATE INDEX idx_pw_reset_user    ON password_reset_tokens (user_id);
CREATE INDEX idx_pw_reset_expires ON password_reset_tokens (expires_at);

-- Rate-limit reset attempts per user / IP
CREATE TABLE password_reset_attempts (
    id         BIGSERIAL   PRIMARY KEY,
    identifier VARCHAR(128) NOT NULL,                 -- user_id OR IP address
    attempted_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_pw_attempts_identifier ON password_reset_attempts (identifier, attempted_at DESC);

-- DOWN
DROP INDEX IF EXISTS idx_pw_attempts_identifier;
DROP TABLE  IF EXISTS password_reset_attempts;
DROP INDEX  IF EXISTS idx_pw_reset_expires;
DROP INDEX  IF EXISTS idx_pw_reset_user;
DROP TABLE  IF EXISTS password_reset_tokens;
DROP INDEX  IF EXISTS idx_pw_history_user;
DROP TABLE  IF EXISTS password_history;
DROP TABLE  IF EXISTS user_credentials;
