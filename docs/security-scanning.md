# Security Scanning

This project uses automated security scanning on every push and pull request, plus a daily scheduled run.

## Tools

| Tool | Purpose |
|------|---------|
| Semgrep | SAST — static analysis for Rust, secrets, OWASP Top 10 |
| cargo-audit | Dependency CVE scanning via RustSec advisory database |
| cargo-deny | License compliance, banned crates, duplicate detection |
| Gitleaks | Secret/credential scanning across full git history |
| Trivy | Container image vulnerability scanning |

## Workflow

The workflow is defined in `.github/workflows/security-scanning.yml` and runs:
- On every push to `main` or `develop`
- On every pull request targeting `main` or `develop`
- Daily at 03:00 UTC via cron schedule

## Blocking Policy

- **SAST findings** at `ERROR` severity block the PR.
- **Critical/High CVEs** in dependencies (`cargo audit --deny warnings`) block the PR.
- **Secrets detected** by Gitleaks block the PR.
- **Critical/High container vulnerabilities** (Trivy) block the PR.
- **License violations** (cargo-deny) block the PR.

## Required Secrets

| Secret | Used by |
|--------|---------|
| `SEMGREP_APP_TOKEN` | Semgrep SAST (optional — runs in OSS mode without it) |
| `GITHUB_TOKEN` | Gitleaks (auto-provided by GitHub Actions) |

## Suppressing False Positives

- **cargo-audit**: Add an `[advisories]` ignore entry in `deny.toml` with a justification comment.
- **Semgrep**: Add a `# nosemgrep: <rule-id>` comment inline with justification.
- **Gitleaks**: Add an entry to `.gitleaks.toml` with a description.

## Reports

SARIF reports from Trivy are uploaded to GitHub Security → Code scanning alerts automatically.
