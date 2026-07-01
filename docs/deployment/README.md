# Pulsar — Multi-Environment Deployment Guide

> Deployment configurations for development, staging, and production environments.

---

## Table of Contents

1. [Overview](#1-overview)
2. [Development Environment](#2-development-environment)
3. [Staging Environment](#3-staging-environment)
4. [Production Environment (HA)](#4-production-environment-ha)
5. [Secrets Management](#5-secrets-management)
6. [Monitoring per Environment](#6-monitoring-per-environment)
7. [Backup Strategy](#7-backup-strategy)
8. [DNS Setup](#8-dns-setup)

---

## 1. Overview

| Environment | Network | Purpose |
|---|---|---|
| Development | Local Docker | Local development and unit testing |
| Staging | Stellar Testnet | Integration testing with real-data volumes |
| Production | Stellar Mainnet | Live traffic; high-availability setup |

Environment configs live in `config/environments/`:

```
config/environments/
├── development.toml   # local Docker network
├── staging.toml       # Stellar testnet
└── production.toml    # Stellar mainnet (HA)
```

---

## 2. Development Environment

### Setup

```bash
# 1. Start local Stellar network
docker compose up -d

# 2. Build the contract
cargo build --target wasm32-unknown-unknown --release

# 3. Deploy
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/payment_processing_contract.wasm \
  --source-account <DEV_SECRET_KEY> \
  --network local

export CONTRACT_ID="<returned ID>"

# 4. Initialize
stellar contract invoke --id $CONTRACT_ID --source-account <DEV_SECRET_KEY> --network local \
  -- set_admin --admin <ADMIN_ADDRESS>

# 5. Seed test data
bash scripts/seed.sh config/environments/development.toml
```

### Verify

```bash
stellar contract invoke --id $CONTRACT_ID --source-account <KEY> --network local \
  -- ping
```

Horizon available at `http://localhost:8000`.

---

## 3. Staging Environment

### Setup

```bash
# Fund accounts via Friendbot
stellar keys generate --global staging-admin
curl "https://friendbot.stellar.org?addr=$(stellar keys address staging-admin)"

# Build and deploy to testnet
cargo build --target wasm32-unknown-unknown --release
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/payment_processing_contract.wasm \
  --source-account <STAGING_ADMIN_SECRET> \
  --network testnet

export STAGING_CONTRACT_ID="<returned ID>"

# Initialize
stellar contract invoke --id $STAGING_CONTRACT_ID \
  --source-account <STAGING_ADMIN_SECRET> --network testnet \
  -- set_admin --admin <STAGING_ADMIN_ADDRESS>

# Seed with staging data
bash scripts/seed.sh config/environments/staging.toml
```

### CI/CD Integration

Set these secrets in your CI/CD pipeline (GitHub Actions example):

```yaml
# .github/workflows/deploy-staging.yml
env:
  STAGING_ADMIN_SECRET: ${{ secrets.STAGING_ADMIN_SECRET }}
  STAGING_CONTRACT_ID: ${{ secrets.STAGING_CONTRACT_ID }}
  MONITORING_ENDPOINT: ${{ secrets.STAGING_MONITORING_ENDPOINT }}
  BACKUP_BUCKET: ${{ secrets.STAGING_BACKUP_BUCKET }}
```

---

## 4. Production Environment (HA)

### Architecture

```
                    ┌─────────────┐
Clients ──────────► │ Load Balancer│
                    └──────┬──────┘
                           │
              ┌────────────┴────────────┐
              │                         │
       ┌──────▼──────┐          ┌───────▼─────┐
       │ API Gateway  │          │ API Gateway  │
       │  (primary)   │          │  (secondary) │
       └──────┬───────┘          └──────┬───────┘
              │                         │
       ┌──────▼─────────────────────────▼──────┐
       │          Stellar Mainnet RPC           │
       │  (primary + backup endpoint failover)  │
       └────────────────────────────────────────┘
```

### Deployment

```bash
# 1. Ensure admin account is funded on mainnet
stellar keys generate --global prod-admin
# Fund via exchange or transfer — Friendbot is testnet only

# 2. Build release WASM
cargo build --target wasm32-unknown-unknown --release

# 3. Deploy to mainnet
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/payment_processing_contract.wasm \
  --source-account <PROD_ADMIN_SECRET> \
  --network public

export PROD_CONTRACT_ID="<returned ID>"

# 4. Initialize admin
stellar contract invoke --id $PROD_CONTRACT_ID \
  --source-account <PROD_ADMIN_SECRET> --network public \
  -- set_admin --admin <PROD_ADMIN_ADDRESS>
```

### Upgrade Procedure (zero-downtime)

```bash
# 1. Build new WASM
cargo build --target wasm32-unknown-unknown --release

# 2. Upload WASM to network (does not affect live contract)
NEW_HASH=$(stellar contract upload \
  --wasm target/wasm32-unknown-unknown/release/payment_processing_contract.wasm \
  --source-account <PROD_ADMIN_SECRET> \
  --network public)

# 3. Apply upgrade (in-place, address unchanged)
stellar contract invoke --id $PROD_CONTRACT_ID \
  --source-account <PROD_ADMIN_SECRET> --network public \
  -- upgrade \
  --admin <PROD_ADMIN_ADDRESS> \
  --new_wasm_hash $NEW_HASH

# 4. Verify
stellar contract invoke --id $PROD_CONTRACT_ID \
  --source-account <PROD_ADMIN_SECRET> --network public \
  -- get_version
```

---

## 5. Secrets Management

**Never commit secrets to source control.**

### Recommended Tools

| Tool | Use Case |
|---|---|
| AWS Secrets Manager | Cloud-native; auto-rotation support |
| HashiCorp Vault | Self-hosted; fine-grained policies |
| GitHub Actions Secrets | CI/CD pipeline secrets |
| Docker secrets | Container-level injection |

### Environment Variables

All sensitive values are injected at runtime via environment variables:

| Variable | Description |
|---|---|
| `ADMIN_SECRET_KEY` | Admin Stellar secret key |
| `CONTRACT_ID` | Deployed contract address |
| `MONITORING_ENDPOINT` | Metrics/alerting endpoint URL |
| `BACKUP_BUCKET` | S3-compatible bucket name |
| `BACKUP_RPC_URL` | Failover Stellar RPC endpoint |
| `ALERTING_KEY` | PagerDuty / OpsGenie integration key |
| `TLS_CERT_ARN` | TLS certificate ARN (ACM) |

### .env Example (development only)

See `.env.example` in the repo root. Copy to `.env` and fill in values:

```bash
cp .env.example .env
# Edit .env — never commit this file
```

`.env` is listed in `.gitignore`.

---

## 6. Monitoring per Environment

| Environment | Tool | Alerts |
|---|---|---|
| Development | None (local logs) | None |
| Staging | Prometheus + Grafana | Slack notification on 3+ consecutive failures |
| Production | DataDog / CloudWatch | PagerDuty on first failure; 60s uptime checks |

### Key Metrics to Monitor

- `payment_processed` event rate (payments per minute)
- `refund_initiated` / `refund_executed` rate
- Contract invocation error rate (by error code)
- Admin account XLM balance (alert if < 1 XLM)
- Soroban RPC latency (p50, p99)

### Health Check

```bash
# Ping contract — returns current ledger timestamp
stellar contract invoke --id $CONTRACT_ID --source-account <KEY> --network <NETWORK> \
  -- ping
```

Automate this check every 60s in production.

---

## 7. Backup Strategy

### What to Back Up

| Data | Location | Criticality |
|---|---|---|
| Contract WASM | Build artifact + upload hash | High — needed for upgrades |
| Contract ID | Deployment record | Critical — cannot be recovered |
| Admin keypair | Secrets manager | Critical — controls contract |
| Merchant keypairs | Merchant custody | High |
| Off-chain event index | Your indexer DB | Medium |

### Backup Schedule

| Environment | Frequency | Retention |
|---|---|---|
| Development | None | N/A |
| Staging | Daily at 02:00 UTC | 14 days |
| Production | Daily at 01:00 UTC | 90 days + cross-region replica |

### Restore Procedure

On-chain state (payments, merchants, refunds) lives on the Stellar ledger — it cannot be lost as long as the network is live and TTLs are maintained. The key assets to protect are:

1. **Contract ID** — record in multiple secure locations immediately after deployment
2. **Admin private key** — store in secrets manager with access audit logging
3. **WASM hash** — tag your container image or artifact store with the deployed hash

---

## 8. DNS Setup

### Staging

| Record | Type | Value |
|---|---|---|
| `api-staging.yourapp.com` | CNAME | Load balancer hostname |

### Production

| Record | Type | Value |
|---|---|---|
| `api.yourapp.com` | A / CNAME | Load balancer hostname |
| `api.yourapp.com` | TXT | SPF/DMARC for email notifications |

### TLS

Use ACM (AWS) or Let's Encrypt for TLS certificates. Enforce HTTPS-only — redirect HTTP → HTTPS at the load balancer.

```nginx
# nginx — force HTTPS
server {
    listen 80;
    server_name api.yourapp.com;
    return 301 https://$host$request_uri;
}
```
