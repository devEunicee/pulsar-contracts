# Contributing to Pulsar

Thank you for your interest in contributing! Pulsar is an open-source project and we welcome contributions of all kinds.

## Getting Started

1. Fork the repository and clone your fork.
2. Install prerequisites (see README).
3. Create a feature branch: `git checkout -b feat/your-feature`.
4. Make your changes, add tests, and ensure everything passes.
5. Open a pull request against `main`.

## Development Setup

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target
rustup target add wasm32-unknown-unknown

# Run tests
cd contracts/payment-processing-contract
cargo test

# Build WASM
cargo build --target wasm32-unknown-unknown --release
```

## Code Standards

- **Formatting**: run `cargo fmt` before committing.
- **Linting**: run `cargo clippy -- -D warnings`; fix all warnings.
- **Tests**: every new function must have at least one unit test.
- **No unsafe**: do not use `unsafe` blocks.
- **No std**: the contract crate is `#![no_std]`; keep it that way.

## Pull Request Guidelines

- Keep PRs focused — one feature or fix per PR.
- Write a clear description of what changed and why.
- Reference any related issues with `Closes #<issue>`.
- All CI checks must pass before merging.

## Reporting Issues

Open a GitHub Issue with:
- A clear title and description.
- Steps to reproduce (for bugs).
- Expected vs actual behaviour.
- Rust / Stellar CLI version (`rustc --version`, `stellar --version`).

## Security Vulnerabilities

Do **not** open a public issue for security vulnerabilities. Email the maintainers directly or use GitHub's private security advisory feature.

## License

By contributing you agree that your contributions will be licensed under the [MIT License](LICENSE).
