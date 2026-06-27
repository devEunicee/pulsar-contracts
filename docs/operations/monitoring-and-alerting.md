# Monitoring and Alerting

This guide defines a practical baseline for monitoring the Pulsar payment-processing stack, including contract health, application performance, infrastructure, and operational alerts.

## Goals

- Track contract and application health in real time.
- Detect regressions in payment throughput, error rates, and latency.
- Provide actionable alerts for on-call responders.
- Keep alerting lightweight and compatible with common observability tools.

## Metrics to collect

### Application metrics

- Request volume and success/error rate
- Payment processing latency (p50/p95/p99)
- Refund initiation and execution latency
- Contract invocation failures
- Authentication failures and rejected transactions

### Infrastructure metrics

- CPU usage per service/container
- Memory usage and pressure
- Disk usage and inode pressure
- Network I/O and dropped packets

### Database and storage metrics

- Connection pool saturation
- Query latency for contract state reads/writes
- Storage growth and compaction backlog

### Business metrics

- Payments per hour
- Refunds per hour
- Average payment value
- Failed payment rate
- Merchant registration rate

## Recommended alert thresholds

| Signal | Warning | Critical |
| --- | --- | --- |
| Error rate | > 2% for 10m | > 5% for 5m |
| P95 latency | > 800ms for 15m | > 2s for 5m |
| CPU usage | > 75% for 15m | > 90% for 5m |
| Memory usage | > 80% for 15m | > 95% for 5m |
| Disk usage | > 80% for 1h | > 95% for 15m |
| Payment throughput drop | < 80% of baseline for 15m | < 50% of baseline for 5m |

## Alert routing

- Primary channel: incident Slack channel
- Secondary channel: email and PagerDuty
- Escalation path:
  1. On-call engineer
  2. Backend lead
  3. Engineering manager

## On-call rotation

- Rotate weekly for the initial rollout.
- Keep a primary and secondary on-call contact.
- Document escalation steps in the incident runbook.

## Example Prometheus-style scrape config

```yaml
scrape_configs:
  - job_name: pulsar-api
    static_configs:
      - targets: ["api:9100"]
    metrics_path: /metrics
    scrape_interval: 15s
```

## Example alert rules

```yaml
groups:
  - name: pulsar-alerts
    rules:
      - alert: HighErrorRate
        expr: rate(http_requests_total{status=~"5.."}[5m]) / rate(http_requests_total[5m]) > 0.05
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: Elevated request error rate
```

## Operational checklist

- Verify dashboards are populated after deployment.
- Test each alert pathway at least once per quarter.
- Review thresholds after observing baseline behavior.
- Preserve runbooks and dashboard links in the incident channel.
