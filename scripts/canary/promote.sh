#!/usr/bin/env bash
# scripts/canary/promote.sh
# Gradually promote canary by increasing its replica share:
#   5% (1/20) → 25% (5/20) → 50% (10/20) → 100% (full rollout)
# Usage: ./promote.sh [--step 5|25|50|100]

set -euo pipefail

NAMESPACE="${NAMESPACE:-pulsar}"
CANARY="${CANARY:-pulsar-api-canary}"
STABLE="${STABLE:-pulsar-api-stable}"
TOTAL_REPLICAS=20
STEP="${1:-5}"

case "$STEP" in
  5)   CANARY_REPLICAS=1;  STABLE_REPLICAS=19 ;;
  25)  CANARY_REPLICAS=5;  STABLE_REPLICAS=15 ;;
  50)  CANARY_REPLICAS=10; STABLE_REPLICAS=10 ;;
  100) CANARY_REPLICAS=20; STABLE_REPLICAS=0  ;;
  *) echo "Usage: $0 [5|25|50|100]"; exit 1 ;;
esac

echo "[promote] Setting canary to ${STEP}% traffic (${CANARY_REPLICAS} replicas)..."
kubectl scale deployment "$CANARY" --replicas="$CANARY_REPLICAS" -n "$NAMESPACE"
kubectl scale deployment "$STABLE" --replicas="$STABLE_REPLICAS" -n "$NAMESPACE"
echo "[promote] Traffic split: ${STEP}% canary / $((100 - STEP))% stable."

if [[ "$STEP" == "100" ]]; then
  echo "[promote] Canary is now serving 100% of traffic. Consider tagging canary image as stable."
fi
