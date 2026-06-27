# Secrets Management

This document outlines the baseline secret-handling approach for Pulsar deployments and local development.

## Objectives

- Centralize credentials, keys, and certificates.
- Reduce the blast radius of compromised credentials.
- Provide auditable access to secrets.
- Support safe rotation and incident response.

## Recommended tools

- HashiCorp Vault for centralized secret storage and dynamic credentials.
- AWS Secrets Manager as an alternative for AWS-hosted environments.
- Kubernetes Secrets or an injected sidecar where container orchestration is used.

## Secret categories

- API keys and access tokens
- Contract signing keys and admin keys
- Database credentials
- TLS certificates and private keys

## Access control

- Enforce least-privilege access by environment and role.
- Restrict secret read access to the services that need them.
- Require MFA for human access to production secrets.
- Keep an audit trail for all secret reads, writes, and rotations.

## Rotation policy

- Rotate long-lived credentials at least every 90 days.
- Rotate service account credentials immediately after suspected exposure.
- Maintain a documented rollback procedure for rotated secrets.

## Injection model

- Inject secrets at runtime through environment variables or mounted files.
- Avoid baking secrets into container images.
- Prefer short-lived credentials where supported.

## Development workflow

- Store local development secrets in a `.env.local` file that is ignored by Git.
- Use a local development vault or an encrypted secrets store for shared test credentials.
- Never commit private keys or certificates to the repository.

## Encryption at rest

- Enable encryption for the secrets backend and any backups.
- Protect secret snapshots with separate access controls.

## Emergency access

- Maintain a break-glass procedure for production incidents.
- Record who can access emergency credentials and why.
- Review emergency access quarterly.
