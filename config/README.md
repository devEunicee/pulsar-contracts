# Environment Configuration

This directory contains per-environment configuration files for the payment processing contract.

## Files

| File | Environment | Safe to commit? |
|------|-------------|-----------------|
| `local.toml` | Local standalone node | ✅ Yes |
| `testnet.toml` | Stellar Testnet | ✅ Yes |
| `mainnet.toml` | Stellar Mainnet | ✅ Yes |

## Public vs Secret values

**Public (committed in TOML files):**
- `network.rpc_url` — RPC endpoint URL
- `network.network_passphrase` — Network identifier string
- `network.horizon_url` — Horizon REST API URL
- `contract.contract_id` — Deployed contract address (update after each deploy)
- `admin.admin_public_key` — Admin account public key (G...)
- `token.native_token_address` — Token contract address

**Secret (environment variables only — never committed):**
- `ADMIN_SECRET_KEY` — Stellar secret key (S...) for the admin account

## Usage

Scripts should read the target environment's TOML file and merge with environment
variables for secrets. Select the environment via the `STELLAR_ENV` variable:

```bash
# Deploy to testnet
STELLAR_ENV=testnet ADMIN_SECRET_KEY=$MY_SECRET ./scripts/deploy.sh

# Deploy to mainnet
STELLAR_ENV=mainnet ADMIN_SECRET_KEY=$MY_SECRET ./scripts/deploy.sh
```

A minimal shell helper to load config values:

```bash
#!/usr/bin/env bash
ENV="${STELLAR_ENV:-local}"
CONFIG_FILE="config/${ENV}.toml"

if [ ! -f "$CONFIG_FILE" ]; then
  echo "Error: config file not found: $CONFIG_FILE" >&2
  exit 1
fi

# Parse a value from the TOML file (requires `tomlq` or `dasel`)
rpc_url=$(dasel -f "$CONFIG_FILE" -r toml 'network.rpc_url')
contract_id=$(dasel -f "$CONFIG_FILE" -r toml 'contract.contract_id')
```

## Adding a new environment

1. Copy `testnet.toml` to `<env>.toml`
2. Update all values for the new environment
3. Add the new file to the table above
4. Never add secret values to the TOML file
