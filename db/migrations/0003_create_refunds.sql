-- UP
CREATE TABLE refunds (
  refund_id  TEXT PRIMARY KEY,
  order_id   TEXT NOT NULL REFERENCES payments(order_id),
  amount     NUMERIC NOT NULL,
  reason     TEXT,
  status     TEXT NOT NULL DEFAULT 'Pending',
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_refunds_order ON refunds(order_id);

-- DOWN
DROP INDEX IF EXISTS idx_refunds_order;
DROP TABLE IF EXISTS refunds;
