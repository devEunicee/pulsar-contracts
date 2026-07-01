-- UP
-- Issue #308: Add soft delete support to merchants, payments, and refunds

ALTER TABLE merchants
  ADD COLUMN deleted_at TIMESTAMPTZ DEFAULT NULL;

ALTER TABLE payments
  ADD COLUMN deleted_at TIMESTAMPTZ DEFAULT NULL;

ALTER TABLE refunds
  ADD COLUMN deleted_at TIMESTAMPTZ DEFAULT NULL;

-- Partial indexes so queries filtering soft-deleted rows stay fast
CREATE INDEX idx_merchants_not_deleted ON merchants (address)   WHERE deleted_at IS NULL;
CREATE INDEX idx_payments_not_deleted  ON payments  (order_id)  WHERE deleted_at IS NULL;
CREATE INDEX idx_refunds_not_deleted   ON refunds   (refund_id) WHERE deleted_at IS NULL;

-- DOWN
DROP INDEX IF EXISTS idx_refunds_not_deleted;
DROP INDEX IF EXISTS idx_payments_not_deleted;
DROP INDEX IF EXISTS idx_merchants_not_deleted;

ALTER TABLE refunds   DROP COLUMN IF EXISTS deleted_at;
ALTER TABLE payments  DROP COLUMN IF EXISTS deleted_at;
ALTER TABLE merchants DROP COLUMN IF EXISTS deleted_at;
