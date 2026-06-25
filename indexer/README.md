# Pulsar Event Indexer

Off-chain service that streams Pulsar contract events from Stellar's RPC node into PostgreSQL and exposes a REST API for queries.

## Architecture

```
Stellar RPC (getEvents) → Indexer (Node.js) → PostgreSQL → REST API
```

## Setup

### 1. Database

```bash
psql -U postgres -c "CREATE DATABASE pulsar_events;"
psql -U postgres -d pulsar_events -f schema.sql
```

### 2. Environment

```bash
cp .env.example .env
# Edit .env with your CONTRACT_ID and DATABASE_URL
```

### 3. Install & run

```bash
npm install
npm start
```

## Schema

| Column       | Type        | Description                        |
|--------------|-------------|------------------------------------|
| id           | SERIAL      | Auto-increment primary key         |
| ledger       | BIGINT      | Ledger sequence number             |
| tx_hash      | TEXT        | Transaction hash                   |
| contract_id  | TEXT        | Contract address                   |
| event_type   | TEXT        | Event name (e.g. `payment_processed`) |
| topics       | JSONB       | Full topic array                   |
| value        | JSONB       | Event value payload                |
| created_at   | TIMESTAMPTZ | Row insertion time                 |

## REST API

| Endpoint            | Description                              |
|---------------------|------------------------------------------|
| `GET /events`       | List events (query: `type`, `limit`, `offset`) |
| `GET /events/:type` | List events filtered by type             |

### Example

```bash
# All payment_processed events
curl http://localhost:3001/events/payment_processed

# Paginated
curl "http://localhost:3001/events?limit=20&offset=0"
```

## Indexed Event Types

See the [Events table in the main README](../README.md#events) for all emitted event types.
