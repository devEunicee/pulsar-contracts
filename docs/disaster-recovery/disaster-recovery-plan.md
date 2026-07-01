# Disaster Recovery Plan — Pulsar Contracts

## 1. Overview

This document defines the Disaster Recovery (DR) plan for the Pulsar Contracts infrastructure. It covers RTO/RPO targets, backup and restore procedures, cross-region failover, communication protocols, and drill schedules.

---

## 2. Recovery Objectives

| Metric | Target |
|---|---|
| **RTO** (Recovery Time Objective) | < 1 hour |
| **RPO** (Recovery Point Objective) | < 15 minutes |

---

## 3. Backup Procedures

### 3.1 Contract State Backups
- Stellar network ledger state is replicated automatically across validators.
- Off-chain databases (if any) are snapshotted every 15 minutes via automated cron jobs.
- Snapshots are stored in a secondary region S3 bucket with versioning enabled.

### 3.2 Configuration Backups
- All infrastructure configs (Kubernetes manifests, Helm charts, Terraform state) are stored in Git and replicated to a secondary region bucket.
- Secrets are managed via AWS Secrets Manager with cross-region replication.

### 3.3 Backup Retention
| Type | Retention |
|---|---|
| 15-min snapshots | 48 hours |
| Daily snapshots | 30 days |
| Weekly snapshots | 12 weeks |

---

## 4. Restore Procedures

### 4.1 Database Restore
```bash
# Restore from latest snapshot
./scripts/dr/restore-db.sh --snapshot latest --env production

# Restore from specific point-in-time
./scripts/dr/restore-db.sh --timestamp "2026-06-28T05:00:00Z" --env production
```

### 4.2 Contract Redeployment
```bash
# Redeploy contract to testnet/mainnet from last known good WASM
./scripts/dr/redeploy-contract.sh --network mainnet --wasm-path ./artifacts/payment_processing_contract.wasm
```

### 4.3 Infrastructure Restore
```bash
# Apply Terraform from remote state
cd infra/terraform
terraform init -backend-config=backend-prod.hcl
terraform apply -var-file=prod.tfvars
```

---

## 5. Cross-Region Failover

### 5.1 Architecture
- **Primary Region:** us-east-1
- **Secondary Region:** eu-west-1
- DNS failover is managed via Route 53 health checks with a 30-second evaluation period.

### 5.2 Failover Steps
1. Confirm primary region failure via monitoring alerts.
2. Promote secondary region RDS read replica to primary (automated via Lambda if health check fails for 2 minutes).
3. Update Route 53 DNS to point to secondary region load balancer.
4. Verify service health in secondary region.
5. Notify stakeholders per the communication plan.

### 5.3 Failback Steps
1. Restore primary region infrastructure.
2. Re-sync data from secondary to primary.
3. Switch DNS back to primary.
4. Monitor for 30 minutes post-failback.

---

## 6. Recovery Runbooks

### Runbook 1: Node Outage
1. Alert fires from monitoring dashboard.
2. On-call engineer acknowledges within 5 minutes.
3. Check pod/node status: `kubectl get nodes && kubectl get pods -A`.
4. If node failure: `kubectl cordon <node>` then allow Kubernetes to reschedule pods.
5. Replace failed node via autoscaling group.
6. Verify all pods are Running.

### Runbook 2: Contract Deployment Failure
1. Deployment pipeline fails — CI/CD notifies Slack `#deployments`.
2. Engineer reviews deployment logs.
3. Roll back to previous contract: `./scripts/dr/rollback-contract.sh --version <prev>`.
4. Confirm rollback via contract version query.
5. Root cause analysis within 24 hours.

### Runbook 3: Data Corruption
1. Stop writes to affected service immediately.
2. Identify corruption scope from audit logs.
3. Restore from last clean snapshot prior to corruption timestamp.
4. Replay transaction logs from snapshot to corruption point if possible.
5. Resume writes after validation.

---

## 7. Communication Plan

### 7.1 Severity Levels

| Level | Definition | Response Time |
|---|---|---|
| P1 | Full service outage | 15 minutes |
| P2 | Partial outage / degraded performance | 30 minutes |
| P3 | Minor issue, workaround available | 2 hours |

### 7.2 Notification Channels
- **Slack:** `#incidents` for real-time updates
- **PagerDuty:** Auto-page on-call for P1/P2
- **Status Page:** Public updates every 30 minutes during active incident
- **Email:** Stakeholder summary after incident resolved

### 7.3 Escalation Path
1. On-call engineer
2. Engineering Lead
3. CTO (P1 only, if not resolved in 30 minutes)

### 7.4 Incident Template
```
[INCIDENT] <Short description>
Severity: P1/P2/P3
Start Time: <UTC timestamp>
Impact: <services/users affected>
Current Status: Investigating / Identified / Monitoring / Resolved
Next Update: <UTC timestamp>
```

---

## 8. Disaster Recovery Drill Schedule

| Drill | Frequency | Responsible Team |
|---|---|---|
| Backup restore test | Monthly | Infrastructure |
| Failover simulation | Quarterly | Infrastructure + Engineering |
| Full DR drill | Bi-annually | All Engineering |
| Runbook review | Quarterly | On-call rotation |

### 8.1 Drill Procedure
1. Announce drill window 48 hours in advance.
2. Execute failover or restore in staging environment.
3. Measure actual RTO/RPO achieved.
4. Document results in `docs/disaster-recovery/drill-results/`.
5. Update runbooks based on findings.

---

## 9. Testing and Updates

- This document is reviewed and updated **quarterly** or after any significant infrastructure change.
- All runbooks are tested during drills and updated if steps are found to be inaccurate.
- DR automation scripts in `scripts/dr/` are covered by CI smoke tests.

---

## 10. Related Documents
- [Contributing](../../CONTRIBUTING.md)
- [Infrastructure Runbooks](../runbooks/)
- [Monitoring Setup](../monitoring/)
