pub mod service;
pub mod types;

pub use service::{MerchantRepository, MerchantService, ServiceError};
pub use types::{AuditEntry, Merchant, MerchantCategory, MerchantFilter, MerchantStats, UpdateMerchantRequest};
