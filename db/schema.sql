-- Pulsar Off-Chain Database Schema
-- Issue #293: Design Database Schema for Off-Chain Data

-- ── Merchants ─────────────────────────────────────────────────────────────────

CREATE TABLE merchants (
    address         VARCHAR(56)  PRIMARY KEY,          -- Stellar account ID (G...)
    name            VARCHAR(255) NOT NULL,
    description     TEXT,
    contact_info    VARCHAR(255),
    category        VARCHAR(50)  NOT NULL
                    CHECK (category IN ('Retail','Food','Services','Digital','Other')),
    active          BOOLEAN      NOT NULL DEFAULT TRUE,
    whitelisted     BOOLEAN      NOT NULL DEFAULT FALSE,
    registered_at   BIGINT       NOT NULL,             -- Unix timestamp (seconds)
    updated_at      BIGINT       NOT NULL
);

CREATE INDEX idx_merchants_category ON merchants (category);
CREATE INDEX idx_merchants_active   ON merchants (active);
CREATE INDEX idx_merchants_name     ON merchants (name);

-- ── Payments ──────────────────────────────────────────────────────────────────

CREATE TABLE payments (
    order_id          VARCHAR(128) PRIMARY KEY,
    merchant_address  VARCHAR(56)  NOT NULL REFERENCES merchants (address),
    payer_address     VARCHAR(56)  NOT NULL,
    token_address     VARCHAR(56)  NOT NULL,
    amount            NUMERIC(38,0) NOT NULL,
    refunded_amount   NUMERIC(38,0) NOT NULL DEFAULT 0,
    status            VARCHAR(30)  NOT NULL
                      CHECK (status IN ('Completed','PartiallyRefunded','FullyRefunded')),
    description       TEXT,
    paid_at           BIGINT       NOT NULL,
    idempotency_key   VARCHAR(128) UNIQUE
);

CREATE INDEX idx_payments_merchant  ON payments (merchant_address);
CREATE INDEX idx_payments_payer     ON payments (payer_address);
CREATE INDEX idx_payments_paid_at   ON payments (paid_at);
CREATE INDEX idx_payments_status    ON payments (status);
CREATE INDEX idx_payments_amount    ON payments (amount);
CREATE INDEX idx_payments_token     ON payments (token_address);

-- ── Refunds ───────────────────────────────────────────────────────────────────

CREATE TABLE refunds (
    refund_id      VARCHAR(128) PRIMARY KEY,
    order_id       VARCHAR(128) NOT NULL REFERENCES payments (order_id),
    amount         NUMERIC(38,0) NOT NULL,
    reason         TEXT,
    status         VARCHAR(20)  NOT NULL
                   CHECK (status IN ('Pending','Approved','Rejected','Completed')),
    initiated_by   VARCHAR(56)  NOT NULL,
    initiated_at   BIGINT       NOT NULL,
    resolved_at    BIGINT
);

CREATE INDEX idx_refunds_order_id    ON refunds (order_id);
CREATE INDEX idx_refunds_status      ON refunds (status);
CREATE INDEX idx_refunds_initiated_at ON refunds (initiated_at);

-- ── Subscriptions ─────────────────────────────────────────────────────────────

CREATE TABLE subscriptions (
    subscription_id  VARCHAR(128) PRIMARY KEY,
    merchant_address VARCHAR(56)  NOT NULL REFERENCES merchants (address),
    payer_address    VARCHAR(56)  NOT NULL,
    token_address    VARCHAR(56)  NOT NULL,
    amount           NUMERIC(38,0) NOT NULL,
    interval_seconds BIGINT       NOT NULL,            -- recurrence period
    next_payment_at  BIGINT       NOT NULL,
    active           BOOLEAN      NOT NULL DEFAULT TRUE,
    created_at       BIGINT       NOT NULL
);

CREATE INDEX idx_subscriptions_merchant      ON subscriptions (merchant_address);
CREATE INDEX idx_subscriptions_payer         ON subscriptions (payer_address);
CREATE INDEX idx_subscriptions_next_payment  ON subscriptions (next_payment_at);
CREATE INDEX idx_subscriptions_active        ON subscriptions (active);

-- ── Webhooks ──────────────────────────────────────────────────────────────────

CREATE TABLE webhooks (
    webhook_id       VARCHAR(128) PRIMARY KEY,
    merchant_address VARCHAR(56)  NOT NULL REFERENCES merchants (address),
    url              TEXT         NOT NULL,
    events           TEXT         NOT NULL,            -- comma-separated event names
    secret           VARCHAR(255) NOT NULL,            -- HMAC signing secret
    active           BOOLEAN      NOT NULL DEFAULT TRUE,
    created_at       BIGINT       NOT NULL
);

CREATE INDEX idx_webhooks_merchant ON webhooks (merchant_address);
CREATE INDEX idx_webhooks_active   ON webhooks (active);

-- ── Merchant Audit Trail ──────────────────────────────────────────────────────

CREATE TABLE merchant_audit_log (
    id             BIGSERIAL    PRIMARY KEY,
    merchant_address VARCHAR(56) NOT NULL REFERENCES merchants (address),
    action         VARCHAR(50)  NOT NULL,              -- e.g. 'registered','deactivated','updated'
    changed_by     VARCHAR(56)  NOT NULL,
    changed_at     BIGINT       NOT NULL,
    details        TEXT                                -- JSON snapshot of changed fields
);

CREATE INDEX idx_audit_merchant   ON merchant_audit_log (merchant_address);
CREATE INDEX idx_audit_changed_at ON merchant_audit_log (changed_at);

-- ── Idempotency Keys ──────────────────────────────────────────────────────────

CREATE TABLE idempotency_keys (
    idempotency_key  VARCHAR(128) PRIMARY KEY,
    operation        VARCHAR(50)  NOT NULL,            -- 'payment' | 'refund'
    request_hash     VARCHAR(64)  NOT NULL,
    response_body    TEXT,
    created_at       BIGINT       NOT NULL,
    expires_at       BIGINT       NOT NULL
);

CREATE INDEX idx_idempotency_expires_at ON idempotency_keys (expires_at);
