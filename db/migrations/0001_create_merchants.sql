-- UP
CREATE TABLE merchants (
  address      TEXT PRIMARY KEY,
  name         TEXT NOT NULL,
  description  TEXT,
  contact_info TEXT,
  category     TEXT NOT NULL,
  active       BOOLEAN NOT NULL DEFAULT true,
  created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- DOWN
DROP TABLE IF EXISTS merchants;
