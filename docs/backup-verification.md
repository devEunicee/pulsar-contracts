# Automated Backup Verification

The `.github/workflows/backup-verification.yml` workflow verifies that database backups are valid and restorable every day after the nightly backup completes.

## Schedule

- **Backup**: daily at 02:00 UTC (`db-backup.yml`)
- **Verification**: daily at 04:00 UTC (`backup-verification.yml`)
- **Manual trigger**: `workflow_dispatch` with optional `backup_key` input to verify a specific backup

## What the Workflow Does

1. **Resolves** the latest backup key from S3 (or uses the provided key).
2. **Downloads** the backup file from S3.
3. **Integrity check** — runs `pg_restore --list` and asserts at least one `TABLE DATA` entry exists.
4. **Restore drill** — spins up an ephemeral PostgreSQL instance and restores the backup.
5. **Data validation** — queries `merchants` and `payments` row counts.
6. **RTO validation** — asserts restore completed within 5 minutes (configurable via `RTO_LIMIT`).
7. **Alert** — if any step fails, automatically opens a GitHub issue labelled `bug` + `reliability`.

## RTO/RPO Targets

| Target | Value |
|--------|-------|
| RTO (Recovery Time Objective) | ≤ 5 minutes |
| RPO (Recovery Point Objective) | ≤ 24 hours (daily backup cadence) |

## Required Secrets

| Secret | Purpose |
|--------|---------|
| `DATABASE_URL` | Source database (for reference; not used during restore drill) |
| `AWS_S3_BUCKET` | Bucket containing backups |
| `AWS_DEFAULT_REGION` | AWS region (default: `us-east-1`) |
| `AWS_ACCESS_KEY_ID` | AWS credentials for S3 access |
| `AWS_SECRET_ACCESS_KEY` | AWS credentials for S3 access |

## Manual Restore Procedure

To restore from a specific backup manually:

```bash
# Download the backup
aws s3 cp s3://<BUCKET>/pulsar-db-backup-<TIMESTAMP>.dump /tmp/restore.dump

# Verify integrity
pg_restore --list /tmp/restore.dump

# Restore to target database
pg_restore --no-owner --no-privileges --dbname "$DATABASE_URL" /tmp/restore.dump
```

See `scripts/db-backup.sh` for the backup creation script.
