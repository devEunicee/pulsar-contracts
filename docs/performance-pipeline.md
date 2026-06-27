# Performance Optimization Pipeline

Automated performance testing runs on every push and pull request via `.github/workflows/performance.yml`.

## Jobs

| Job | Tool | Blocks PR |
|-----|------|-----------|
| Benchmark & Regression | cargo-criterion + github-action-benchmark | Yes (>120% regression) |
| WASM Bundle Size | wc / baseline diff | Yes (>100 KB or >5 KB increase) |
| API Response Time | custom Node script | Yes (median >200 ms) |
| Compile Time Profiling | `cargo build --timings` | No (artifact only) |

## Regression Detection

- Benchmark results are stored via `github-action-benchmark`. On `main` pushes the baseline is updated; on PRs the action compares against the stored baseline and comments if any benchmark regresses beyond 120%.
- WASM size is cached per branch. A size increase of more than 5 KB triggers a build failure.

## Thresholds

| Metric | Warning | Failure |
|--------|---------|---------|
| WASM size | > 80 KB | > 100 KB or +5 KB vs baseline |
| Benchmark | — | > 120% of baseline |
| API median response | — | > 200 ms (configurable via `PERF_THRESHOLD_MS`) |

## Artifacts

- `cargo-timings` HTML report uploaded for 14 days per run (useful for identifying slow-compiling crates).

## Local Benchmark Run

```bash
cd contracts/payment-processing-contract
cargo install cargo-criterion --locked
cargo criterion
```

Results appear in `target/criterion/`.
