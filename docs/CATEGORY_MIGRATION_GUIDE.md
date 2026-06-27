# Merchant Category Migration Guide

This guide provides step-by-step instructions for adding new merchant categories to the payment processing contract.

## Overview

Merchant categories are currently implemented as a fixed enum. Adding new categories requires a contract upgrade. This guide walks through the process.

## Current Categories

- **Retail** — Retail stores and e-commerce
- **Food** — Restaurants, cafes, food delivery
- **Services** — Professional services, consulting, repairs
- **Digital** — Software, SaaS, digital products
- **Other** — Miscellaneous categories

## Adding a New Category

### Step 1: Update the Enum

Edit `contracts/payment-processing-contract/src/types.rs`:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MerchantCategory {
    Retail,
    Food,
    Services,
    Digital,
    Other,
    YourNewCategory,  // Add here
}
```

### Step 2: Add Test Coverage

Edit `contracts/payment-processing-contract/src/test.rs` to add tests for the new category:

```rust
#[test]
fn test_register_merchant_with_new_category() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    
    // Initialize contract
    let contract = PaymentProcessingContract::new(&env);
    contract.set_admin(vec![admin.clone()], 1).unwrap();
    
    // Register merchant with new category
    contract.register_merchant(
        merchant.clone(),
        String::from_str(&env, "Test Merchant"),
        String::from_str(&env, "Test Description"),
        String::from_str(&env, "contact@example.com"),
        MerchantCategory::YourNewCategory,
        None,
    ).unwrap();
    
    // Verify merchant was registered
    let retrieved = contract.get_merchant(merchant).unwrap();
    assert_eq!(retrieved.category, MerchantCategory::YourNewCategory);
}
```

### Step 3: Build and Test

```bash
cd contracts/payment-processing-contract

# Build the contract
cargo build --release --target wasm32-unknown-unknown

# Run tests
cargo test

# Update snapshots if needed
cargo test -- --nocapture
```

### Step 4: Verify Backward Compatibility

Ensure existing merchants with old categories still work:

```bash
# Run full test suite
cargo test --all

# Check for any compilation warnings
cargo clippy
```

### Step 5: Deploy New Contract

1. **Build WASM**:
   ```bash
   cargo build --release --target wasm32-unknown-unknown
   ```

2. **Upload to Network**:
   - Use Soroban CLI or your deployment tool
   - Get the new WASM hash

3. **Call Upgrade**:
   ```bash
   soroban contract invoke \
     --id <CONTRACT_ADDRESS> \
     --source <ADMIN_KEY> \
     -- upgrade \
     --admin <ADMIN_ADDRESS> \
     --new_wasm_hash <NEW_WASM_HASH>
   ```

4. **Verify Upgrade**:
   - Test merchant registration with new category
   - Verify existing merchants still work
   - Check contract version increased

### Step 6: Update Documentation

1. Update this guide with the new category
2. Update README.md with the new category list
3. Notify merchants of the new category availability
4. Update any external documentation or dashboards

## Rollback Procedure

If issues arise after deployment:

1. **Identify the Problem**: Check contract logs and test results
2. **Fix the Code**: Correct the issue in `types.rs`
3. **Rebuild and Test**: Follow steps 3-4 above
4. **Deploy Previous Version**: Call upgrade with the previous WASM hash
5. **Investigate**: Determine root cause before attempting again

## Best Practices

- **Test Thoroughly**: Add comprehensive tests for new categories
- **Backward Compatibility**: Ensure existing merchants are unaffected
- **Documentation**: Update all relevant documentation
- **Gradual Rollout**: Consider phased rollout to merchants
- **Monitoring**: Monitor contract events and merchant registrations
- **Communication**: Notify stakeholders before and after deployment

## Future: Dynamic Categories

In a future version, categories may become dynamic (string-based with admin-managed allowlist). This would eliminate the need for contract upgrades when adding categories. See ADR-0004 for details.

## Troubleshooting

### Compilation Errors

**Error**: `unknown variant in enum MerchantCategory`

**Solution**: Ensure the new variant is added to the enum definition in `types.rs` and all pattern matches are updated.

### Test Failures

**Error**: `test snapshots do not match`

**Solution**: Review the snapshot changes and update them if the changes are intentional:
```bash
cargo test -- --nocapture --test-threads=1
```

### Deployment Issues

**Error**: `contract upgrade failed`

**Solution**: 
- Verify the WASM hash is correct
- Ensure admin has authorization
- Check network connectivity
- Review contract logs for detailed errors

## Questions?

Refer to:
- ADR-0004: Merchant Category Management Strategy
- ADR-0002: Per-Entity Storage Layout
- README.md: Contract Upgrade Procedure
