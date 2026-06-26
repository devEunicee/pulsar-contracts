use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, MultisigServiceError>;

#[derive(Debug, Error)]
pub enum MultisigServiceError {
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("payment not found")]
    PaymentNotFound,
    #[error("multisig payment already exists")]
    DuplicateMultisig,
    #[error("not authorized")]
    Unauthorized,
    #[error("payment already executed")]
    AlreadyExecuted,
    #[error("multisig payment expired")]
    Expired,
    #[error("insufficient signatures")]
    InsufficientSignatures,
    #[error("invalid input")]
    InvalidInput,
    #[error("internal error")]
    Internal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PaymentOrder {
    pub order_id: String,
    pub merchant_address: String,
    pub payer_address: String,
    pub token_address: String,
    pub amount: i128,
    pub description: String,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MultisigPayment {
    pub payment_id: String,
    pub order: PaymentOrder,
    pub required_signers: Vec<String>,
    pub signatures: Vec<String>,
    pub executed: bool,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MultisigEvent {
    Initiated { payment_id: String },
    Signed { payment_id: String, signer: String },
    Rejected { payment_id: String, signer: String },
    Executed { payment_id: String },
    Cancelled { payment_id: String },
}

#[derive(Debug, Default)]
pub struct MultisigService {
    payments: HashMap<String, MultisigPayment>,
    events: Vec<MultisigEvent>,
}

impl MultisigService {
    pub fn new() -> Self {
        Self {
            payments: HashMap::new(),
            events: Vec::new(),
        }
    }

    pub fn initiate_multisig_payment(
        &mut self,
        initiator: String,
        payment_id: String,
        order: PaymentOrder,
        required_signers: Vec<String>,
        timeout_seconds: u64,
    ) -> Result<&MultisigPayment> {
        if required_signers.is_empty() || required_signers.len() > 10 {
            return Err(MultisigServiceError::InvalidInput);
        }
        if self.payments.contains_key(&payment_id) {
            return Err(MultisigServiceError::DuplicateMultisig);
        }

        let mut unique_signers = Vec::new();
        for signer in required_signers.iter() {
            if unique_signers.contains(signer) {
                return Err(MultisigServiceError::InvalidInput);
            }
            unique_signers.push(signer.clone());
        }

        let now = Utc::now();
        let expires_at = now + chrono::Duration::seconds(timeout_seconds as i64);

        let payment = MultisigPayment {
            payment_id: payment_id.clone(),
            order,
            required_signers: unique_signers,
            signatures: Vec::new(),
            executed: false,
            expires_at,
            created_at: now,
        };

        self.payments.insert(payment_id.clone(), payment);
        self.events.push(MultisigEvent::Initiated { payment_id: payment_id.clone() });
        Ok(self.payments.get(&payment_id).unwrap())
    }

    pub fn sign_multisig_payment(&mut self, signer: String, payment_id: &str) -> Result<&MultisigPayment> {
        let payment = self
            .payments
            .get_mut(payment_id)
            .ok_or(MultisigServiceError::PaymentNotFound)?;

        if payment.executed {
            return Err(MultisigServiceError::AlreadyExecuted);
        }
        if Utc::now() > payment.expires_at {
            return Err(MultisigServiceError::Expired);
        }
        if !payment.required_signers.contains(&signer) {
            return Err(MultisigServiceError::Unauthorized);
        }
        if payment.signatures.contains(&signer) {
            return Err(MultisigServiceError::InvalidInput);
        }

        payment.signatures.push(signer.clone());
        self.events.push(MultisigEvent::Signed {
            payment_id: payment_id.to_string(),
            signer,
        });
        Ok(payment)
    }

    pub fn execute_multisig_payment(&mut self, executor: String, payment_id: &str) -> Result<&MultisigPayment> {
        let payment = self
            .payments
            .get_mut(payment_id)
            .ok_or(MultisigServiceError::PaymentNotFound)?;

        if payment.executed {
            return Err(MultisigServiceError::AlreadyExecuted);
        }
        if Utc::now() > payment.expires_at {
            return Err(MultisigServiceError::Expired);
        }
        if payment.signatures.len() < payment.required_signers.len() {
            return Err(MultisigServiceError::InsufficientSignatures);
        }

        payment.executed = true;
        self.events.push(MultisigEvent::Executed { payment_id: payment_id.to_string() });
        Ok(payment)
    }

    pub fn cancel_multisig_payment(&mut self, initiator: String, payment_id: &str) -> Result<&MultisigPayment> {
        let payment = self
            .payments
            .get_mut(payment_id)
            .ok_or(MultisigServiceError::PaymentNotFound)?;

        if payment.order.payer_address != initiator {
            return Err(MultisigServiceError::Unauthorized);
        }
        if payment.executed {
            return Err(MultisigServiceError::AlreadyExecuted);
        }

        payment.executed = true;
        self.events.push(MultisigEvent::Cancelled { payment_id: payment_id.to_string() });
        Ok(payment)
    }

    pub fn get_multisig_payment(&self, payment_id: &str) -> Option<&MultisigPayment> {
        self.payments.get(payment_id)
    }

    pub fn get_events(&self) -> &[MultisigEvent] {
        &self.events
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_order() -> PaymentOrder {
        PaymentOrder {
            order_id: "MS_001".into(),
            merchant_address: "MERCHANT_001".into(),
            payer_address: "PAYER_001".into(),
            token_address: "TOKEN_001".into(),
            amount: 1000,
            description: "Multisig payment".into(),
            expires_at: None,
        }
    }

    #[test]
    fn initiate_multisig_payment_success() {
        let mut svc = MultisigService::new();
        let payment = svc
            .initiate_multisig_payment(
                "PAYER_001".into(),
                "MS_001".into(),
                sample_order(),
                vec!["SIGNER_1".into(), "SIGNER_2".into()],
                3600,
            )
            .unwrap();

        assert_eq!(payment.required_signers.len(), 2);
        assert!(!payment.executed);
    }

    #[test]
    fn sign_multisig_payment_and_execute() {
        let mut svc = MultisigService::new();
        svc.initiate_multisig_payment(
            "PAYER_001".into(),
            "MS_002".into(),
            sample_order(),
            vec!["SIGNER_1".into(), "SIGNER_2".into()],
            3600,
        )
        .unwrap();

        svc.sign_multisig_payment("SIGNER_1".into(), "MS_002").unwrap();
        svc.sign_multisig_payment("SIGNER_2".into(), "MS_002").unwrap();
        let payment = svc.execute_multisig_payment("SIGNER_1".into(), "MS_002").unwrap();

        assert!(payment.executed);
        assert_eq!(svc.get_events().len(), 4);
    }
}
