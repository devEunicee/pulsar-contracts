# Migration Guide: vX.Y.Z → vA.B.C

> Copy this file to `docs/migrations/vX.Y.Z-to-vA.B.C.md` for each breaking release.

## Overview

Brief description of what changed and why.

## Breaking Changes

### Renamed / Removed Functions

| Old | New | Notes |
|-----|-----|-------|
| `old_function_name` | `new_function_name` | Reason for rename |

### Changed Parameters

| Function | Old Signature | New Signature | Notes |
|----------|--------------|---------------|-------|
| `function_name` | `(param: OldType)` | `(param: NewType)` | Reason |

### Changed Return Types

| Function | Old Return | New Return | Notes |
|----------|-----------|-----------|-------|
| `function_name` | `OldType` | `NewType` | Reason |

### New Required Fields

List any new fields added to existing structs that integrators must supply.

## Migration Steps

1. **Step one** — describe the first action integrators must take.
2. **Step two** — describe the next action.
3. **Rebuild and redeploy** — `cargo build --target wasm32-unknown-unknown --release` then redeploy.

## Compatibility Notes

- Minimum `soroban-sdk` version required: `X.Y.Z`
- Storage layout changes: yes / no (if yes, describe impact on existing on-chain data)

## Example

Before:

```rust
// old call
client.old_function(&arg);
```

After:

```rust
// new call
client.new_function(&arg, &new_param);
```
