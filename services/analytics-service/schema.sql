CREATE TABLE IF NOT EXISTS payments (
  id SERIAL PRIMARY KEY,
  merchant_id TEXT NOT NULL,
  customer_id TEXT NOT NULL,
  amount NUMERIC NOT NULL CHECK (amount > 0),
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW() CHECK (created_at <= NOW()),
  CONSTRAINT uq_payments_merchant_customer_date UNIQUE (merchant_id, customer_id, created_at)
);

CREATE TABLE IF NOT EXISTS refunds (
  id SERIAL PRIMARY KEY,
  payment_id INTEGER NOT NULL,
  merchant_id TEXT NOT NULL,
  amount NUMERIC NOT NULL CHECK (amount > 0),
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  CONSTRAINT fk_refunds_payment_id FOREIGN KEY (payment_id) REFERENCES payments(id) ON DELETE CASCADE,
  CONSTRAINT uq_refunds_payment_id UNIQUE (payment_id),
  CONSTRAINT ck_refunds_amount_valid CHECK (amount <= (SELECT amount FROM payments WHERE id = refunds.payment_id))
);

CREATE INDEX IF NOT EXISTS idx_payments_merchant_id ON payments(merchant_id);
CREATE INDEX IF NOT EXISTS idx_payments_created_at ON payments(created_at);
CREATE INDEX IF NOT EXISTS idx_refunds_merchant_id ON refunds(merchant_id);
CREATE INDEX IF NOT EXISTS idx_refunds_created_at ON refunds(created_at);
