# Security Audit — Pulsar Contracts

> **Status: PENDING** — An independent security audit must be completed before any mainnet deployment.

---

## Requirement (SEC-001)

The Pulsar payment-processing contract handles real token transfers on the Stellar network. Before any mainnet deployment, the contract must be reviewed by a recognised Soroban/Stellar smart contract security auditor.

---

## Audit Criteria

| Criterion | Requirement |
|---|---|
| Auditor | Recognised firm with Soroban/Stellar smart contract experience |
| Scope | All contract source files in `contracts/payment-processing-contract/src/` |
| Blocking findings | All **Critical** and **High** severity findings must be resolved before mainnet deployment |
| Report publication | Full audit report must be published in this repository under `docs/` |
| Re-audit | A re-audit or targeted review is required if Critical/High findings result in significant code changes |

---

## Scope of Audit

The following source files are in scope:

| File | Description |
|---|---|
| `src/lib.rs` | Contract entry-point and all public functions |
| `src/types.rs` | All data structures and storage keys |
| `src/storage.rs` | Storage read/write helpers and TTL management |
| `src/error.rs` | ContractError enum |
| `src/helper.rs` | Auth, validation, and filter helpers |

### Key areas of focus

- **Signature verification** — ed25519 signature scheme used in `process_payment_with_signature`
- **Authorization checks** — `require_auth` usage across all privileged functions
- **Refund logic** — 30-day window enforcement, cumulative refund cap, state machine transitions
- **Multi-signature payments** — signer deduplication, execution guards, state transitions
- **Storage and TTL** — persistent entry expiry, data integrity over time
- **Integer arithmetic** — overflow/underflow in amount calculations
- **Access control** — admin, merchant, and payer permission boundaries
- **Replay protection** — duplicate order ID and refund ID prevention

---

## Recommended Auditors

The following firms have demonstrated experience auditing Soroban/Stellar smart contracts or Rust-based blockchain code:

| Firm | Website | Notes |
|---|---|---|
| OtterSec | https://osec.io | Soroban and Rust smart contract audits |
| Trail of Bits | https://www.trailofbits.com | Rust and blockchain security specialists |
| Halborn | https://halborn.com | Stellar ecosystem experience |
| Cure53 | https://cure53.de | Rust and cryptographic code review |

> This list is informational. The project maintainers should conduct their own due diligence when selecting an auditor.

---

## Pre-Audit Checklist

Before engaging an auditor, ensure the following are complete:

- [ ] All known Critical and High issues from `ISSUES.md` are resolved
- [ ] Test coverage is at or above 80% (see T-015)
- [ ] `cargo audit` reports no unresolved advisories
- [ ] `cargo deny check` passes cleanly
- [ ] All public functions have inline documentation comments
- [ ] The contract ABI and storage layout are documented (see `docs/adr/`)

---

## Audit Process

1. **Engage auditor** — share contract source, documentation, and test suite
2. **Preliminary review** — auditor provides scope confirmation and timeline
3. **Audit execution** — typically 1–3 weeks depending on scope
4. **Draft report** — review findings, provide clarifications
5. **Remediation** — resolve all Critical and High findings; document mitigations for Medium/Low
6. **Final report** — auditor confirms fixes; final report issued
7. **Publication** — publish final report as `docs/audit-report-<version>-<auditor>.pdf` in this repository

---

## Deployment Gate

**Mainnet deployment is blocked until:**

1. A final audit report with no unresolved Critical or High findings exists in this repository
2. The report covers the exact commit hash being deployed
3. The project maintainer has signed off on all Medium findings

---

## Audit History

| Version | Auditor | Date | Report | Status |
|---|---|---|---|---|
| — | — | — | — | Pending |

*This table will be updated once an audit is completed.*
