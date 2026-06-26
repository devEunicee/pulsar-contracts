# Database Migration System

## Overview

Migrations live in `db/migrations/` as numbered `.sql` files. The runner (`db/migrate.js`) applies them in alphabetical order and tracks history in a `migration_history` table. PostgreSQL advisory locks prevent concurrent runs.

## File Format

Each file must contain an `-- UP` section and a `-- DOWN` section:

```sql
-- UP
CREATE TABLE example (...);

-- DOWN
DROP TABLE IF EXISTS example;
```

## Commands

```bash
cd db && npm install

# Apply all pending migrations
node migrate.js up

# Roll back the last applied migration
node migrate.js down

# Show applied/pending status
node migrate.js status

# Preview pending migrations without applying
node migrate.js dry-run
```

Or via npm scripts:

```bash
npm run migrate:up
npm run migrate:down
npm run migrate:status
npm run migrate:dry-run
```

## Environment

Set `DATABASE_URL` before running:

```bash
export DATABASE_URL=postgres://user:pass@localhost:5432/pulsar
```

## Adding a Migration

1. Create `db/migrations/<NNNN>_description.sql` (increment the prefix).
2. Write `-- UP` and `-- DOWN` sections.
3. Run `node migrate.js up` to apply.
4. Commit the file — migration history is tracked in the database, files are the source of truth in version control.

## CI Integration

The CI workflow runs `node migrate.js dry-run` to verify all migration files parse correctly before merging. See `.github/workflows/ci.yml`.
