# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | ✅        |

## Reporting a Vulnerability

**Do not open a public GitHub issue for security vulnerabilities.**

Please report security vulnerabilities through the GitHub private security advisory system:

👉 **[Report a vulnerability](https://github.com/devEunicee/pulsar-contracts/security/advisories/new)**

### What to include

- A clear description of the vulnerability and its potential impact
- Steps to reproduce or a proof-of-concept
- Affected versions
- Any suggested mitigations (optional)

### Response SLA

| Milestone | Target |
|-----------|--------|
| Acknowledgement | Within **48 hours** of submission |
| Initial assessment | Within **5 business days** |
| Fix or mitigation | Within **90 days** (critical issues prioritised) |

We will keep you informed of progress throughout the process and credit you in the release notes unless you prefer to remain anonymous.

## Scope

This policy covers the smart contract code in this repository. Issues in third-party dependencies (e.g. `soroban-sdk`) should be reported to their respective maintainers.

---

## Mainnet Security Audit (SEC-001)

The Pulsar contract handles real token transfers. **An independent security audit by a recognised Soroban/Stellar smart contract auditor is required before any mainnet deployment.**

All Critical and High severity findings must be resolved before deployment. The full audit report will be published in [`docs/security-audit.md`](docs/security-audit.md) once completed.

See [`docs/security-audit.md`](docs/security-audit.md) for the full audit scope, recommended auditors, pre-audit checklist, and deployment gate criteria.
