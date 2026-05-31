# ADR-0004: Merchant Category Management Strategy

**Status:** Accepted  
**Date:** 2024-01-15  
**Deciders:** Pulsar Contributors

## Context

MerchantCategory is currently implemented as a fixed enum with five variants (Retail, Food, Services, Digital, Other). This design prevents admins from adding new categories without a contract upgrade, limiting flexibility for evolving business requirements.

## Problem Statement

- New merchant categories require contract code changes and redeployment
- No way to add categories dynamically without contract upgrade
- All tests use the same category (Retail), indicating limited multi-category support
- Category field has no functional impact on contract behavior (metadata-only)

## Decision

Implement a **hybrid approach** with two phases:

### Phase 1: Documentation & Migration Guide (Current)
- Document the current enum-based limitation
- Provide a migration guide template for adding new categories
- Establish best practices for category management

### Phase 2: Future Enhancement (Optional)
- Migrate to string-based categories with admin-managed allowlist
- Implement category validation and querying capabilities
- Add category-based business logic (filtering, statistics, access control)

## Implementation Details

### Current Enum-Based Approach (Phase 1)

**Variants:**
- `Retail` — Retail stores and e-commerce
- `Food` — Restaurants, cafes, food delivery
- `Services` — Professional services, consulting, repairs
- `Digital` — Software, SaaS, digital products
- `Other` — Miscellaneous categories

**Limitations:**
- Adding categories requires contract upgrade
- Existing merchants retain their category through upgrades
- No category-based filtering or queries
- No category-based access control

### Future String-Based Approach (Phase 2)

**Proposed Structure:**
```rust
#[contracttype]
pub struct AllowedCategory {
    pub name: String,
    pub description: String,
    pub created_at: u64,
}

// Storage: AllowedCategories(Vec<String>)
// Merchant: category: String (validated against allowlist)
```

**Benefits:**
- Admin can add categories without contract upgrade
- Backward compatible with existing merchants
- Enables category-based queries and filtering
- Supports category-based business logic

## Migration Path

### Adding a New Category (Current Enum Approach)

1. **Update the enum** in `src/types.rs`:
   ```rust
   pub enum MerchantCategory {
       Retail,
       Food,
       Services,
       Digital,
       Other,
       NewCategory,  // Add new variant
   }
   ```

2. **Update tests** to cover the new category:
   - Add test cases in `src/test.rs`
   - Update test snapshots if needed

3. **Build and test**:
   ```bash
   cargo build --release
   cargo test
   ```

4. **Deploy new contract**:
   - Build WASM: `cargo build --release --target wasm32-unknown-unknown`
   - Upload to network
   - Call `upgrade()` with new WASM hash

5. **Update documentation**:
   - Update this ADR with new category details
   - Update README with category list
   - Notify merchants of new category availability

### Backward Compatibility

- Existing merchants retain their original category through upgrades
- New merchants can use new categories immediately after upgrade
- No data migration required for existing merchants
- Category field is immutable per merchant (cannot be changed after registration)

## Consequences

### Positive
- Clear documentation of category management strategy
- Established migration path for future category additions
- Type-safe enum prevents invalid categories
- Backward compatible with existing merchants

### Negative
- Adding categories requires contract upgrade (not dynamic)
- No category-based business logic currently implemented
- All merchants must be aware of upgrade to use new categories
- No category querying or filtering capabilities

### Neutral
- Category is metadata-only (no functional impact on payments)
- All tests currently use Retail category (limited coverage)

## Future Considerations

1. **Phase 2 Migration**: Implement string-based categories with allowlist
2. **Category Queries**: Add functions to query merchants by category
3. **Category Statistics**: Track payment volume by category
4. **Category-Based Access Control**: Implement category-specific rules or restrictions
5. **Category Deprecation**: Plan for removing or renaming categories

## References

- ADR-0002: Per-Entity Storage Layout
- ADR-0001: Signature Scheme
- README: Contract Upgrade Procedure
