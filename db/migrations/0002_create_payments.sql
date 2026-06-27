-- UP
CREATE TABLE payments (
  order_id         TEXT PRIMARY KEY,
  merchant_address TEXT NOT NULL REFERENCES merchants(address),
  payer            TEXT NOT NULL,
  token            TEXT NOT NULL,
  amount           NUMERIC NOT NULL,
  status           TEXT NOT NULL DEFAULT 'Completed',
  paid_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_payments_merchant ON payments(merchant_address);
CREATE INDEX idx_payments_payer    ON payments(payer);
CREATE INDEX idx_payments_paid_at  ON payments(paid_at);

-- DOWN
DROP INDEX IF EXISTS idx_payments_paid_at;
DROP INDEX IF EXISTS idx_payments_payer;
DROP INDEX IF EXISTS idx_payments_merchant;
DROP TABLE IF EXISTS payments;
