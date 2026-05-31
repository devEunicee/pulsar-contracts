# Pull Request Details

## Branch
`misc-006-merchant-category-validation`

## Title
MISC-006: Merchant category validation and migration guide

## Description

### Overview
This PR addresses issue #101 MISC-006 by implementing comprehensive documentation and migration guidance for merchant category management.

### Changes

#### 1. ADR-0004: Merchant Category Management Strategy
- **File**: `docs/adr/0004-merchant-category-management.md`
- Documents the current enum-based category system
- Explains why categories require contract upgrades
- Outlines a future Phase 2 approach with string-based categories and admin-managed allowlist
- Provides migration path and backward compatibility notes

#### 2. Category Migration Guide
- **File**: `docs/CATEGORY_MIGRATION_GUIDE.md`
- Step-by-step instructions for adding new merchant categories
- Covers: enum updates, test coverage, build/test, deployment, and rollback procedures
- Includes troubleshooting section for common issues
- Best practices for category management

#### 3. README Updates
- Added new "Merchant Categories" section
- Documents all five current categories (Retail, Food, Services, Digital, Other)
- Links to migration guide and ADR-0004
- Updated register_merchant documentation with category notes

### Acceptance Criteria Met

✅ **Document that category additions require a contract upgrade**
- Clearly documented in ADR-0004 and migration guide
- Explains the type-safety benefits of enum-based approach

✅ **Add a migration guide template for category additions**
- Comprehensive CATEGORY_MIGRATION_GUIDE.md with step-by-step instructions
- Includes testing, deployment, and rollback procedures

✅ **Consider a string-based category with admin-managed allowlist**
- Documented as Phase 2 enhancement in ADR-0004
- Provides rationale and implementation approach for future versions

### Technical Details

#### Current Implementation
- MerchantCategory is a Soroban #[contracttype] enum with 5 variants
- Type-safe, prevents invalid categories at serialization
- Immutable per merchant (cannot be changed after registration)
- No functional impact on contract behavior (metadata-only)

#### Future Considerations
- Phase 2: Implement string-based categories with admin-managed allowlist
- Enable dynamic category additions without contract upgrades
- Add category-based queries, filtering, and statistics
- Implement category-based access control or restrictions

### Testing

All existing tests pass without modification. The changes are documentation-only and do not affect contract behavior or storage.

### Related Issues

Closes #101 MISC-006

### Labels

smart-contract, product, documentation

---

## How to Create the PR

1. Go to: https://github.com/MooreTheAnalyst/pulsar-contracts/pull/new/misc-006-merchant-category-validation
2. Copy the title and description above
3. Click "Create pull request"

Or use GitHub CLI:
```bash
gh pr create \
  --repo MooreTheAnalyst/pulsar-contracts \
  --base main \
  --head misc-006-merchant-category-validation \
  --title "MISC-006: Merchant category validation and migration guide" \
  --body "$(cat PR_DETAILS.md | tail -n +3)"
```
