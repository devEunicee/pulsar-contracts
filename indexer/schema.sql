-- Pulsar event indexer schema

CREATE TABLE IF NOT EXISTS events (
  id            SERIAL PRIMARY KEY,
  ledger        BIGINT NOT NULL CHECK (ledger >= 0),
  tx_hash       TEXT NOT NULL CHECK (tx_hash != ''),
  contract_id   TEXT NOT NULL CHECK (contract_id != ''),
  event_type    TEXT NOT NULL CHECK (event_type != ''),
  topics        JSONB NOT NULL DEFAULT '[]' CHECK (jsonb_typeof(topics) = 'array'),
  value         JSONB,
  created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  CONSTRAINT uq_events_ledger_tx_contract_type UNIQUE (ledger, tx_hash, contract_id, event_type)
);

CREATE INDEX IF NOT EXISTS idx_events_event_type  ON events (event_type);
CREATE INDEX IF NOT EXISTS idx_events_ledger       ON events (ledger);
CREATE INDEX IF NOT EXISTS idx_events_contract_id  ON events (contract_id);
