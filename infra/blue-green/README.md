# Blue-Green Deployment

This directory captures the rollout strategy for zero-downtime deployments.

## Workflow

1. Provision or update the green environment.
2. Run health checks against the green environment.
3. Switch the load balancer target group from blue to green.
4. Keep the blue environment available for a rapid rollback.

## Rollback

If the health checks fail after promotion, switch the traffic back to blue immediately and destroy or isolate the green environment.

## Operational checklist

- Health endpoint returns 200 on the green environment.
- Database migrations are backward compatible.
- DNS changes are applied only after the new environment is healthy.
