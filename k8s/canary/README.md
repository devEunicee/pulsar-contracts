# Canary Deployments — Pulsar Contracts

Implements a pod-count-based canary strategy on Kubernetes (no service mesh required).

## How It Works

Two Deployments share the same `app: pulsar-api` Service selector. Traffic is split proportionally to replica counts:

| Phase | Canary replicas | Stable replicas | Canary traffic |
|---|---|---|---|
| Initial | 1 | 19 | ~5% |
| Phase 2 | 5 | 15 | ~25% |
| Phase 3 | 10 | 10 | ~50% |
| Full rollout | 20 | 0 | 100% |

## Files

```
k8s/canary/
├── canary-deployment.yaml   # Stable + Canary Deployments
└── canary-monitor.yaml      # CronJob, RBAC for automated rollback

scripts/canary/
├── rollback.sh              # Scales canary to 0 if error rate > threshold
└── promote.sh               # Gradually increases canary traffic share
```

## Deploy Canary

```bash
# Apply canary manifests
kubectl apply -f k8s/canary/

# Verify both deployments
kubectl get deployments -n pulsar -l app=pulsar-api
```

## Promote

```bash
# Increase canary to 25%
./scripts/canary/promote.sh 25

# Increase to 50%
./scripts/canary/promote.sh 50

# Full rollout
./scripts/canary/promote.sh 100
```

## Rollback

```bash
# Manual rollback — scale canary to 0
kubectl scale deployment pulsar-api-canary --replicas=0 -n pulsar

# Or run the rollback script directly
NAMESPACE=pulsar ERROR_THRESHOLD=5 ./scripts/canary/rollback.sh
```

Automated rollback runs every 5 minutes via the `canary-monitor` CronJob querying Prometheus.

## Monitoring

- Error rate: `http_requests_total{track="canary",status=~"5.."}`
- Latency p99: `histogram_quantile(0.99, http_request_duration_seconds_bucket{track="canary"})`
- Traffic split: compare `http_requests_total{track="canary"}` vs `{track="stable"}`

Import the Grafana dashboard from `docs/monitoring/canary-dashboard.json` for a pre-built view.
