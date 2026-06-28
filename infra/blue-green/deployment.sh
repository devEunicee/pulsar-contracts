#!/usr/bin/env bash
set -euo pipefail

ENVIRONMENT="${1:-green}"
IMAGE="${2:-pulsar:latest}"

if [[ "$ENVIRONMENT" != "blue" && "$ENVIRONMENT" != "green" ]]; then
  echo "Usage: $0 [blue|green] [image]" >&2
  exit 1
fi

echo "Deploying $IMAGE to $ENVIRONMENT"
echo "1. Run health checks"
echo "2. Switch traffic"
echo "3. Validate application"
echo "4. Keep rollback target available"
