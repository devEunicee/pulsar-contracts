#!/usr/bin/env bash
# Smoke test for the deployed Pulsar payment-processing contract.
# Required env vars: CONTRACT_ID, ADMIN_SECRET, NETWORK
set -euo pipefail

: "${CONTRACT_ID:?CONTRACT_ID is required}"
: "${ADMIN_SECRET:?ADMIN_SECRET is required}"
: "${NETWORK:?NETWORK is required}"

ADMIN_ADDRESS=$(stellar keys address --secret-key "$ADMIN_SECRET" 2>/dev/null || \
  stellar keys address deployer 2>/dev/null || \
  stellar keys generate --global smoke-admin --secret-key "$ADMIN_SECRET" && stellar keys address smoke-admin)

invoke() {
  stellar contract invoke \
    --id "$CONTRACT_ID" \
    --source-account "$ADMIN_SECRET" \
    --network "$NETWORK" \
    -- "$@"
}

echo "==> [1/4] set_admin"
invoke set_admin \
  --admins "[\"$ADMIN_ADDRESS\"]" \
  --threshold 1

echo "==> [2/4] register_merchant"
invoke register_merchant \
  --merchant_address "$ADMIN_ADDRESS" \
  --name "SmokeStore" \
  --description "Smoke test merchant" \
  --contact_info "smoke@test.local" \
  --category Retail \
  --signing_public_key null

echo "==> [3/4] get_merchant"
invoke get_merchant --merchant_address "$ADMIN_ADDRESS"

echo "==> [4/4] get_global_payment_stats"
invoke get_global_payment_stats \
  --admins "[\"$ADMIN_ADDRESS\"]" \
  --date_start null \
  --date_end null

echo "✅ Smoke test passed"
