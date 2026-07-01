# DevOps Operations

## Logging

- Application logs should be emitted as JSON with fields such as `timestamp`, `level`, `service`, `message`, and `request_id`.
- Centralized collection is configured in [logging/collector.conf](../logging/collector.conf) and [logging/parsers.conf](../logging/parsers.conf).
- Sensitive fields are stripped by the collector before export.

## Deployment

- Blue-green deployments are described in [infra/blue-green/README.md](../infra/blue-green/README.md).
- Use [infra/blue-green/deployment.sh](../infra/blue-green/deployment.sh) to coordinate a rollout.

## Infrastructure as Code

- Terraform scaffolding is available in [infra/terraform/main.tf](../infra/terraform/main.tf).
- Review `terraform plan` output before applying changes.

## Container image

- The repository uses a multi-stage Docker build in [Dockerfile](../Dockerfile).
- The runtime image runs as a non-root user and uses a slim base image.
