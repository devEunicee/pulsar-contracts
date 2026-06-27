/// Advanced Search & Filtering Service — Issue #275
///
/// Provides cursor-based paginated search across payments, merchants, and refunds
/// with multi-field sorting and combined filter support.

use crate::types::{
    MerchantPage, MerchantRecord, MerchantSearchQuery, MerchantSortField,
    PaymentPage, PaymentRecord, PaymentSearchQuery, PaymentSortField,
    RefundPage, RefundRecord, RefundSearchQuery, RefundSortField,
    SortOrder,
};

pub const DEFAULT_PAGE_LIMIT: u32 = 20;
pub const MAX_PAGE_LIMIT: u32 = 100;

// ── Repository traits ─────────────────────────────────────────────────────────

pub trait PaymentIndex: Send + Sync {
    /// Return all payment records (implementations should use DB indexes).
    fn all(&self) -> Vec<PaymentRecord>;
}

pub trait MerchantIndex: Send + Sync {
    fn all(&self) -> Vec<MerchantRecord>;
}

pub trait RefundIndex: Send + Sync {
    fn all(&self) -> Vec<RefundRecord>;
}

// ── Search service ────────────────────────────────────────────────────────────

pub struct SearchService<P, M, R>
where
    P: PaymentIndex,
    M: MerchantIndex,
    R: RefundIndex,
{
    payments: P,
    merchants: M,
    refunds: R,
}

impl<P: PaymentIndex, M: MerchantIndex, R: RefundIndex> SearchService<P, M, R> {
    pub fn new(payments: P, merchants: M, refunds: R) -> Self {
        Self { payments, merchants, refunds }
    }

    // ── Payments ──────────────────────────────────────────────────────────────

    pub fn search_payments(&self, q: PaymentSearchQuery) -> PaymentPage {
        let limit = q.limit.unwrap_or(DEFAULT_PAGE_LIMIT).min(MAX_PAGE_LIMIT) as usize;

        let mut records: Vec<PaymentRecord> = self
            .payments
            .all()
            .into_iter()
            .filter(|r| {
                if let Some(ref a) = q.merchant_address {
                    if !r.merchant_address.eq_ignore_ascii_case(a) { return false; }
                }
                if let Some(ref a) = q.payer_address {
                    if !r.payer_address.eq_ignore_ascii_case(a) { return false; }
                }
                if let Some(ref t) = q.token_address {
                    if !r.token_address.eq_ignore_ascii_case(t) { return false; }
                }
                if let Some(min) = q.amount_min { if r.amount < min { return false; } }
                if let Some(max) = q.amount_max { if r.amount > max { return false; } }
                if let Some(start) = q.date_start { if r.paid_at < start { return false; } }
                if let Some(end) = q.date_end   { if r.paid_at > end   { return false; } }
                if let Some(ref statuses) = q.statuses {
                    let matched = statuses.iter().any(|s| format!("{:?}", s) == r.status);
                    if !matched { return false; }
                }
                true
            })
            .collect();

        // Multi-field sort
        let sort_field = q.sort_field.unwrap_or(PaymentSortField::Date);
        let asc = q.sort_order.as_ref().map_or(false, |o| *o == SortOrder::Ascending);
        records.sort_by(|a, b| {
            let ord = match sort_field {
                PaymentSortField::Date            => a.paid_at.cmp(&b.paid_at),
                PaymentSortField::Amount          => a.amount.cmp(&b.amount),
                PaymentSortField::MerchantAddress => a.merchant_address.cmp(&b.merchant_address),
                PaymentSortField::PayerAddress    => a.payer_address.cmp(&b.payer_address),
            };
            if asc { ord } else { ord.reverse() }
        });

        let total = records.len() as u64;
        let (records, next_cursor) = paginate(records, q.cursor.as_deref(), limit, |r| r.order_id.clone());
        PaymentPage { records, next_cursor, total }
    }

    // ── Merchants ─────────────────────────────────────────────────────────────

    pub fn search_merchants(&self, q: MerchantSearchQuery) -> MerchantPage {
        let limit = q.limit.unwrap_or(DEFAULT_PAGE_LIMIT).min(MAX_PAGE_LIMIT) as usize;

        let mut records: Vec<MerchantRecord> = self
            .merchants
            .all()
            .into_iter()
            .filter(|r| {
                if let Some(ref name) = q.name_contains {
                    if !r.name.to_lowercase().contains(&name.to_lowercase()) { return false; }
                }
                if let Some(ref cat) = q.category {
                    if !r.category.eq_ignore_ascii_case(cat) { return false; }
                }
                if let Some(active) = q.active { if r.active != active { return false; } }
                if let Some(wl)     = q.whitelisted { if r.whitelisted != wl { return false; } }
                true
            })
            .collect();

        let sort_field = q.sort_field.unwrap_or(MerchantSortField::Name);
        let asc = q.sort_order.as_ref().map_or(true, |o| *o == SortOrder::Ascending);
        records.sort_by(|a, b| {
            let ord = match sort_field {
                MerchantSortField::Name         => a.name.cmp(&b.name),
                MerchantSortField::RegisteredAt => a.registered_at.cmp(&b.registered_at),
                MerchantSortField::Category     => a.category.cmp(&b.category),
            };
            if asc { ord } else { ord.reverse() }
        });

        let total = records.len() as u64;
        let (records, next_cursor) = paginate(records, q.cursor.as_deref(), limit, |r| r.address.clone());
        MerchantPage { records, next_cursor, total }
    }

    // ── Refunds ───────────────────────────────────────────────────────────────

    pub fn search_refunds(&self, q: RefundSearchQuery) -> RefundPage {
        let limit = q.limit.unwrap_or(DEFAULT_PAGE_LIMIT).min(MAX_PAGE_LIMIT) as usize;

        let mut records: Vec<RefundRecord> = self
            .refunds
            .all()
            .into_iter()
            .filter(|r| {
                if let Some(ref oid) = q.order_id { if r.order_id != *oid { return false; } }
                if let Some(ref by) = q.initiated_by { if !r.initiated_by.eq_ignore_ascii_case(by) { return false; } }
                if let Some(ref statuses) = q.statuses {
                    if !statuses.iter().any(|s| s.eq_ignore_ascii_case(&r.status)) { return false; }
                }
                if let Some(min) = q.amount_min { if r.amount < min { return false; } }
                if let Some(max) = q.amount_max { if r.amount > max { return false; } }
                if let Some(start) = q.date_start { if r.initiated_at < start { return false; } }
                if let Some(end)   = q.date_end   { if r.initiated_at > end   { return false; } }
                true
            })
            .collect();

        let sort_field = q.sort_field.unwrap_or(RefundSortField::InitiatedAt);
        let asc = q.sort_order.as_ref().map_or(false, |o| *o == SortOrder::Ascending);
        records.sort_by(|a, b| {
            let ord = match sort_field {
                RefundSortField::InitiatedAt => a.initiated_at.cmp(&b.initiated_at),
                RefundSortField::Amount      => a.amount.cmp(&b.amount),
            };
            if asc { ord } else { ord.reverse() }
        });

        let total = records.len() as u64;
        let (records, next_cursor) = paginate(records, q.cursor.as_deref(), limit, |r| r.refund_id.clone());
        RefundPage { records, next_cursor, total }
    }
}

// ── Cursor-based pagination ───────────────────────────────────────────────────

/// Slices `items` starting after the item whose key matches `cursor`.
/// Returns (page_items, next_cursor).
fn paginate<T, F>(items: Vec<T>, cursor: Option<&str>, limit: usize, key_fn: F) -> (Vec<T>, Option<String>)
where
    F: Fn(&T) -> String,
{
    let start = match cursor {
        None => 0,
        Some(c) => items.iter().position(|i| key_fn(i) == c).map_or(0, |p| p + 1),
    };
    let slice: Vec<T> = items.into_iter().skip(start).take(limit + 1).collect();
    if slice.len() > limit {
        let mut page = slice;
        let extra = page.pop().unwrap();
        let next = key_fn(&extra);
        (page, Some(next))
    } else {
        (slice, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn payments() -> Vec<PaymentRecord> {
        vec![
            PaymentRecord { order_id: "P1".into(), merchant_address: "M1".into(), payer_address: "A1".into(), token_address: "T1".into(), amount: 500, refunded_amount: 0, status: "Completed".into(), paid_at: 1000 },
            PaymentRecord { order_id: "P2".into(), merchant_address: "M2".into(), payer_address: "A2".into(), token_address: "T1".into(), amount: 200, refunded_amount: 0, status: "Completed".into(), paid_at: 2000 },
            PaymentRecord { order_id: "P3".into(), merchant_address: "M1".into(), payer_address: "A1".into(), token_address: "T1".into(), amount: 800, refunded_amount: 0, status: "PartiallyRefunded".into(), paid_at: 3000 },
        ]
    }

    struct StaticPayments(Vec<PaymentRecord>);
    struct StaticMerchants(Vec<MerchantRecord>);
    struct StaticRefunds(Vec<RefundRecord>);
    impl PaymentIndex for StaticPayments { fn all(&self) -> Vec<PaymentRecord> { self.0.clone() } }
    impl MerchantIndex for StaticMerchants { fn all(&self) -> Vec<MerchantRecord> { self.0.clone() } }
    impl RefundIndex for StaticRefunds { fn all(&self) -> Vec<RefundRecord> { self.0.clone() } }

    fn svc() -> SearchService<StaticPayments, StaticMerchants, StaticRefunds> {
        SearchService::new(
            StaticPayments(payments()),
            StaticMerchants(vec![
                MerchantRecord { address: "M1".into(), name: "Coffee House".into(), category: "Food".into(), active: true, whitelisted: false, registered_at: 100 },
                MerchantRecord { address: "M2".into(), name: "Tech Store".into(), category: "Digital".into(), active: false, whitelisted: true, registered_at: 200 },
            ]),
            StaticRefunds(vec![
                RefundRecord { refund_id: "R1".into(), order_id: "P1".into(), amount: 100, status: "Pending".into(), initiated_by: "A1".into(), initiated_at: 1500 },
            ]),
        )
    }

    #[test] fn filter_by_merchant() {
        let page = svc().search_payments(PaymentSearchQuery { merchant_address: Some("M1".into()), ..Default::default() });
        assert_eq!(page.records.len(), 2);
    }

    #[test] fn filter_by_amount_range() {
        let page = svc().search_payments(PaymentSearchQuery { amount_min: Some(300), amount_max: Some(600), ..Default::default() });
        assert_eq!(page.records.len(), 1);
        assert_eq!(page.records[0].order_id, "P1");
    }

    #[test] fn filter_by_date_range() {
        let page = svc().search_payments(PaymentSearchQuery { date_start: Some(1500), date_end: Some(2500), ..Default::default() });
        assert_eq!(page.records.len(), 1);
        assert_eq!(page.records[0].order_id, "P2");
    }

    #[test] fn sort_by_amount_ascending() {
        let page = svc().search_payments(PaymentSearchQuery { sort_field: Some(PaymentSortField::Amount), sort_order: Some(SortOrder::Ascending), ..Default::default() });
        assert_eq!(page.records[0].amount, 200);
    }

    #[test] fn cursor_pagination() {
        let page1 = svc().search_payments(PaymentSearchQuery { limit: Some(2), sort_field: Some(PaymentSortField::Date), sort_order: Some(SortOrder::Descending), ..Default::default() });
        assert_eq!(page1.records.len(), 2);
        assert!(page1.next_cursor.is_some());
        let page2 = svc().search_payments(PaymentSearchQuery { limit: Some(2), cursor: page1.next_cursor, sort_field: Some(PaymentSortField::Date), sort_order: Some(SortOrder::Descending), ..Default::default() });
        assert_eq!(page2.records.len(), 1);
        assert!(page2.next_cursor.is_none());
    }

    #[test] fn merchant_text_search() {
        let page = svc().search_merchants(MerchantSearchQuery { name_contains: Some("coffee".into()), ..Default::default() });
        assert_eq!(page.records.len(), 1);
    }

    #[test] fn merchant_filter_active() {
        let page = svc().search_merchants(MerchantSearchQuery { active: Some(false), ..Default::default() });
        assert_eq!(page.records.len(), 1);
        assert_eq!(page.records[0].address, "M2");
    }

    #[test] fn refund_filter_by_order() {
        let page = svc().search_refunds(RefundSearchQuery { order_id: Some("P1".into()), ..Default::default() });
        assert_eq!(page.records.len(), 1);
    }
}
