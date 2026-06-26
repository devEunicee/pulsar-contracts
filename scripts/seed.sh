#!/bin/bash
# SPDX-License-Identifier: MIT
#
# Seed script for Pulsar payment processing contract.
# Populates a local or testnet environment with sample merchants, payments, and refunds.
#
# Usage: bash scripts/seed.sh [config_file]
# Example: bash scripts/seed.sh config/local.toml

set -e

# ── Configuration ─────────────────────────────────────────────────────────────

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
CONFIG_FILE="${1:-$REPO_ROOT/config/local.toml}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# ── Helper Functions ──────────────────────────────────────────────────────────

log_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

log_success() {
    echo -e "${GREEN}✓${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}⚠${NC} $1"
}

log_error() {
    echo -e "${RED}✗${NC} $1"
}

# Parse TOML config file (simple parser for basic key=value pairs)
parse_config() {
    local file=$1
    local section=$2
    local key=$3
    
    if [ ! -f "$file" ]; then
        log_error "Config file not found: $file"
        exit 1
    fi
    
    # Extract value from [section] key = value
    grep -A 100 "^\[$section\]" "$file" | grep "^$key" | head -1 | cut -d'=' -f2 | xargs
}

# ── Main Script ───────────────────────────────────────────────────────────────

main() {
    log_info "Pulsar Contract Seeding Script"
    log_info "Config file: $CONFIG_FILE"
    echo ""
    
    # Load configuration
    log_info "Loading configuration..."
    NETWORK=$(parse_config "$CONFIG_FILE" "network" "name")
    ADMIN_ACCOUNT=$(parse_config "$CONFIG_FILE" "admin" "account")
    TOKEN_ACCOUNT=$(parse_config "$CONFIG_FILE" "token" "account")
    TOKEN_CODE=$(parse_config "$CONFIG_FILE" "token" "code")
    PAYER_ACCOUNT=$(parse_config "$CONFIG_FILE" "payer" "account")
    MERCHANT_COUNT=$(parse_config "$CONFIG_FILE" "merchants" "count")
    PAYMENT_COUNT=$(parse_config "$CONFIG_FILE" "payments" "count")
    REFUND_COUNT=$(parse_config "$CONFIG_FILE" "refunds" "count")
    
    log_success "Configuration loaded"
    log_info "Network: $NETWORK"
    log_info "Admin: $ADMIN_ACCOUNT"
    log_info "Token: $TOKEN_CODE"
    log_info "Payer: $PAYER_ACCOUNT"
    echo ""
    
    # Verify Stellar CLI is available
    log_info "Checking Stellar CLI..."
    if ! command -v stellar &> /dev/null; then
        log_error "Stellar CLI not found. Please install it first."
        log_info "See: https://developers.stellar.org/docs/tools/stellar-cli"
        exit 1
    fi
    log_success "Stellar CLI found"
    echo ""
    
    # Verify accounts exist
    log_info "Verifying accounts..."
    for account in "$ADMIN_ACCOUNT" "$TOKEN_ACCOUNT" "$PAYER_ACCOUNT"; do
        if ! stellar keys show "$account" &> /dev/null; then
            log_error "Account not found: $account"
            log_info "Create it with: stellar keys generate $account"
            exit 1
        fi
        log_success "Account verified: $account"
    done
    echo ""
    
    # Get contract ID from environment or prompt
    log_info "Getting contract ID..."
    if [ -z "$CONTRACT_ID" ]; then
        read -p "Enter contract ID: " CONTRACT_ID
    fi
    
    if [ -z "$CONTRACT_ID" ]; then
        log_error "Contract ID is required"
        exit 1
    fi
    log_success "Contract ID: $CONTRACT_ID"
    echo ""
    
    # Initialize admin (if not already done)
    log_info "Initializing admin..."
    ADMIN_ADDR=$(stellar keys address "$ADMIN_ACCOUNT")
    
    # Try to set admin (will fail if already set, which is fine)
    if stellar contract invoke \
        --id "$CONTRACT_ID" \
        --source-account "$ADMIN_ACCOUNT" \
        --network "$NETWORK" \
        -- set_admin \
        --admin "$ADMIN_ADDR" 2>/dev/null; then
        log_success "Admin initialized"
    else
        log_warn "Admin already initialized (or error occurred)"
    fi
    echo ""
    
    # Register merchants
    log_info "Registering $MERCHANT_COUNT merchants..."
    MERCHANT_ADDRESSES=()
    CATEGORIES=("Retail" "Food" "Services" "Digital" "Other")
    
    for i in $(seq 1 "$MERCHANT_COUNT"); do
        MERCHANT_ACCOUNT="merchant_$i"
        CATEGORY_IDX=$(( (i - 1) % 5 ))
        CATEGORY="${CATEGORIES[$CATEGORY_IDX]}"
        
        # Create merchant account if it doesn't exist
        if ! stellar keys show "$MERCHANT_ACCOUNT" &> /dev/null; then
            stellar keys generate "$MERCHANT_ACCOUNT" > /dev/null 2>&1
            log_info "Created account: $MERCHANT_ACCOUNT"
        fi
        
        MERCHANT_ADDR=$(stellar keys address "$MERCHANT_ACCOUNT")
        MERCHANT_ADDRESSES+=("$MERCHANT_ADDR")
        
        # Register merchant
        stellar contract invoke \
            --id "$CONTRACT_ID" \
            --source-account "$MERCHANT_ACCOUNT" \
            --network "$NETWORK" \
            -- register_merchant \
            --merchant_address "$MERCHANT_ADDR" \
            --name "Test Merchant $i" \
            --description "Sample merchant for testing" \
            --contact_info "merchant$i@example.com" \
            --category "$CATEGORY" \
            --signing_public_key null > /dev/null 2>&1
        
        log_success "Registered merchant $i: $MERCHANT_ADDR ($CATEGORY)"
    done
    echo ""
    
    # Process payments
    log_info "Processing $PAYMENT_COUNT payments..."
    PAYER_ADDR=$(stellar keys address "$PAYER_ACCOUNT")
    
    for i in $(seq 1 "$PAYMENT_COUNT"); do
        MERCHANT_IDX=$(( (i - 1) % MERCHANT_COUNT ))
        MERCHANT_ADDR="${MERCHANT_ADDRESSES[$MERCHANT_IDX]}"
        AMOUNT=$(( 100000 + (i * 10000) ))
        ORDER_ID="ORDER_SEED_$(printf "%03d" $i)"
        
        # Process payment
        stellar contract invoke \
            --id "$CONTRACT_ID" \
            --source-account "$PAYER_ACCOUNT" \
            --network "$NETWORK" \
            -- process_payment_with_signature \
            --payer "$PAYER_ADDR" \
            --order "{\"order_id\":\"$ORDER_ID\",\"merchant_address\":\"$MERCHANT_ADDR\",\"payer\":\"$PAYER_ADDR\",\"token\":\"CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4\",\"amount\":$AMOUNT,\"description\":\"Test payment $i\",\"expires_at\":0}" \
            --signature "0000000000000000000000000000000000000000000000000000000000000000" \
            --merchant_public_key "0000000000000000000000000000000000000000000000000000000000000000" > /dev/null 2>&1
        
        log_success "Processed payment $i: $AMOUNT stroops to merchant $((MERCHANT_IDX + 1))"
    done
    echo ""
    
    # Initiate refunds
    log_info "Initiating $REFUND_COUNT refunds..."
    for i in $(seq 1 "$REFUND_COUNT"); do
        ORDER_IDX=$i
        ORDER_ID="ORDER_SEED_$(printf "%03d" $ORDER_IDX)"
        REFUND_ID="REFUND_SEED_$(printf "%03d" $i)"
        REFUND_AMOUNT=$(( (100000 + (ORDER_IDX * 10000)) / 2 ))
        
        # Initiate refund
        stellar contract invoke \
            --id "$CONTRACT_ID" \
            --source-account "$PAYER_ACCOUNT" \
            --network "$NETWORK" \
            -- initiate_refund \
            --caller "$PAYER_ADDR" \
            --refund_id "$REFUND_ID" \
            --order_id "$ORDER_ID" \
            --amount "$REFUND_AMOUNT" \
            --reason "Test refund" > /dev/null 2>&1
        
        log_success "Initiated refund $i: $REFUND_AMOUNT stroops for $ORDER_ID"
    done
    echo ""
    
    # Summary
    log_success "Seeding complete!"
    echo ""
    log_info "Summary:"
    log_info "  Merchants registered: $MERCHANT_COUNT"
    log_info "  Payments processed: $PAYMENT_COUNT"
    log_info "  Refunds initiated: $REFUND_COUNT"
    echo ""
    log_info "Next steps:"
    log_info "  1. Query merchant stats: stellar contract invoke --id $CONTRACT_ID --source-account $ADMIN_ACCOUNT --network $NETWORK -- get_merchant_stats --merchant <MERCHANT_ADDRESS> --date_start null --date_end null"
    log_info "  2. Query global stats: stellar contract invoke --id $CONTRACT_ID --source-account $ADMIN_ACCOUNT --network $NETWORK -- get_global_payment_stats --admins '[\"'$ADMIN_ADDR'\"]' --date_start null --date_end null"
    log_info "  3. Query payment history: stellar contract invoke --id $CONTRACT_ID --source-account $PAYER_ACCOUNT --network $NETWORK -- get_payer_payment_history --payer $PAYER_ADDR --cursor null --limit 10 --filter null --sort_field Date --sort_order Descending"
}

# Run main function
main "$@"
