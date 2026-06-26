/// Audit Logging System — Issue #278
///
/// Provides immutable, append-only audit log entries for all state-changing
/// operations. Entries are stored on-chain via persistent storage and
/// emitted as contract events for off-chain indexing.
///
/// Sensitive values (contact_info, description text) are not stored in the
/// log — only operation type, actor, target ID, and timestamp are recorded.
use soroban_sdk::{contracttype, symbol_short, Address, Bytes, Env, String, Vec};

use crate::types::DataKey;

// ── TTL re-use ────────────────────────────────────────────────────────────────

use crate::storage::{TTL_LEDGERS, TTL_THRESHOLD};

// ── Audit entry types ─────────────────────────────────────────────────────────

/// High-level action categories stored in each audit entry.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuditAction {
    /// Admin initialisation or upgrade.
    AdminSet,
    ContractUpgraded,
    /// Merchant lifecycle.
    MerchantRegistered,
    MerchantDeactivated,
    /// Payment lifecycle.
    PaymentProcessed,
    PaymentArchived,
    /// Refund lifecycle.
    RefundInitiated,
    RefundApproved,
    RefundRejected,
    RefundExecuted,
    /// Multi-sig lifecycle.
    MultisigInitiated,
    MultisigSigned,
    MultisigExecuted,
    /// Config changes.
    ConfigChanged,
    /// Whitelist management.
    WhitelistUpdated,
}

/// A single immutable audit log entry.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditEntry {
    /// Sequential log index (1-based).
    pub index: u64,
    /// Block/ledger timestamp at the time of the action.
    pub timestamp: u64,
    /// The action that was performed.
    pub action: AuditAction,
    /// Address of the actor (caller / admin) who triggered the action.
    /// Redacted to a zero address when the actor is anonymous.
    pub actor: Address,
    /// Stable opaque identifier of the affected entity (order_id, refund_id,
    /// merchant address bytes, etc.). Empty when not applicable.
    pub target_id: Bytes,
}

// ── Storage helpers ───────────────────────────────────────────────────────────

/// Return the current audit log count.
pub fn get_audit_count(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&DataKey::AuditCount)
        .unwrap_or(0u64)
}

fn set_audit_count(env: &Env, count: u64) {
    env.storage()
        .instance()
        .set(&DataKey::AuditCount, &count);
}

/// Persist a single audit entry.
fn save_entry(env: &Env, entry: &AuditEntry) {
    let key = DataKey::AuditEntry(entry.index);
    env.storage().persistent().set(&key, entry);
    env.storage()
        .persistent()
        .extend_ttl(&key, TTL_THRESHOLD, TTL_LEDGERS);
}

/// Retrieve a single audit entry by its 1-based index.
pub fn get_entry(env: &Env, index: u64) -> Option<AuditEntry> {
    let key = DataKey::AuditEntry(index);
    let result: Option<AuditEntry> = env.storage().persistent().get(&key);
    if result.is_some() {
        env.storage()
            .persistent()
            .extend_ttl(&key, TTL_THRESHOLD, TTL_LEDGERS);
    }
    result
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Append a new audit entry and emit a contract event.
/// This is the single write path — every state-changing function calls this.
pub fn log(env: &Env, action: AuditAction, actor: &Address, target_id: Bytes) {
    let next_index = get_audit_count(env) + 1;

    let entry = AuditEntry {
        index: next_index,
        timestamp: env.ledger().timestamp(),
        action: action.clone(),
        actor: actor.clone(),
        target_id: target_id.clone(),
    };

    save_entry(env, &entry);
    set_audit_count(env, next_index);

    // Emit an event so off-chain indexers can stream the log without reading
    // individual storage keys.
    env.events().publish(
        (symbol_short!("audit"),),
        (next_index, env.ledger().timestamp(), action, actor.clone(), target_id),
    );
}

/// Return a page of audit entries (most-recent-first).
///
/// `from_index` — 1-based start index (inclusive). Pass `0` to start from the
/// most recent entry.
/// `limit` — maximum entries to return (capped at 50).
pub fn get_entries(env: &Env, from_index: u64, limit: u32) -> Vec<AuditEntry> {
    let count = get_audit_count(env);
    if count == 0 {
        return Vec::new(env);
    }

    let capped_limit = limit.min(50) as u64;
    let start = if from_index == 0 || from_index > count {
        count
    } else {
        from_index
    };

    let mut results = Vec::new(env);
    let mut idx = start;
    let mut fetched = 0u64;

    while idx >= 1 && fetched < capped_limit {
        if let Some(entry) = get_entry(env, idx) {
            results.push_back(entry);
            fetched += 1;
        }
        if idx == 0 {
            break;
        }
        idx -= 1;
    }

    results
}
