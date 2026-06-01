-- Pulsar event indexer schema

CREATE TABLE IF NOT EXISTS events (
  id            SERIAL PRIMARY KEY,
  ledger        BIGINT NOT NULL,
  tx_hash       TEXT NOT NULL,
  contract_id   TEXT NOT NULL,
  event_type    TEXT NOT NULL,
  topics        JSONB NOT NULL DEFAULT '[]',
  value         JSONB,
  created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_events_event_type  ON events (event_type);
CREATE INDEX IF NOT EXISTS idx_events_ledger       ON events (ledger);
CREATE INDEX IF NOT EXISTS idx_events_contract_id  ON events (contract_id);
