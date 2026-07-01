# Kubernetes / Container Orchestration — Pulsar Contracts

## Structure

```
k8s/
├── namespace.yaml    # pulsar namespace
├── config.yaml       # ConfigMap + PersistentVolumeClaim
├── deployment.yaml   # Deployment (3 replicas, rolling update)
├── service.yaml      # ClusterIP Service + Ingress
└── hpa.yaml          # HorizontalPodAutoscaler (2–10 replicas)
```

## Acceptance Criteria Coverage

| Criterion | Implementation |
|---|---|
| Kubernetes cluster setup | All manifests target the `pulsar` namespace |
| Container scheduling & scaling | `deployment.yaml` with rolling update strategy |
| Service discovery | `Service` (ClusterIP) + `Ingress` in `service.yaml` |
| Load balancing | nginx Ingress controller |
| Persistent volume management | `PersistentVolumeClaim` in `config.yaml` |
| Health checks & auto-restart | `livenessProbe` + `readinessProbe` in `deployment.yaml` |
| Resource limits & requests | `resources.requests/limits` in `deployment.yaml` |
| Auto-scaling | `HorizontalPodAutoscaler` CPU 70% / Memory 80% in `hpa.yaml` |

## Deploy

```bash
kubectl apply -f k8s/namespace.yaml
kubectl apply -f k8s/config.yaml
kubectl apply -f k8s/deployment.yaml
kubectl apply -f k8s/service.yaml
kubectl apply -f k8s/hpa.yaml
```

Or apply everything at once:

```bash
kubectl apply -f k8s/
```

## Verify

```bash
kubectl get all -n pulsar
kubectl get hpa -n pulsar
kubectl describe deployment pulsar-api -n pulsar
```

## Dashboard & Monitoring

Install the Kubernetes Dashboard:

```bash
kubectl apply -f https://raw.githubusercontent.com/kubernetes/dashboard/v2.7.0/aio/deploy/recommended.yaml
kubectl proxy
```

Then open: http://localhost:8001/api/v1/namespaces/kubernetes-dashboard/services/https:kubernetes-dashboard:/proxy/

For production monitoring, connect to Prometheus + Grafana (see `docs/monitoring/`).
