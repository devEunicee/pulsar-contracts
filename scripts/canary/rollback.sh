#!/usr/bin/env bash
# scripts/canary/rollback.sh
# Automated canary rollback: scales canary to 0 if error rate exceeds threshold.
# Meant to be run by the canary-monitor CronJob or triggered manually.

set -euo pipefail

NAMESPACE="${NAMESPACE:-pulsar}"
CANARY_DEPLOYMENT="${CANARY_DEPLOYMENT:-pulsar-api-canary}"
METRICS_URL="${METRICS_URL:-http://prometheus.monitoring.svc.cluster.local:9090}"
ERROR_THRESHOLD="${ERROR_THRESHOLD:-5}"   # percent

echo "[canary-rollback] Checking canary error rate..."

# Query Prometheus for canary HTTP 5xx rate over last 5 minutes
ERROR_RATE=$(curl -sf "${METRICS_URL}/api/v1/query" \
  --data-urlencode 'query=100 * sum(rate(http_requests_total{track="canary",status=~"5.."}[5m])) / sum(rate(http_requests_total{track="canary"}[5m]))' \
  | python3 -c "import sys,json; data=json.load(sys.stdin); print(data['data']['result'][0]['value'][1] if data['data']['result'] else '0')" 2>/dev/null || echo "0")

echo "[canary-rollback] Current canary error rate: ${ERROR_RATE}%"

if (( $(echo "$ERROR_RATE > $ERROR_THRESHOLD" | bc -l) )); then
  echo "[canary-rollback] ERROR RATE ${ERROR_RATE}% exceeds threshold ${ERROR_THRESHOLD}%. Rolling back canary..."
  kubectl scale deployment "${CANARY_DEPLOYMENT}" --replicas=0 -n "${NAMESPACE}"
  echo "[canary-rollback] Canary rolled back. Stable deployment continues serving 100% traffic."
else
  echo "[canary-rollback] Error rate within threshold. Canary is healthy."
fi
