-- UP: Row-Level Security policies
-- Issue #306: Ensure users can only access their own data

-- Enable RLS on core tables
ALTER TABLE payments ENABLE ROW LEVEL SECURITY;
ALTER TABLE refunds ENABLE ROW LEVEL SECURITY;
ALTER TABLE merchants ENABLE ROW LEVEL SECURITY;
ALTER TABLE merchant_audit_log ENABLE ROW LEVEL SECURITY;

-- ── payments ──────────────────────────────────────────────────────────────────
-- Admins see all rows; merchants see their own; payers see their own.
CREATE POLICY payments_admin
  ON payments FOR ALL
  TO pulsar_admin
  USING (true);

CREATE POLICY payments_merchant
  ON payments FOR SELECT
  TO pulsar_merchant
  USING (merchant_address = current_setting('app.current_user', true));

CREATE POLICY payments_payer
  ON payments FOR SELECT
  TO pulsar_customer
  USING (payer_address = current_setting('app.current_user', true));

-- ── refunds ───────────────────────────────────────────────────────────────────
CREATE POLICY refunds_admin
  ON refunds FOR ALL
  TO pulsar_admin
  USING (true);

CREATE POLICY refunds_merchant
  ON refunds FOR SELECT
  TO pulsar_merchant
  USING (
    order_id IN (
      SELECT order_id FROM payments
      WHERE merchant_address = current_setting('app.current_user', true)
    )
  );

CREATE POLICY refunds_payer
  ON refunds FOR SELECT
  TO pulsar_customer
  USING (initiated_by = current_setting('app.current_user', true));

-- ── merchants ─────────────────────────────────────────────────────────────────
CREATE POLICY merchants_admin
  ON merchants FOR ALL
  TO pulsar_admin
  USING (true);

CREATE POLICY merchants_self
  ON merchants FOR SELECT
  TO pulsar_merchant
  USING (address = current_setting('app.current_user', true));

-- Customers can view active merchants (public directory)
CREATE POLICY merchants_public
  ON merchants FOR SELECT
  TO pulsar_customer
  USING (active = true);

-- ── merchant_audit_log ────────────────────────────────────────────────────────
CREATE POLICY audit_admin
  ON merchant_audit_log FOR ALL
  TO pulsar_admin
  USING (true);

CREATE POLICY audit_merchant_own
  ON merchant_audit_log FOR SELECT
  TO pulsar_merchant
  USING (merchant_address = current_setting('app.current_user', true));

-- ── access_attempts audit ─────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS rls_access_log (
  id            BIGSERIAL PRIMARY KEY,
  table_name    TEXT        NOT NULL,
  operation     TEXT        NOT NULL,
  user_address  TEXT        NOT NULL,
  user_role     TEXT        NOT NULL,
  accessed_at   BIGINT      NOT NULL,
  allowed       BOOLEAN     NOT NULL
);

CREATE INDEX idx_rls_access_log_user ON rls_access_log (user_address);
CREATE INDEX idx_rls_access_log_at   ON rls_access_log (accessed_at);

-- DOWN
-- ALTER TABLE payments  DISABLE ROW LEVEL SECURITY;
-- ALTER TABLE refunds   DISABLE ROW LEVEL SECURITY;
-- ALTER TABLE merchants DISABLE ROW LEVEL SECURITY;
-- ALTER TABLE merchant_audit_log DISABLE ROW LEVEL SECURITY;
-- DROP POLICY IF EXISTS payments_admin     ON payments;
-- DROP POLICY IF EXISTS payments_merchant  ON payments;
-- DROP POLICY IF EXISTS payments_payer     ON payments;
-- DROP POLICY IF EXISTS refunds_admin      ON refunds;
-- DROP POLICY IF EXISTS refunds_merchant   ON refunds;
-- DROP POLICY IF EXISTS refunds_payer      ON refunds;
-- DROP POLICY IF EXISTS merchants_admin    ON merchants;
-- DROP POLICY IF EXISTS merchants_self     ON merchants;
-- DROP POLICY IF EXISTS merchants_public   ON merchants;
-- DROP POLICY IF EXISTS audit_admin        ON merchant_audit_log;
-- DROP POLICY IF EXISTS audit_merchant_own ON merchant_audit_log;
-- DROP TABLE IF EXISTS rls_access_log;
