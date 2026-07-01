# Load Testing — Pulsar Contracts

Load tests are written with [k6](https://k6.io) and cover four scenarios aligned with the acceptance criteria.

## Scenarios

| Scenario | Description |
|---|---|
| `ramp_up` | Gradually increase from 0 → 50 VUs over 2 min, hold 5 min, ramp down |
| `sustained` | 100 VUs held constant for 10 minutes |
| `spike` | 0 → 500 VUs in 30 s, hold 1 min, drop back to 0 |
| `endurance` | 50 VUs for 2 hours (run separately) |

## Thresholds

- p95 response time < 2 s
- p99 response time < 5 s
- Error rate < 5%

## Prerequisites

```bash
# Install k6
brew install k6          # macOS
sudo snap install k6     # Linux
```

## Running Tests

```bash
# All scenarios (ramp-up + sustained + spike)
BASE_URL=https://your-api.example.com k6 run load-tests/k6/load-test.js

# Endurance test only
BASE_URL=https://your-api.example.com k6 run --include-system-env \
  -e K6_SCENARIOS=endurance load-tests/k6/load-test.js

# Output results to JSON for analysis
k6 run --out json=results.json load-tests/k6/load-test.js
```

## CI Integration

The `ramp_up` and `sustained` scenarios run automatically in CI on every push to `main`. See `.github/workflows/load-test.yml`.

## Analysing Results

- Use [k6 Cloud](https://k6.io/cloud) or Grafana + InfluxDB for dashboards.
- Results JSON can be imported into the Grafana k6 dashboard (ID: 2587).
- Bottlenecks are surfaced via the `payment_duration` trend metric.
