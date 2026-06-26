# Pulsar Roadmap

This document communicates planned features, priorities, and release milestones for contributors and users.

---

## v0.1 — Current (shipped)

**Theme:** Core payment infrastructure on Soroban Stellar.

### Acceptance Criteria
- [x] Merchant registry (register, deactivate, query)
- [x] Signed payments via ed25519 signature verification
- [x] Refund lifecycle (initiate → approve/reject → execute, 30-day window)
- [x] Multi-signature payments (N-of-N signers)
- [x] Cursor-based paginated payment history (merchant + payer views)
- [x] Global admin stats
- [x] CI pipeline (fmt, clippy, test, WASM build, audit)
- [x] Structured error registry and TypeScript SDK mapping

---

## v0.2 — Bug Fixes & DX

**Theme:** Stability, developer experience, and off-chain tooling.

### Acceptance Criteria
- [ ] Webhook / notification service for payment events (BE-002)
- [ ] Payer payment history UI with Freighter wallet connection (FE-002)
- [ ] All known bugs from v0.1 resolved
- [ ] SDK published to npm with full TypeScript types
- [ ] Integration test suite covering all contract entry-points
- [ ] Deployment guide expanded with mainnet instructions

---

## v1.0 — Production-Ready

**Theme:** Security audit, performance, and ecosystem readiness.

### Acceptance Criteria
- [ ] Independent security audit completed with findings addressed
- [ ] Gas / resource optimisation pass on all contract functions
- [ ] Mainnet deployment with verified contract address
- [ ] Full API documentation site (auto-generated from source)
- [ ] Merchant dashboard UI (registration, payment analytics, refund management)
- [ ] Rate limiting and abuse-prevention mechanisms
- [ ] Upgrade path documented and tested end-to-end

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for how to pick up issues tied to these milestones.
