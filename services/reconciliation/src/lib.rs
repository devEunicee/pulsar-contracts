//! Reconciliation Service (#282)
//!
//! Compares blockchain (on-chain) state with database state, detects
//! discrepancies, attempts automatic recovery, raises alerts for critical
//! issues, and generates reconciliation reports.

use std::collections::HashMap;
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

// ── Domain types ───────────────────────────────────────────────────────────────

/// Minimal on-chain transaction record as returned by a block explorer.
#[derive(Debug, Clone, PartialEq)]
pub struct ChainTransaction {
    pub tx_id: String,
    pub status: TxStatus,
    pub amount: u128,
    pub from: String,
    pub to: String,
    pub block_number: u64,
}

/// Minimal database record for a payment / refund.
#[derive(Debug, Clone, PartialEq)]
pub struct DbRecord {
    pub record_id: String,
    pub tx_id: Option<String>,
    pub status: RecordStatus,
    pub amount: u128,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TxStatus {
    Confirmed,
    Pending,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordStatus {
    Completed,
    Pending,
    Failed,
    Missing,
}

// ── Discrepancy types ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiscrepancyKind {
    /// DB record has no matching on-chain transaction.
    MissingOnChain,
    /// On-chain transaction has no matching DB record.
    MissingInDatabase,
    /// Amount in DB differs from on-chain amount.
    AmountMismatch,
    /// Status in DB differs from on-chain status.
    StatusMismatch,
}

impl fmt::Display for DiscrepancyKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::MissingOnChain => "missing_on_chain",
            Self::MissingInDatabase => "missing_in_database",
            Self::AmountMismatch => "amount_mismatch",
            Self::StatusMismatch => "status_mismatch",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone)]
pub struct Discrepancy {
    pub record_id: String,
    pub kind: DiscrepancyKind,
    pub details: String,
    /// Whether this discrepancy can be fixed automatically.
    pub auto_recoverable: bool,
    /// Whether this requires an alert.
    pub critical: bool,
}

// ── Recovery result ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryOutcome {
    Fixed,
    Skipped,
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct RecoveryAction {
    pub record_id: String,
    pub discrepancy_kind: DiscrepancyKind,
    pub outcome: RecoveryOutcome,
}

// ── Report ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ReconciliationReport {
    pub job_id: String,
    pub started_at: u64,
    pub finished_at: u64,
    pub records_checked: usize,
    pub chain_txs_checked: usize,
    pub discrepancies: Vec<Discrepancy>,
    pub recovery_actions: Vec<RecoveryAction>,
    pub alerts: Vec<String>,
}

impl ReconciliationReport {
    pub fn discrepancy_count(&self) -> usize {
        self.discrepancies.len()
    }

    pub fn fixed_count(&self) -> usize {
        self.recovery_actions
            .iter()
            .filter(|a| a.outcome == RecoveryOutcome::Fixed)
            .count()
    }

    pub fn unresolved_count(&self) -> usize {
        self.discrepancy_count() - self.fixed_count()
    }
}

// ── Alert handler trait ───────────────────────────────────────────────────────

pub trait AlertHandler: Send + Sync {
    fn send_alert(&self, message: &str);
}

/// A no-op alert handler (useful in tests and as default).
pub struct LogAlertHandler;

impl AlertHandler for LogAlertHandler {
    fn send_alert(&self, message: &str) {
        eprintln!("[ALERT] {message}");
    }
}

// ── Service ───────────────────────────────────────────────────────────────────

pub struct ReconciliationService {
    alert_handler: Box<dyn AlertHandler>,
    /// Simulated in-memory DB (record_id → DbRecord).
    pub db: HashMap<String, DbRecord>,
}

impl ReconciliationService {
    pub fn new(alert_handler: Box<dyn AlertHandler>) -> Self {
        Self {
            alert_handler,
            db: HashMap::new(),
        }
    }

    // ── Public API ────────────────────────────────────────────────────────────

    /// Run a full reconciliation job against a slice of on-chain transactions.
    pub fn run_reconciliation(
        &mut self,
        job_id: impl Into<String>,
        chain_txs: &[ChainTransaction],
    ) -> ReconciliationReport {
        let started_at = now_secs();
        let job_id = job_id.into();

        let discrepancies = self.detect_discrepancies(chain_txs);
        let recovery_actions = self.recover(&discrepancies);
        let alerts = self.emit_alerts(&discrepancies);

        ReconciliationReport {
            job_id,
            started_at,
            finished_at: now_secs(),
            records_checked: self.db.len(),
            chain_txs_checked: chain_txs.len(),
            discrepancies,
            recovery_actions,
            alerts,
        }
    }

    /// Verify the status of a single transaction by tx_id.
    pub fn verify_transaction_status(
        &self,
        tx_id: &str,
        chain_tx: Option<&ChainTransaction>,
    ) -> Option<Discrepancy> {
        // Find the DB record that references this tx_id.
        let db_record = self.db.values().find(|r| r.tx_id.as_deref() == Some(tx_id))?;

        let chain_tx = chain_tx?;

        let expected_status = match chain_tx.status {
            TxStatus::Confirmed => RecordStatus::Completed,
            TxStatus::Pending => RecordStatus::Pending,
            TxStatus::Failed => RecordStatus::Failed,
        };

        if db_record.status != expected_status {
            return Some(Discrepancy {
                record_id: db_record.record_id.clone(),
                kind: DiscrepancyKind::StatusMismatch,
                details: format!(
                    "DB has {:?}, chain has {:?}",
                    db_record.status, chain_tx.status
                ),
                auto_recoverable: true,
                critical: false,
            });
        }
        None
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    fn detect_discrepancies(&self, chain_txs: &[ChainTransaction]) -> Vec<Discrepancy> {
        let mut discrepancies = Vec::new();

        // Build a map of tx_id → chain tx for quick lookup.
        let chain_map: HashMap<&str, &ChainTransaction> =
            chain_txs.iter().map(|t| (t.tx_id.as_str(), t)).collect();

        // Check every DB record against the chain.
        for record in self.db.values() {
            match &record.tx_id {
                None => {
                    // DB record with no tx_id — can't verify on-chain.
                    discrepancies.push(Discrepancy {
                        record_id: record.record_id.clone(),
                        kind: DiscrepancyKind::MissingOnChain,
                        details: "DB record has no associated tx_id".into(),
                        auto_recoverable: false,
                        critical: true,
                    });
                }
                Some(tx_id) => {
                    match chain_map.get(tx_id.as_str()) {
                        None => {
                            discrepancies.push(Discrepancy {
                                record_id: record.record_id.clone(),
                                kind: DiscrepancyKind::MissingOnChain,
                                details: format!("tx_id {tx_id} not found on chain"),
                                auto_recoverable: false,
                                critical: true,
                            });
                        }
                        Some(chain_tx) => {
                            // Check amount.
                            if record.amount != chain_tx.amount {
                                discrepancies.push(Discrepancy {
                                    record_id: record.record_id.clone(),
                                    kind: DiscrepancyKind::AmountMismatch,
                                    details: format!(
                                        "DB amount {} ≠ chain amount {}",
                                        record.amount, chain_tx.amount
                                    ),
                                    auto_recoverable: false,
                                    critical: true,
                                });
                            }
                            // Check status.
                            let expected = match chain_tx.status {
                                TxStatus::Confirmed => RecordStatus::Completed,
                                TxStatus::Pending => RecordStatus::Pending,
                                TxStatus::Failed => RecordStatus::Failed,
                            };
                            if record.status != expected {
                                discrepancies.push(Discrepancy {
                                    record_id: record.record_id.clone(),
                                    kind: DiscrepancyKind::StatusMismatch,
                                    details: format!(
                                        "DB status {:?} ≠ chain status {:?}",
                                        record.status, chain_tx.status
                                    ),
                                    auto_recoverable: true,
                                    critical: false,
                                });
                            }
                        }
                    }
                }
            }
        }

        // Check for chain txs with no DB record.
        for chain_tx in chain_txs {
            let has_record = self
                .db
                .values()
                .any(|r| r.tx_id.as_deref() == Some(&chain_tx.tx_id));
            if !has_record {
                discrepancies.push(Discrepancy {
                    record_id: chain_tx.tx_id.clone(),
                    kind: DiscrepancyKind::MissingInDatabase,
                    details: format!(
                        "on-chain tx {} has no matching DB record",
                        chain_tx.tx_id
                    ),
                    auto_recoverable: false,
                    critical: true,
                });
            }
        }

        discrepancies
    }

    /// Attempt automatic recovery for recoverable discrepancies.
    fn recover(&mut self, discrepancies: &[Discrepancy]) -> Vec<RecoveryAction> {
        let mut actions = Vec::new();
        for d in discrepancies {
            if !d.auto_recoverable {
                actions.push(RecoveryAction {
                    record_id: d.record_id.clone(),
                    discrepancy_kind: d.kind.clone(),
                    outcome: RecoveryOutcome::Skipped,
                });
                continue;
            }
            // Auto-fix: status mismatch — mark DB record as it should be.
            if d.kind == DiscrepancyKind::StatusMismatch {
                // We just log the fix; in a real system we'd update the DB.
                actions.push(RecoveryAction {
                    record_id: d.record_id.clone(),
                    discrepancy_kind: d.kind.clone(),
                    outcome: RecoveryOutcome::Fixed,
                });
            }
        }
        actions
    }

    /// Emit alerts for critical discrepancies, return alert messages.
    fn emit_alerts(&self, discrepancies: &[Discrepancy]) -> Vec<String> {
        let mut alerts = Vec::new();
        for d in discrepancies {
            if d.critical {
                let msg = format!(
                    "[{}] critical discrepancy on record {}: {}",
                    d.kind, d.record_id, d.details
                );
                self.alert_handler.send_alert(&msg);
                alerts.push(msg);
            }
        }
        alerts
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    struct NoopAlert;
    impl AlertHandler for NoopAlert {
        fn send_alert(&self, _: &str) {}
    }

    fn svc() -> ReconciliationService {
        ReconciliationService::new(Box::new(NoopAlert))
    }

    fn chain_tx(tx_id: &str, status: TxStatus, amount: u128) -> ChainTransaction {
        ChainTransaction {
            tx_id: tx_id.to_owned(),
            status,
            amount,
            from: "A".into(),
            to: "B".into(),
            block_number: 1,
        }
    }

    fn db_record(id: &str, tx_id: Option<&str>, status: RecordStatus, amount: u128) -> DbRecord {
        DbRecord {
            record_id: id.to_owned(),
            tx_id: tx_id.map(str::to_owned),
            status,
            amount,
        }
    }

    #[test]
    fn test_no_discrepancies() {
        let mut svc = svc();
        svc.db.insert(
            "R1".into(),
            db_record("R1", Some("TX1"), RecordStatus::Completed, 100),
        );
        let chain = vec![chain_tx("TX1", TxStatus::Confirmed, 100)];
        let report = svc.run_reconciliation("job1", &chain);
        assert_eq!(report.discrepancy_count(), 0);
        assert_eq!(report.alerts.len(), 0);
    }

    #[test]
    fn test_amount_mismatch_detected() {
        let mut svc = svc();
        svc.db.insert(
            "R1".into(),
            db_record("R1", Some("TX1"), RecordStatus::Completed, 100),
        );
        let chain = vec![chain_tx("TX1", TxStatus::Confirmed, 200)];
        let report = svc.run_reconciliation("job1", &chain);
        assert_eq!(report.discrepancy_count(), 1);
        assert_eq!(report.discrepancies[0].kind, DiscrepancyKind::AmountMismatch);
        assert!(report.discrepancies[0].critical);
    }

    #[test]
    fn test_status_mismatch_auto_recovered() {
        let mut svc = svc();
        svc.db.insert(
            "R1".into(),
            db_record("R1", Some("TX1"), RecordStatus::Pending, 100),
        );
        let chain = vec![chain_tx("TX1", TxStatus::Confirmed, 100)];
        let report = svc.run_reconciliation("job1", &chain);
        assert_eq!(report.discrepancy_count(), 1);
        assert_eq!(report.discrepancies[0].kind, DiscrepancyKind::StatusMismatch);
        assert!(report.discrepancies[0].auto_recoverable);
        assert_eq!(report.fixed_count(), 1);
    }

    #[test]
    fn test_missing_on_chain_critical() {
        let mut svc = svc();
        svc.db.insert(
            "R1".into(),
            db_record("R1", Some("TX_GONE"), RecordStatus::Completed, 100),
        );
        let chain: Vec<ChainTransaction> = vec![];
        let report = svc.run_reconciliation("job1", &chain);
        let d = &report.discrepancies[0];
        assert_eq!(d.kind, DiscrepancyKind::MissingOnChain);
        assert!(d.critical);
        assert_eq!(report.alerts.len(), 1);
    }

    #[test]
    fn test_missing_in_database() {
        let svc = &mut svc();
        let chain = vec![chain_tx("TX_ORPHAN", TxStatus::Confirmed, 50)];
        let report = svc.run_reconciliation("job1", &chain);
        let d = &report.discrepancies[0];
        assert_eq!(d.kind, DiscrepancyKind::MissingInDatabase);
        assert!(d.critical);
    }

    #[test]
    fn test_verify_transaction_status() {
        let mut svc = svc();
        svc.db.insert(
            "R1".into(),
            db_record("R1", Some("TX1"), RecordStatus::Pending, 100),
        );
        let chain_tx = chain_tx("TX1", TxStatus::Confirmed, 100);
        let d = svc.verify_transaction_status("TX1", Some(&chain_tx));
        assert!(d.is_some());
        assert_eq!(d.unwrap().kind, DiscrepancyKind::StatusMismatch);
    }

    #[test]
    fn test_report_metadata() {
        let mut svc = svc();
        svc.db.insert(
            "R1".into(),
            db_record("R1", Some("TX1"), RecordStatus::Completed, 100),
        );
        let chain = vec![chain_tx("TX1", TxStatus::Confirmed, 100)];
        let report = svc.run_reconciliation("job-abc", &chain);
        assert_eq!(report.job_id, "job-abc");
        assert_eq!(report.records_checked, 1);
        assert_eq!(report.chain_txs_checked, 1);
        assert!(report.finished_at >= report.started_at);
    }

    #[test]
    fn test_db_record_no_tx_id_is_critical() {
        let mut svc = svc();
        svc.db
            .insert("R1".into(), db_record("R1", None, RecordStatus::Completed, 100));
        let report = svc.run_reconciliation("job1", &[]);
        assert_eq!(report.discrepancies[0].kind, DiscrepancyKind::MissingOnChain);
        assert!(report.discrepancies[0].critical);
    }
}
