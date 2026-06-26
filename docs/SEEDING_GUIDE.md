# Environment Seeding Guide

This guide explains how to use the seeding script to quickly populate a local or testnet environment with sample merchants, payments, and refunds for manual testing.

## Overview

The seeding script (`scripts/seed.sh`) automates the setup of a test environment by:

1. Registering multiple merchants with different categories
2. Processing payments between payers and merchants
3. Initiating refunds for testing the refund workflow

This is useful for:
- Manual testing of contract functionality
- Integration testing with external systems
- Demo environments
- Development and debugging

## Prerequisites

### Required Tools

- **Stellar CLI** — Command-line interface for Stellar network
  - Install: https://developers.stellar.org/docs/tools/stellar-cli
  - Verify: `stellar --version`

- **Bash** — Shell script interpreter
  - Available on macOS, Linux, and Windows (WSL)

### Required Accounts

You need the following accounts set up in Stellar CLI:

- **Admin account** — Initializes the contract
- **Token issuer account** — Issues test tokens
- **Payer account** — Initiates payments
- **Merchant accounts** — Receive payments (created automatically)

Create accounts with:
```bash
stellar keys generate admin
stellar keys generate token_issuer
stellar keys generate payer
```

### Deployed Contract

You need a deployed contract ID. If you haven't deployed yet:

```bash
# Build the contract
cd contracts/payment-processing-contract
cargo build --target wasm32-unknown-unknown --release

# Deploy to local network
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/payment_processing_contract.wasm \
  --source-account admin \
  --network local
```

Save the returned contract ID for use with the seeding script.

## Configuration

The seeding script reads configuration from `config/local.toml`. Edit this file to customize:

### Network Configuration

```toml
[network]
name = "local"  # or "testnet", "public"
```

### Merchant Configuration

```toml
[merchants]
count = 3                                    # Number of merchants to create
categories = ["Retail", "Food", "Services"]  # Categories to assign
name_prefix = "Test Merchant"                # Merchant name prefix
```

### Payment Configuration

```toml
[payments]
count = 10                    # Number of payments to process
amount_min = 100000          # Minimum payment amount (stroops)
amount_max = 500000          # Maximum payment amount (stroops)
description_prefix = "Test payment"
```

### Refund Configuration

```toml
[refunds]
count = 2                    # Number of refunds to initiate
amount_percentage = 50       # Refund as % of original payment
reason = "Test refund"
```

## Usage

### Basic Usage

```bash
# Run with default config (config/local.toml)
bash scripts/seed.sh

# Run with custom config
bash scripts/seed.sh config/testnet.toml
```

### Step-by-Step

1. **Ensure contract is deployed**:
   ```bash
   export CONTRACT_ID="CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4"
   ```

2. **Run the seeding script**:
   ```bash
   bash scripts/seed.sh config/local.toml
   ```

3. **Follow the prompts** (if needed)

4. **Verify the results**:
   ```bash
   # Query merchant stats
   stellar contract invoke --id $CONTRACT_ID --source-account admin --network local \
     -- get_merchant_stats --merchant <MERCHANT_ADDRESS> --date_start null --date_end null
   ```

## What Gets Created

### Merchants

The script creates 3 merchants with different categories:

| Merchant | Category | Address |
|----------|----------|---------|
| Test Merchant 1 | Retail | merchant_1 |
| Test Merchant 2 | Food | merchant_2 |
| Test Merchant 3 | Services | merchant_3 |

### Payments

10 payments are processed:

- **Payer**: payer account
- **Merchants**: Cycled through the 3 merchants
- **Amounts**: 100,000 to 200,000 stroops (increasing)
- **Order IDs**: ORDER_SEED_001 through ORDER_SEED_010

### Refunds

2 refunds are initiated:

- **Refund 1**: 50% of ORDER_SEED_001
- **Refund 2**: 50% of ORDER_SEED_002
- **Status**: Pending (awaiting merchant approval)

## Querying Results

After seeding, you can query the contract to verify the data:

### Global Stats

```bash
stellar contract invoke --id $CONTRACT_ID --source-account admin --network local \
  -- get_global_payment_stats \
  --admins '["<ADMIN_ADDRESS>"]' \
  --date_start null \
  --date_end null
```

Expected output:
- total_payments: 10
- total_volume: 1,550,000 stroops
- total_refunds: 2
- total_refund_volume: 150,000 stroops

### Merchant Stats

```bash
stellar contract invoke --id $CONTRACT_ID --source-account merchant_1 --network local \
  -- get_merchant_stats \
  --merchant <MERCHANT_1_ADDRESS> \
  --date_start null \
  --date_end null
```

Expected output:
- total_payments: 4 (payments 1, 4, 7, 10)
- total_volume: 430,000 stroops
- total_refunds: 1 (refund for payment 1)
- total_refund_volume: 50,000 stroops

### Payment History

```bash
stellar contract invoke --id $CONTRACT_ID --source-account payer --network local \
  -- get_payer_payment_history \
  --payer <PAYER_ADDRESS> \
  --cursor null \
  --limit 10 \
  --filter null \
  --sort_field Date \
  --sort_order Descending
```

## Troubleshooting

### "Stellar CLI not found"

Install Stellar CLI:
```bash
# macOS
brew install stellar-cli

# Linux
curl -L https://github.com/stellar/stellar-cli/releases/download/v21.0.0/stellar-cli-21.0.0-x86_64-unknown-linux-gnu.tar.gz | tar xz
sudo mv stellar /usr/local/bin/

# Windows (WSL)
# Use Linux instructions above
```

### "Account not found"

Create the account:
```bash
stellar keys generate <account_name>
```

### "Contract ID is required"

Set the contract ID:
```bash
export CONTRACT_ID="CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4"
bash scripts/seed.sh
```

Or pass it when prompted.

### "Admin already initialized"

This is normal if you've run the script before. The script will continue with merchant registration.

### Payments fail with signature errors

This is expected in the current implementation. The script uses dummy signatures for demonstration. In production, you would:

1. Generate proper ed25519 signatures
2. Use the merchant's actual signing key
3. Sign the payment order with the merchant's private key

For testing purposes, the dummy signatures allow the script to demonstrate the contract's functionality.

## Advanced Usage

### Custom Configuration

Create a new config file for different scenarios:

```toml
# config/testnet.toml
[network]
name = "testnet"

[merchants]
count = 5

[payments]
count = 20

[refunds]
count = 5
```

Then run:
```bash
bash scripts/seed.sh config/testnet.toml
```

### Scripting Multiple Environments

```bash
#!/bin/bash
# Seed multiple environments

for env in local testnet; do
    echo "Seeding $env..."
    bash scripts/seed.sh config/$env.toml
done
```

### Automated Testing

Use the seeding script in CI/CD pipelines:

```yaml
# .github/workflows/integration-test.yml
- name: Seed test environment
  run: bash scripts/seed.sh config/local.toml
  env:
    CONTRACT_ID: ${{ secrets.CONTRACT_ID }}
```

## References

- [Stellar CLI Documentation](https://developers.stellar.org/docs/tools/stellar-cli)
- [Soroban Contract Invocation](https://developers.stellar.org/docs/learn/soroban)
- [Payment Processing Contract API](../README.md#contract-api)
