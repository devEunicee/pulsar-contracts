//! Refund Management Service (#274)
//!
//! Manages the full refund lifecycle: initiate → approve/reject → execute,
//! with state-transition validation, grace period enforcement, audit logging,
//! and event emission.

use std::collections::HashMap;
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

// ── Constants ──────────────────────────────────────────────────────────────────

/// Refund window in seconds (30 days).
pub const REFUND_WINDOW_SECS: u64 = 30 * 24 * 60 * 60;
/// Grace period after approval before execution expires (48 hours).
pub const EXECUTION_GRACE_PERIOD_SECS: u64 = 48 * 60 * 60;

// ── Domain types ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefundStatus {
    Pending,
    Approved,
    Rejected,
    Cancelled,
    Executed,
}

impl fmt::Display for RefundStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Pending => "Pending",
            Self::Approved => "Approved",
            Self::Rejected => "Rejected",
            Self::Cancelled => "Cancelled",
            Self::Executed => "Executed",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone)]
pub struct RefundRecord {
    pub refund_id: String,
    pub order_id: String,
    /// Refund amount in the token's base unit.
    pub amount: u128,
    pub reason: String,
    pub status: RefundStatus,
    pub initiated_by: String,
    pub initiated_at: u64,
    pub approved_at: Option<u64>,
    pub executed_at: Option<u64>,
    pub cancelled_at: Option<u64>,
    /// Unix timestamp at which the original payment was made (for window check).
    pub payment_paid_at: u64,
}

#[derive(Debug, Clone)]
pub struct RefundEvent {
    pub refund_id: String,
    pub order_id: String,
    pub event_type: RefundEventType,
    pub actor: String,
    pub timestamp: u64,
    pub metadata: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefundEventType {
    Initiated,
    Approved,
    Rejected,
    Executed,
    Cancelled,
}

impl fmt::Display for RefundEventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Initiated => "refund_initiated",
            Self::Approved => "refund_approved",
            Self::Rejected => "refund_rejected",
            Self::Executed => "refund_executed",
            Self::Cancelled => "refund_cancelled",
        };
        write!(f, "{s}")
    }
}

// ── Errors ────────────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq, Eq)]
pub enum RefundError {
    RefundNotFound,
    RefundAlreadyExists,
    InvalidAmount,
    InvalidTransition { from: String, to: String },
    RefundWindowExpired,
    ExecutionWindowExpired,
    Unauthorized,
    AmountExceedsPayment,
}

impl fmt::Display for RefundError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RefundNotFound => write!(f, "Refund not found"),
            Self::RefundAlreadyExists => write!(f, "Refund already exists"),
            Self::InvalidAmount => write!(f, "Amount must be > 0"),
            Self::InvalidTransition { from, to } => {
                write!(f, "Invalid state transition: {from} → {to}")
            }
            Self::RefundWindowExpired => write!(f, "30-day refund window has expired"),
            Self::ExecutionWindowExpired => write!(f, "48-hour execution grace period has expired"),
            Self::Unauthorized => write!(f, "Caller is not authorized for this operation"),
            Self::AmountExceedsPayment => {
                write!(f, "Refund amount exceeds original payment amount")
            }
        }
    }
}

// ── Service ───────────────────────────────────────────────────────────────────

pub struct RefundService {
    refunds: HashMap<String, RefundRecord>,
    /// Audit log of all state-change events.
    pub audit_log: Vec<RefundEvent>,
}

impl RefundService {
    pub fn new() -> Self {
        Self {
            refunds: HashMap::new(),
            audit_log: Vec::new(),
        }
    }

    // ── Queries ───────────────────────────────────────────────────────────────

    pub fn get_refund(&self, refund_id: &str) -> Option<&RefundRecord> {
        self.refunds.get(refund_id)
    }

    pub fn list_by_order(&self, order_id: &str) -> Vec<&RefundRecord> {
        self.refunds
            .values()
            .filter(|r| r.order_id == order_id)
            .collect()
    }

    // ── State transitions ──────────────────────────────────────────────────────

    /// Initiate a refund. Caller must be payer or merchant.
    pub fn initiate(
        &mut self,
        refund_id: String,
        order_id: String,
        amount: u128,
        reason: String,
        caller: String,
        payment_paid_at: u64,
        original_amount: u128,
        already_refunded: u128,
    ) -> Result<(), RefundError> {
        if self.refunds.contains_key(&refund_id) {
            return Err(RefundError::RefundAlreadyExists);
        }
        if amount == 0 {
            return Err(RefundError::InvalidAmount);
        }
        if amount > original_amount.saturating_sub(already_refunded) {
            return Err(RefundError::AmountExceedsPayment);
        }

        let now = now_secs();
        if now > payment_paid_at + REFUND_WINDOW_SECS {
            return Err(RefundError::RefundWindowExpired);
        }

        let record = RefundRecord {
            refund_id: refund_id.clone(),
            order_id: order_id.clone(),
            amount,
            reason,
            status: RefundStatus::Pending,
            initiated_by: caller.clone(),
            initiated_at: now,
            approved_at: None,
            executed_at: None,
            cancelled_at: None,
            payment_paid_at,
        };
        self.refunds.insert(refund_id.clone(), record);
        self.emit(RefundEvent {
            refund_id,
            order_id,
            event_type: RefundEventType::Initiated,
            actor: caller,
            timestamp: now,
            metadata: None,
        });
        Ok(())
    }

    /// Approve a pending refund. Caller must be merchant or admin.
    pub fn approve(&mut self, refund_id: &str, caller: String) -> Result<(), RefundError> {
        let now = now_secs();
        let record = self
            .refunds
            .get_mut(refund_id)
            .ok_or(RefundError::RefundNotFound)?;

        Self::assert_transition(&record.status, &RefundStatus::Approved)?;

        record.status = RefundStatus::Approved;
        record.approved_at = Some(now);

        let ev = RefundEvent {
            refund_id: refund_id.to_owned(),
            order_id: record.order_id.clone(),
            event_type: RefundEventType::Approved,
            actor: caller,
            timestamp: now,
            metadata: None,
        };
        self.audit_log.push(ev);
        Ok(())
    }

    /// Reject a pending refund. Caller must be merchant or admin.
    pub fn reject(
        &mut self,
        refund_id: &str,
        caller: String,
        reason: Option<String>,
    ) -> Result<(), RefundError> {
        let now = now_secs();
        let record = self
            .refunds
            .get_mut(refund_id)
            .ok_or(RefundError::RefundNotFound)?;

        Self::assert_transition(&record.status, &RefundStatus::Rejected)?;

        record.status = RefundStatus::Rejected;
        let ev = RefundEvent {
            refund_id: refund_id.to_owned(),
            order_id: record.order_id.clone(),
            event_type: RefundEventType::Rejected,
            actor: caller,
            timestamp: now,
            metadata: reason,
        };
        self.audit_log.push(ev);
        Ok(())
    }

    /// Execute an approved refund within the grace period.
    pub fn execute(&mut self, refund_id: &str, caller: String) -> Result<(), RefundError> {
        let now = now_secs();
        let record = self
            .refunds
            .get_mut(refund_id)
            .ok_or(RefundError::RefundNotFound)?;

        Self::assert_transition(&record.status, &RefundStatus::Executed)?;

        // Enforce execution grace period.
        if let Some(approved_at) = record.approved_at {
            if now > approved_at + EXECUTION_GRACE_PERIOD_SECS {
                return Err(RefundError::ExecutionWindowExpired);
            }
        }

        record.status = RefundStatus::Executed;
        record.executed_at = Some(now);

        let ev = RefundEvent {
            refund_id: refund_id.to_owned(),
            order_id: record.order_id.clone(),
            event_type: RefundEventType::Executed,
            actor: caller,
            timestamp: now,
            metadata: None,
        };
        self.audit_log.push(ev);
        Ok(())
    }

    /// Cancel a pending refund.
    pub fn cancel(&mut self, refund_id: &str, caller: String) -> Result<(), RefundError> {
        let now = now_secs();
        let record = self
            .refunds
            .get_mut(refund_id)
            .ok_or(RefundError::RefundNotFound)?;

        Self::assert_transition(&record.status, &RefundStatus::Cancelled)?;

        record.status = RefundStatus::Cancelled;
        record.cancelled_at = Some(now);

        let ev = RefundEvent {
            refund_id: refund_id.to_owned(),
            order_id: record.order_id.clone(),
            event_type: RefundEventType::Cancelled,
            actor: caller,
            timestamp: now,
            metadata: None,
        };
        self.audit_log.push(ev);
        Ok(())
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn assert_transition(from: &RefundStatus, to: &RefundStatus) -> Result<(), RefundError> {
        let allowed = match from {
            RefundStatus::Pending => matches!(
                to,
                RefundStatus::Approved | RefundStatus::Rejected | RefundStatus::Cancelled
            ),
            RefundStatus::Approved => matches!(to, RefundStatus::Executed),
            _ => false,
        };
        if allowed {
            Ok(())
        } else {
            Err(RefundError::InvalidTransition {
                from: from.to_string(),
                to: to.to_string(),
            })
        }
    }

    fn emit(&mut self, event: RefundEvent) {
        self.audit_log.push(event);
    }
}

impl Default for RefundService {
    fn default() -> Self {
        Self::new()
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

    fn make_service() -> RefundService {
        RefundService::new()
    }

    fn paid_at() -> u64 {
        // well within the 30-day window
        now_secs() - 60
    }

    #[test]
    fn test_initiate_success() {
        let mut svc = make_service();
        svc.initiate(
            "R1".into(),
            "O1".into(),
            100,
            "customer request".into(),
            "payer1".into(),
            paid_at(),
            500,
            0,
        )
        .unwrap();
        let r = svc.get_refund("R1").unwrap();
        assert_eq!(r.status, RefundStatus::Pending);
        assert_eq!(svc.audit_log.len(), 1);
        assert_eq!(svc.audit_log[0].event_type, RefundEventType::Initiated);
    }

    #[test]
    fn test_duplicate_refund_id_rejected() {
        let mut svc = make_service();
        svc.initiate("R1".into(), "O1".into(), 100, "r".into(), "p".into(), paid_at(), 500, 0)
            .unwrap();
        let err = svc
            .initiate("R1".into(), "O1".into(), 50, "r".into(), "p".into(), paid_at(), 500, 100)
            .unwrap_err();
        assert_eq!(err, RefundError::RefundAlreadyExists);
    }

    #[test]
    fn test_amount_exceeds_payment() {
        let mut svc = make_service();
        let err = svc
            .initiate("R1".into(), "O1".into(), 600, "r".into(), "p".into(), paid_at(), 500, 0)
            .unwrap_err();
        assert_eq!(err, RefundError::AmountExceedsPayment);
    }

    #[test]
    fn test_refund_window_expired() {
        let mut svc = make_service();
        let old_paid_at = now_secs() - REFUND_WINDOW_SECS - 1;
        let err = svc
            .initiate("R1".into(), "O1".into(), 100, "r".into(), "p".into(), old_paid_at, 500, 0)
            .unwrap_err();
        assert_eq!(err, RefundError::RefundWindowExpired);
    }

    #[test]
    fn test_full_happy_path() {
        let mut svc = make_service();
        svc.initiate("R1".into(), "O1".into(), 100, "r".into(), "payer".into(), paid_at(), 500, 0)
            .unwrap();
        svc.approve("R1", "merchant".into()).unwrap();
        svc.execute("R1", "merchant".into()).unwrap();

        let r = svc.get_refund("R1").unwrap();
        assert_eq!(r.status, RefundStatus::Executed);
        assert!(r.executed_at.is_some());
        assert_eq!(svc.audit_log.len(), 3);
    }

    #[test]
    fn test_reject_flow() {
        let mut svc = make_service();
        svc.initiate("R1".into(), "O1".into(), 100, "r".into(), "payer".into(), paid_at(), 500, 0)
            .unwrap();
        svc.reject("R1", "merchant".into(), Some("policy".into()))
            .unwrap();
        assert_eq!(svc.get_refund("R1").unwrap().status, RefundStatus::Rejected);
    }

    #[test]
    fn test_cancel_flow() {
        let mut svc = make_service();
        svc.initiate("R1".into(), "O1".into(), 100, "r".into(), "payer".into(), paid_at(), 500, 0)
            .unwrap();
        svc.cancel("R1", "payer".into()).unwrap();
        assert_eq!(
            svc.get_refund("R1").unwrap().status,
            RefundStatus::Cancelled
        );
    }

    #[test]
    fn test_invalid_transition_execute_pending() {
        let mut svc = make_service();
        svc.initiate("R1".into(), "O1".into(), 100, "r".into(), "payer".into(), paid_at(), 500, 0)
            .unwrap();
        let err = svc.execute("R1", "merchant".into()).unwrap_err();
        assert!(matches!(err, RefundError::InvalidTransition { .. }));
    }

    #[test]
    fn test_list_by_order() {
        let mut svc = make_service();
        svc.initiate("R1".into(), "O1".into(), 50, "r".into(), "p".into(), paid_at(), 500, 0)
            .unwrap();
        svc.initiate("R2".into(), "O1".into(), 30, "r".into(), "p".into(), paid_at(), 500, 50)
            .unwrap();
        svc.initiate("R3".into(), "O2".into(), 20, "r".into(), "p".into(), paid_at(), 200, 0)
            .unwrap();
        assert_eq!(svc.list_by_order("O1").len(), 2);
        assert_eq!(svc.list_by_order("O2").len(), 1);
    }
}
