# Centralized Logging

This directory contains a reference logging stack for the Pulsar contracts project.

## Components

- Structured JSON application logs emitted by the application runtime
- A Fluent Bit or vector-style collector configuration that forwards logs to a central backend
- Redaction rules to remove sensitive values before export
- An example alert rule for repeated error patterns

## Suggested deployment

1. Run the collector service on each host or Kubernetes node.
2. Ship logs to a managed backend such as OpenSearch, Elasticsearch, or cloud-native logging.
3. Use the example dashboard and query snippets to investigate failures.

## Redaction policy

The redaction rules remove values for the following keys by default:

- `authorization`
- `token`
- `password`
- `secret`
- `api_key`

## Example query

```json
{
  "query": {
    "bool": {
      "must": [{"match": {"level": "error"}}]
    }
  }
}
```
