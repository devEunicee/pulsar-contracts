# Contributing to Pulsar

Thank you for your interest in contributing! Pulsar is an open-source project and we welcome contributions of all kinds.

By participating in this project, you agree to abide by our [Code of Conduct](CODE_OF_CONDUCT.md).

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

## Branch Protection Rules

The `main` branch is protected with the following enforced settings:

- **Required reviews**: at least 1 approving review before merging.
- **Required status checks**: all CI jobs (`test`, `build`, `security-audit`) must pass.
- **No direct pushes**: commits must be submitted via a pull request; force-pushes are disabled.

These rules are enforced via GitHub branch protection settings. Contributors cannot bypass them.

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

## Issue Triage

**Label taxonomy**

- `bug` – a defect in the code or documentation.
- `enhancement` – a new feature or improvement.
- `security` – security‑related issue.
- `documentation` – docs improvements or corrections.
- `question` – user questions or usage help.

**SLA for first response**

- All new issues receive an initial acknowledgement within **24 hours** on weekdays.
- Critical security issues are responded to within **4 hours**.
- Non‑critical issues aim for a response within **48 hours**.

## Security Vulnerabilities

Do **not** open a public issue for security vulnerabilities. Email the maintainers directly or use GitHub's private security advisory feature.

## License

By contributing you agree that your contributions will be licensed under the [MIT License](LICENSE).
