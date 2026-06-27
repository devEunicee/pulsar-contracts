-- Subscription Payment Service Schema

-- Subscriptions table
CREATE TABLE IF NOT EXISTS subscriptions (
  id TEXT PRIMARY KEY,
  merchant_id TEXT NOT NULL,
  customer_id TEXT NOT NULL,
  amount NUMERIC NOT NULL CHECK (amount > 0),
  currency TEXT NOT NULL DEFAULT 'USD',
  frequency TEXT NOT NULL CHECK (frequency IN ('daily', 'weekly', 'biweekly', 'monthly', 'quarterly', 'annually')),
  status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'paused', 'pending_payment', 'past_due', 'cancelled')),
  started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  next_payment_at TIMESTAMPTZ NOT NULL,
  paused_at TIMESTAMPTZ,
  cancelled_at TIMESTAMPTZ,
  cancellation_reason TEXT,
  metadata JSONB DEFAULT '{}',
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_subscriptions_merchant_id ON subscriptions(merchant_id);
CREATE INDEX IF NOT EXISTS idx_subscriptions_customer_id ON subscriptions(customer_id);
CREATE INDEX IF NOT EXISTS idx_subscriptions_status ON subscriptions(status);
CREATE INDEX IF NOT EXISTS idx_subscriptions_next_payment_at ON subscriptions(next_payment_at);

-- Payment attempts table
CREATE TABLE IF NOT EXISTS payment_attempts (
  id TEXT PRIMARY KEY,
  subscription_id TEXT NOT NULL REFERENCES subscriptions(id) ON DELETE CASCADE,
  amount NUMERIC NOT NULL CHECK (amount > 0),
  status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'processing', 'success', 'failed')),
  attempt_number INTEGER NOT NULL DEFAULT 1 CHECK (attempt_number > 0),
  error_message TEXT,
  attempted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  completed_at TIMESTAMPTZ,
  CONSTRAINT ck_payment_timestamp CHECK (completed_at IS NULL OR completed_at >= attempted_at)
);

CREATE INDEX IF NOT EXISTS idx_payment_attempts_subscription_id ON payment_attempts(subscription_id);
CREATE INDEX IF NOT EXISTS idx_payment_attempts_status ON payment_attempts(status);
CREATE INDEX IF NOT EXISTS idx_payment_attempts_attempted_at ON payment_attempts(attempted_at DESC);

-- Invoices table
CREATE TABLE IF NOT EXISTS invoices (
  id TEXT PRIMARY KEY,
  subscription_id TEXT NOT NULL REFERENCES subscriptions(id) ON DELETE CASCADE,
  merchant_id TEXT NOT NULL,
  customer_id TEXT NOT NULL,
  amount NUMERIC NOT NULL CHECK (amount > 0),
  currency TEXT NOT NULL DEFAULT 'USD',
  status TEXT NOT NULL DEFAULT 'issued' CHECK (status IN ('issued', 'sent', 'viewed', 'paid', 'partial', 'refunded', 'voided')),
  invoice_date TIMESTAMPTZ NOT NULL,
  due_date TIMESTAMPTZ NOT NULL,
  paid_at TIMESTAMPTZ,
  invoice_number TEXT NOT NULL UNIQUE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_invoices_subscription_id ON invoices(subscription_id);
CREATE INDEX IF NOT EXISTS idx_invoices_merchant_id ON invoices(merchant_id);
CREATE INDEX IF NOT EXISTS idx_invoices_customer_id ON invoices(customer_id);
CREATE INDEX IF NOT EXISTS idx_invoices_status ON invoices(status);
CREATE INDEX IF NOT EXISTS idx_invoices_due_date ON invoices(due_date);
CREATE INDEX IF NOT EXISTS idx_invoices_invoice_number ON invoices(invoice_number);

-- Subscription events table
CREATE TABLE IF NOT EXISTS subscription_events (
  id TEXT PRIMARY KEY,
  subscription_id TEXT NOT NULL REFERENCES subscriptions(id) ON DELETE CASCADE,
  event_type TEXT NOT NULL,
  data JSONB DEFAULT '{}',
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_subscription_events_subscription_id ON subscription_events(subscription_id);
CREATE INDEX IF NOT EXISTS idx_subscription_events_event_type ON subscription_events(event_type);
CREATE INDEX IF NOT EXISTS idx_subscription_events_created_at ON subscription_events(created_at DESC);

-- View for subscription status dashboard
CREATE OR REPLACE VIEW subscription_status_summary AS
SELECT
  status,
  COUNT(*) as count,
  SUM(amount) as total_amount,
  AVG(amount) as avg_amount,
  MIN(created_at) as oldest,
  MAX(created_at) as newest
FROM subscriptions
GROUP BY status;

-- Function to update subscription next_payment_at
CREATE OR REPLACE FUNCTION calculate_next_payment_date(
  frequency TEXT,
  current_date TIMESTAMPTZ
)
RETURNS TIMESTAMPTZ AS $$
BEGIN
  CASE frequency
    WHEN 'daily' THEN
      RETURN current_date + INTERVAL '1 day';
    WHEN 'weekly' THEN
      RETURN current_date + INTERVAL '7 days';
    WHEN 'biweekly' THEN
      RETURN current_date + INTERVAL '14 days';
    WHEN 'monthly' THEN
      RETURN current_date + INTERVAL '30 days';
    WHEN 'quarterly' THEN
      RETURN current_date + INTERVAL '90 days';
    WHEN 'annually' THEN
      RETURN current_date + INTERVAL '365 days';
    ELSE
      RETURN current_date + INTERVAL '30 days';
  END CASE;
END;
$$ LANGUAGE plpgsql;

-- Function to mark subscription as past due
CREATE OR REPLACE FUNCTION mark_past_due_subscriptions()
RETURNS void AS $$
BEGIN
  UPDATE subscriptions
  SET status = 'past_due'
  WHERE status = 'pending_payment'
  AND next_payment_at < NOW() - INTERVAL '24 hours';
END;
$$ LANGUAGE plpgsql;

-- Function to get subscription metrics
CREATE OR REPLACE FUNCTION get_subscription_metrics(
  days_back INTEGER DEFAULT 30
)
RETURNS TABLE (
  total_subscriptions BIGINT,
  active_subscriptions BIGINT,
  paused_subscriptions BIGINT,
  cancelled_subscriptions BIGINT,
  total_revenue NUMERIC,
  new_subscriptions_last_period BIGINT
) AS $$
BEGIN
  RETURN QUERY
  SELECT
    (SELECT COUNT(*) FROM subscriptions),
    (SELECT COUNT(*) FROM subscriptions WHERE status = 'active'),
    (SELECT COUNT(*) FROM subscriptions WHERE status = 'paused'),
    (SELECT COUNT(*) FROM subscriptions WHERE status = 'cancelled'),
    (SELECT SUM(amount) FROM subscriptions WHERE status = 'active'),
    (SELECT COUNT(*) FROM subscriptions WHERE created_at > NOW() - (days_back || ' days')::INTERVAL)
  ;
END;
$$ LANGUAGE plpgsql;
