# Fuzz & Property-Based Tests

> **Issue T-011** — Property-based tests for `validate_amount`,
> `matches_filter`, and `paginate_payments`.

---

## Overview

The contract uses [proptest](https://proptest-rs.github.io/proptest/) for
property-based testing. Proptest generates hundreds of random inputs for each
property and shrinks any failing case to the smallest reproducer.

The test module lives at:

```
contracts/payment-processing-contract/src/prop_tests.rs
```

---

## Properties Covered

### `validate_amount`

| Property | Description |
|---|---|
| Positive amounts accepted | Any `amount > 0` returns `Ok(())` |
| Zero rejected | `amount == 0` returns `Err(InvalidAmount)` |
| Negative amounts rejected | Any `amount < 0` returns `Err(InvalidAmount)` |
| Boundary: `i128::MAX` valid | Maximum representable value is accepted |
| Boundary: `i128::MIN` invalid | Minimum representable value is rejected |

### `matches_filter`

| Property | Description |
|---|---|
| Empty filter always passes | A filter with no constraints and `StatusFilter::Any` never rejects a record |
| `amount_min` boundary | Record at exactly `amount_min` passes; record at `amount_min + 1` fails |
| `amount_max` boundary | Record at exactly `amount_max` passes; record at `amount_max - 1` fails |
| Inverted range never passes | When `amount_min > amount_max` no record can satisfy both bounds |
| `date_start` boundary | Record at exactly `date_start` passes; record one second earlier fails |
| Status mismatch | A `Completed` record fails a `PartiallyRefunded` filter |
| Status mismatch (refunded) | A `FullyRefunded` record fails a `Completed` filter |

### `paginate_payments`

| Property | Description |
|---|---|
| Page size bounded | Returned page never exceeds `min(limit, 100)` records |
| Cursor iff more records | `next_cursor` is `Some` iff there are records beyond the current page |
| Full traversal | Following all cursors yields every record exactly once |
| Empty input | Empty record set returns empty page, no cursor, total = 0 |
| Zero limit | A limit of 0 produces an empty page |

---

## Running Locally

```bash
# Run all property tests with the default 256 cases
cargo test prop_ --all-features --locked
```

Run from the contract directory:

```bash
cd contracts/payment-processing-contract
cargo test prop_ --all-features --locked
```

### Increase the number of cases

```bash
PROPTEST_CASES=1000 cargo test prop_ --all-features --locked
```

### Replay a failing seed

When proptest finds a failure it prints a line like:

```
PROPTEST_SEED=abc123...
```

Replay it with:

```bash
PROPTEST_SEED=abc123... cargo test prop_ --all-features --locked
```

Proptest also saves a regression file at:

```
contracts/payment-processing-contract/proptest-regressions/prop_tests.txt
```

This file is committed to the repository so that previously found failures are
always re-tested on every run.

---

## CI

Property-based tests run as a separate `property-tests` job in
`.github/workflows/ci.yml`. The job runs with `PROPTEST_CASES=512` and is
independent of the main `test` matrix so it does not block the WASM build.

---

## Adding New Properties

1. Add a new `proptest!` block in `src/prop_tests.rs`.
2. Use `prop_assert!` / `prop_assert_eq!` for assertions.
3. Document the new property in this file.
4. Run locally to verify before pushing.
