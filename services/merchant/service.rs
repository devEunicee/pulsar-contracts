/// Merchant Management Service — Issue #273
///
/// Handles merchant lifecycle: registration, activation/deactivation,
/// profile updates, whitelist enforcement, stats aggregation, search, and audit logging.

use crate::types::{AuditEntry, Merchant, MerchantCategory, MerchantFilter, MerchantStats, UpdateMerchantRequest};

pub trait MerchantRepository: Send + Sync {
    fn find_by_address(&self, address: &str) -> Option<Merchant>;
    fn save(&mut self, merchant: &Merchant);
    fn list(&self, filter: &MerchantFilter) -> Vec<Merchant>;
    fn append_audit(&mut self, entry: AuditEntry);
    fn get_stats(&self, address: &str) -> Option<MerchantStats>;
}

pub struct MerchantService<R: MerchantRepository> {
    repo: R,
}

impl<R: MerchantRepository> MerchantService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    /// Register a new merchant. Returns error if already registered.
    pub fn register(
        &mut self,
        address: String,
        name: String,
        description: String,
        contact_info: String,
        category: MerchantCategory,
        caller: &str,
        now: u64,
    ) -> Result<Merchant, ServiceError> {
        if self.repo.find_by_address(&address).is_some() {
            return Err(ServiceError::AlreadyRegistered);
        }
        validate_name(&name)?;

        let merchant = Merchant {
            address: address.clone(),
            name,
            description,
            contact_info,
            category,
            active: true,
            whitelisted: false,
            registered_at: now,
            updated_at: now,
        };
        self.repo.save(&merchant);
        self.repo.append_audit(AuditEntry {
            merchant_address: address,
            action: "registered".into(),
            changed_by: caller.into(),
            changed_at: now,
            details: None,
        });
        Ok(merchant)
    }

    /// Deactivate an active merchant.
    pub fn deactivate(&mut self, address: &str, caller: &str, now: u64) -> Result<(), ServiceError> {
        let mut m = self.repo.find_by_address(address).ok_or(ServiceError::NotFound)?;
        if !m.active {
            return Err(ServiceError::AlreadyInactive);
        }
        m.active = false;
        m.updated_at = now;
        self.repo.save(&m);
        self.repo.append_audit(AuditEntry {
            merchant_address: address.into(),
            action: "deactivated".into(),
            changed_by: caller.into(),
            changed_at: now,
            details: None,
        });
        Ok(())
    }

    /// Reactivate an inactive merchant.
    pub fn activate(&mut self, address: &str, caller: &str, now: u64) -> Result<(), ServiceError> {
        let mut m = self.repo.find_by_address(address).ok_or(ServiceError::NotFound)?;
        if m.active {
            return Err(ServiceError::AlreadyActive);
        }
        m.active = true;
        m.updated_at = now;
        self.repo.save(&m);
        self.repo.append_audit(AuditEntry {
            merchant_address: address.into(),
            action: "activated".into(),
            changed_by: caller.into(),
            changed_at: now,
            details: None,
        });
        Ok(())
    }

    /// Update mutable profile fields.
    pub fn update_profile(
        &mut self,
        address: &str,
        req: UpdateMerchantRequest,
        caller: &str,
        now: u64,
    ) -> Result<Merchant, ServiceError> {
        let mut m = self.repo.find_by_address(address).ok_or(ServiceError::NotFound)?;
        let mut changed = Vec::new();
        if let Some(name) = req.name {
            validate_name(&name)?;
            changed.push(format!("name:{}", name));
            m.name = name;
        }
        if let Some(desc) = req.description {
            changed.push("description".into());
            m.description = desc;
        }
        if let Some(ci) = req.contact_info {
            changed.push("contact_info".into());
            m.contact_info = ci;
        }
        if let Some(cat) = req.category {
            changed.push(format!("category:{:?}", cat));
            m.category = cat;
        }
        if changed.is_empty() {
            return Ok(m);
        }
        m.updated_at = now;
        self.repo.save(&m);
        self.repo.append_audit(AuditEntry {
            merchant_address: address.into(),
            action: "updated".into(),
            changed_by: caller.into(),
            changed_at: now,
            details: Some(changed.join(",")),
        });
        Ok(m)
    }

    /// Set whitelist flag (admin only — caller must enforce auth before calling).
    pub fn set_whitelist(&mut self, address: &str, whitelisted: bool, caller: &str, now: u64) -> Result<(), ServiceError> {
        let mut m = self.repo.find_by_address(address).ok_or(ServiceError::NotFound)?;
        m.whitelisted = whitelisted;
        m.updated_at = now;
        self.repo.save(&m);
        self.repo.append_audit(AuditEntry {
            merchant_address: address.into(),
            action: if whitelisted { "whitelisted" } else { "unwhitelisted" }.into(),
            changed_by: caller.into(),
            changed_at: now,
            details: None,
        });
        Ok(())
    }

    /// Get a single merchant by address.
    pub fn get(&self, address: &str) -> Option<Merchant> {
        self.repo.find_by_address(address)
    }

    /// Search merchants with optional filters.
    pub fn search(&self, filter: MerchantFilter) -> Vec<Merchant> {
        self.repo.list(&filter)
    }

    /// Get aggregated payment/refund stats for a merchant.
    pub fn get_stats(&self, address: &str) -> Result<MerchantStats, ServiceError> {
        self.repo.get_stats(address).ok_or(ServiceError::NotFound)
    }
}

fn validate_name(name: &str) -> Result<(), ServiceError> {
    if name.trim().is_empty() || name.len() > 255 {
        return Err(ServiceError::InvalidInput("name must be 1–255 characters".into()));
    }
    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
pub enum ServiceError {
    NotFound,
    AlreadyRegistered,
    AlreadyActive,
    AlreadyInactive,
    InvalidInput(String),
}

impl std::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceError::NotFound => write!(f, "merchant not found"),
            ServiceError::AlreadyRegistered => write!(f, "merchant already registered"),
            ServiceError::AlreadyActive => write!(f, "merchant already active"),
            ServiceError::AlreadyInactive => write!(f, "merchant already inactive"),
            ServiceError::InvalidInput(msg) => write!(f, "invalid input: {msg}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct InMemoryRepo {
        merchants: HashMap<String, Merchant>,
        audit: Vec<AuditEntry>,
    }

    impl InMemoryRepo {
        fn new() -> Self {
            Self { merchants: HashMap::new(), audit: Vec::new() }
        }
    }

    impl MerchantRepository for InMemoryRepo {
        fn find_by_address(&self, address: &str) -> Option<Merchant> {
            self.merchants.get(address).cloned()
        }
        fn save(&mut self, merchant: &Merchant) {
            self.merchants.insert(merchant.address.clone(), merchant.clone());
        }
        fn list(&self, filter: &MerchantFilter) -> Vec<Merchant> {
            self.merchants.values().filter(|m| {
                if let Some(ref name) = filter.name_contains {
                    if !m.name.to_lowercase().contains(&name.to_lowercase()) { return false; }
                }
                if let Some(ref cat) = filter.category {
                    if &m.category != cat { return false; }
                }
                if let Some(active) = filter.active {
                    if m.active != active { return false; }
                }
                if let Some(wl) = filter.whitelisted {
                    if m.whitelisted != wl { return false; }
                }
                true
            }).cloned().collect()
        }
        fn append_audit(&mut self, entry: AuditEntry) {
            self.audit.push(entry);
        }
        fn get_stats(&self, address: &str) -> Option<MerchantStats> {
            self.merchants.get(address).map(|_| MerchantStats {
                address: address.into(),
                payment_count: 0,
                total_volume: 0,
                refund_count: 0,
                total_refunded: 0,
            })
        }
    }

    fn svc() -> MerchantService<InMemoryRepo> {
        MerchantService::new(InMemoryRepo::new())
    }

    #[test]
    fn register_and_get() {
        let mut s = svc();
        let m = s.register("G1".into(), "Shop".into(), "".into(), "".into(), MerchantCategory::Retail, "admin", 100).unwrap();
        assert_eq!(m.address, "G1");
        assert!(m.active);
        assert!(!m.whitelisted);
        assert!(s.get("G1").is_some());
    }

    #[test]
    fn duplicate_registration_fails() {
        let mut s = svc();
        s.register("G1".into(), "Shop".into(), "".into(), "".into(), MerchantCategory::Retail, "admin", 100).unwrap();
        let err = s.register("G1".into(), "Shop2".into(), "".into(), "".into(), MerchantCategory::Food, "admin", 101);
        assert_eq!(err, Err(ServiceError::AlreadyRegistered));
    }

    #[test]
    fn deactivate_and_activate() {
        let mut s = svc();
        s.register("G1".into(), "Shop".into(), "".into(), "".into(), MerchantCategory::Retail, "admin", 100).unwrap();
        s.deactivate("G1", "admin", 200).unwrap();
        assert!(!s.get("G1").unwrap().active);
        s.activate("G1", "admin", 300).unwrap();
        assert!(s.get("G1").unwrap().active);
    }

    #[test]
    fn update_profile() {
        let mut s = svc();
        s.register("G1".into(), "Shop".into(), "".into(), "".into(), MerchantCategory::Retail, "admin", 100).unwrap();
        let updated = s.update_profile("G1", UpdateMerchantRequest { name: Some("New Name".into()), ..Default::default() }, "admin", 200).unwrap();
        assert_eq!(updated.name, "New Name");
    }

    #[test]
    fn search_by_name() {
        let mut s = svc();
        s.register("G1".into(), "Coffee House".into(), "".into(), "".into(), MerchantCategory::Food, "admin", 100).unwrap();
        s.register("G2".into(), "Tech Store".into(), "".into(), "".into(), MerchantCategory::Digital, "admin", 101).unwrap();
        let results = s.search(MerchantFilter { name_contains: Some("coffee".into()), ..Default::default() });
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].address, "G1");
    }

    #[test]
    fn whitelist_enforcement() {
        let mut s = svc();
        s.register("G1".into(), "Shop".into(), "".into(), "".into(), MerchantCategory::Retail, "admin", 100).unwrap();
        s.set_whitelist("G1", true, "admin", 200).unwrap();
        assert!(s.get("G1").unwrap().whitelisted);
    }

    #[test]
    fn invalid_name_rejected() {
        let mut s = svc();
        let err = s.register("G1".into(), "".into(), "".into(), "".into(), MerchantCategory::Retail, "admin", 100);
        assert!(matches!(err, Err(ServiceError::InvalidInput(_))));
    }
}
