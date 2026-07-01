# Terraform Infrastructure

This directory contains a minimal Terraform scaffold for the Pulsar deployment.

## Notes

- The state backend is configured for an S3 bucket. Replace the placeholder values with your environment-specific bucket and region before first apply.
- The configuration provisions a compute instance, a PostgreSQL database, and a security group.
- Review the planned changes before applying with `terraform plan`.

## Example

```bash
terraform init
terraform plan -var='db_password=change-me'
```
