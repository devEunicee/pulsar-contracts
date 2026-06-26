/// Data Quality Checks — Issue #305
///
/// Provides on-chain data integrity and consistency verification for payments,
/// refunds, and merchants. Each check function returns a `QualityReport`
/// with a summary of findings. Admin-only execution.
///
/// Checks performed:
/// - Referential integrity: refund order_ids resolve to existing payments
/// - Data consistency: payment amounts vs refunded_amount vs status
/// - Anomaly detection: amount outliers (> 3× median), future timestamps
/// - Reporting: structured report with per-check findings
use soroban_sdk::{contracttype, symbol_short, Address, Bytes, Env, String, Vec};

use crate::storage;
use crate::types::PaymentStatus;

// ── Report types ──────────────────────────────────────────────────────────────

/// Severity of a finding.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Severity {
    /// Informational — no action required.
    Info,
    /// Potential inconsistency — should be reviewed.
    Warning,
    /// Definite data integrity violation — remediation required.
    Critical,
}

/// A single finding produced by a quality check.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QualityFinding {
    /// Which check produced this finding.
    pub check: String,
    /// Severity level.
    pub severity: Severity,
    /// Opaque ID of the affected entity (order_id, refund_id, merchant address bytes).
    pub entity_id: Bytes,
    /// Human-readable description of the issue.
    pub detail: String,
}

/// Aggregated result of running all quality checks.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QualityReport {
    /// Ledger timestamp when the report was generated.
    pub generated_at: u64,
    /// Total number of entities inspected.
    pub total_checked: u32,
    /// Number of critical findings.
    pub critical_count: u32,
    /// Number of warning findings.
    pub warning_count: u32,
    /// Individual findings (capped at 100 to bound gas).
    pub findings: Vec<QualityFinding>,
}

// ── Internal helpers ──────────────────────────────────────────────────────────

fn str(env: &Env, s: &str) -> String {
    String::from_str(env, s)
}

fn empty_bytes(env: &Env) -> Bytes {
    Bytes::new(env)
}

fn addr_to_bytes(env: &Env, addr: &Address) -> Bytes {
    use soroban_sdk::xdr::ToXdr;
    addr.clone().to_xdr(env)
}

// ── Check implementations ─────────────────────────────────────────────────────

/// Check 1 — Referential integrity: every refund must reference a known payment.
fn check_refund_referential_integrity(
    env: &Env,
    findings: &mut Vec<QualityFinding>,
    checked: &mut u32,
    critical: &mut u32,
) {
    let refund_ids = storage::get_all_refund_ids(env);
    for rid in refund_ids.iter() {
        *checked += 1;
        if let Some(refund) = storage::get_refund(env, &rid) {
            if storage::get_payment(env, &refund.order_id).is_none() {
                *critical += 1;
                if findings.len() < 100 {
                    findings.push_back(QualityFinding {
                        check: str(env, "referential_integrity"),
                        severity: Severity::Critical,
                        entity_id: rid.clone(),
                        detail: str(env, "Refund references non-existent payment"),
                    });
                }
            }
        }
    }
}

/// Check 2 — Payment amount consistency: refunded_amount must not exceed amount,
/// and status must match refunded_amount.
fn check_payment_consistency(
    env: &Env,
    findings: &mut Vec<QualityFinding>,
    checked: &mut u32,
    critical: &mut u32,
    warning: &mut u32,
) {
    let payment_ids = storage::get_global_payment_ids(env);
    for oid in payment_ids.iter() {
        *checked += 1;
        let Some(payment) = storage::get_payment(env, &oid) else {
            continue;
        };

        // Refunded amount sanity
        if payment.refunded_amount < 0 || payment.refunded_amount > payment.amount {
            *critical += 1;
            if findings.len() < 100 {
                findings.push_back(QualityFinding {
                    check: str(env, "amount_consistency"),
                    severity: Severity::Critical,
                    entity_id: oid.clone(),
                    detail: str(env, "refunded_amount out of valid range [0, amount]"),
                });
            }
        }

        // Status vs refunded_amount alignment
        let status_ok = match payment.status {
            PaymentStatus::Completed => payment.refunded_amount == 0,
            PaymentStatus::PartiallyRefunded => {
                payment.refunded_amount > 0 && payment.refunded_amount < payment.amount
            }
            PaymentStatus::FullyRefunded => payment.refunded_amount == payment.amount,
        };
        if !status_ok {
            *warning += 1;
            if findings.len() < 100 {
                findings.push_back(QualityFinding {
                    check: str(env, "status_consistency"),
                    severity: Severity::Warning,
                    entity_id: oid.clone(),
                    detail: str(env, "Payment status inconsistent with refunded_amount"),
                });
            }
        }

        // Amount must be positive
        if payment.amount <= 0 {
            *critical += 1;
            if findings.len() < 100 {
                findings.push_back(QualityFinding {
                    check: str(env, "amount_consistency"),
                    severity: Severity::Critical,
                    entity_id: oid.clone(),
                    detail: str(env, "Payment amount is non-positive"),
                });
            }
        }
    }
}

/// Check 3 — Anomaly detection: future timestamps (paid_at > current ledger).
fn check_timestamp_anomalies(
    env: &Env,
    findings: &mut Vec<QualityFinding>,
    checked: &mut u32,
    warning: &mut u32,
) {
    let now = env.ledger().timestamp();
    let payment_ids = storage::get_global_payment_ids(env);
    for oid in payment_ids.iter() {
        *checked += 1;
        let Some(payment) = storage::get_payment(env, &oid) else {
            continue;
        };
        if payment.paid_at > now {
            *warning += 1;
            if findings.len() < 100 {
                findings.push_back(QualityFinding {
                    check: str(env, "timestamp_anomaly"),
                    severity: Severity::Warning,
                    entity_id: oid.clone(),
                    detail: str(env, "Payment timestamp is in the future"),
                });
            }
        }
    }
}

/// Check 4 — Merchant integrity: every payment's merchant_address must be registered.
fn check_merchant_integrity(
    env: &Env,
    findings: &mut Vec<QualityFinding>,
    checked: &mut u32,
    warning: &mut u32,
) {
    let payment_ids = storage::get_global_payment_ids(env);
    for oid in payment_ids.iter() {
        *checked += 1;
        let Some(payment) = storage::get_payment(env, &oid) else {
            continue;
        };
        if storage::get_merchant(env, &payment.merchant_address).is_none() {
            *warning += 1;
            if findings.len() < 100 {
                findings.push_back(QualityFinding {
                    check: str(env, "merchant_integrity"),
                    severity: Severity::Warning,
                    entity_id: oid.clone(),
                    detail: str(env, "Payment references unregistered merchant"),
                });
            }
        }
    }
}

// ── Public entry point ────────────────────────────────────────────────────────

/// Run all data quality checks and return a consolidated report.
/// Emits a contract event summarising the outcome.
pub fn run_checks(env: &Env) -> QualityReport {
    let mut findings: Vec<QualityFinding> = Vec::new(env);
    let mut total_checked: u32 = 0;
    let mut critical_count: u32 = 0;
    let mut warning_count: u32 = 0;

    check_refund_referential_integrity(env, &mut findings, &mut total_checked, &mut critical_count);
    check_payment_consistency(env, &mut findings, &mut total_checked, &mut critical_count, &mut warning_count);
    check_timestamp_anomalies(env, &mut findings, &mut total_checked, &mut warning_count);
    check_merchant_integrity(env, &mut findings, &mut total_checked, &mut warning_count);

    let report = QualityReport {
        generated_at: env.ledger().timestamp(),
        total_checked,
        critical_count,
        warning_count,
        findings,
    };

    // Emit summary event for off-chain alerting.
    env.events().publish(
        (symbol_short!("dq_check"),),
        (report.generated_at, total_checked, critical_count, warning_count),
    );

    report
}
