# Developer Onboarding Guide

This guide helps new contributors get productive quickly in the Pulsar contracts repository.

## Prerequisites

- Rust stable toolchain
- Stellar CLI
- Docker Desktop or a compatible container runtime
- Git and GitHub access

## Local setup

```bash
git clone https://github.com/devEunicee/pulsar-contracts.git
cd pulsar-contracts
rustup toolchain install stable
rustup target add wasm32-unknown-unknown
cargo test --manifest-path contracts/payment-processing-contract/Cargo.toml
```

## IDE recommendations

- VS Code with the Rust Analyzer extension
- Optional: Better TOML and GitHub Pull Requests extensions
- Enable format on save and rust-analyzer diagnostics

## Pre-commit hooks

Install the repository hooks before committing:

```bash
cargo install cargo-fmt
cargo install cargo-clippy
```

Then run the formatter and lints before each commit:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
```

## Running tests locally

```bash
cargo test --manifest-path contracts/payment-processing-contract/Cargo.toml
cargo test --manifest-path contracts/payment-processing-contract/Cargo.toml test_successful_payment_with_signature
```

## Debugging guide

- Reproduce the issue with a focused test first.
- Use `RUST_BACKTRACE=1` for panic traces.
- Inspect contract events emitted during tests.
- Read the relevant module files in the contract crate before changing behavior.

## Common development tasks

- Add or update contract logic in the contract crate.
- Add or update unit tests in the test modules.
- Update documentation for public API or operational procedures.
- Run formatting and tests before opening a PR.

## Project structure overview

- [README.md](../../README.md): high-level project overview and usage instructions.
- [contracts/payment-processing-contract/src](../../contracts/payment-processing-contract/src): contract implementation, storage, error handling, and tests.
- [docs/adr](../adr): architecture decision records.
- [docs/operations](.): operational runbooks and onboarding materials.

## Troubleshooting

- If Rust dependencies fail to resolve, update the toolchain and rerun `cargo test`.
- If Stellar CLI commands fail, verify the local network container is running.
- If tests fail unexpectedly, inspect the latest contract changes and the relevant snapshot files.
