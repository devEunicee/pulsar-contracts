use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, PaymentServiceError>;

#[derive(Debug, Error)]
pub enum PaymentServiceError {
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("signature verification failed")]
    SignatureVerification,
    #[error("payment already exists")]
    DuplicatePayment,
    #[error("payment not found")]
    PaymentNotFound,
    #[error("invalid state transition")]
    InvalidStateTransition,
    #[error("idempotency key conflict")]
    IdempotencyConflict,
    #[error("internal error")]
    Internal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PaymentStatus {
    Pending,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PaymentEvent {
    PaymentCreated { order_id: String },
    PaymentCompleted { order_id: String },
    PaymentFailed { order_id: String, reason: String },
    RefundInitiated { order_id: String, amount: i128 },
    RefundCompleted { order_id: String, amount: i128 },
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
pub struct PaymentRecord {
    pub order_id: String,
    pub merchant_address: String,
    pub payer_address: String,
    pub token_address: String,
    pub amount: i128,
    pub status: PaymentStatus,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignaturePayload {
    pub order: PaymentOrder,
    pub signature: String,
}

#[derive(Debug, Default)]
pub struct PaymentService {
    payments: HashMap<String, PaymentRecord>,
    idempotency_map: HashMap<String, String>,
    events: Vec<PaymentEvent>,
}

impl PaymentService {
    pub fn new() -> Self {
        Self {
            payments: HashMap::new(),
            idempotency_map: HashMap::new(),
            events: Vec::new(),
        }
    }

    pub fn submit_payment(
        &mut self,
        key: String,
        order: PaymentOrder,
        signature: String,
    ) -> Result<&PaymentRecord> {
        validate_amount(order.amount)?;
        validate_order_id(&order.order_id)?;

        if self.payments.contains_key(&order.order_id) {
            return Err(PaymentServiceError::DuplicatePayment);
        }

        if let Some(existing_order_id) = self.idempotency_map.get(&key) {
            if existing_order_id != &order.order_id {
                return Err(PaymentServiceError::IdempotencyConflict);
            }
            return self
                .payments
                .get(existing_order_id)
                .ok_or(PaymentServiceError::PaymentNotFound);
        }

        self.idempotency_map.insert(key.clone(), order.order_id.clone());
        self.events
            .push(PaymentEvent::PaymentCreated { order_id: order.order_id.clone() });

        let result = self.process_order(order.clone(), signature);
        let record = result?;
        self.payments.insert(order.order_id.clone(), record.clone());
        Ok(self.payments.get(&order.order_id).unwrap())
    }

    pub fn get_payment(&self, order_id: &str) -> Option<&PaymentRecord> {
        self.payments.get(order_id)
    }

    pub fn get_events(&self) -> &[PaymentEvent] {
        &self.events
    }

    fn process_order(&mut self, order: PaymentOrder, signature: String) -> Result<PaymentRecord> {
        match verify_signature(&order, &signature) {
            Ok(_) => {
                let now = Utc::now();
                let record = PaymentRecord {
                    order_id: order.order_id.clone(),
                    merchant_address: order.merchant_address.clone(),
                    payer_address: order.payer_address.clone(),
                    token_address: order.token_address.clone(),
                    amount: order.amount,
                    status: PaymentStatus::Completed,
                    created_at: now,
                    completed_at: Some(now),
                };
                self.events
                    .push(PaymentEvent::PaymentCompleted { order_id: order.order_id.clone() });
                Ok(record)
            }
            Err(error) => {
                self.events.push(PaymentEvent::PaymentFailed {
                    order_id: order.order_id.clone(),
                    reason: error.to_string(),
                });
                Err(error)
            }
        }
    }
}

pub fn validate_amount(amount: i128) -> Result<()> {
    if amount <= 0 {
        Err(PaymentServiceError::Validation("amount must be positive".into()))
    } else {
        Ok(())
    }
}

pub fn validate_order_id(order_id: &str) -> Result<()> {
    if order_id.is_empty() || order_id.len() > 64 {
        Err(PaymentServiceError::Validation(
            "order_id must be present and <= 64 chars".into(),
        ))
    } else {
        Ok(())
    }
}

pub fn verify_signature(_payload: &PaymentOrder, _signature: &str) -> Result<()> {
    Ok(())
}

pub fn create_payment(order: PaymentOrder, signature: String) -> Result<PaymentRecord> {
    let mut service = PaymentService::new();
    service
        .submit_payment("default".into(), order, signature)
        .map(|record| record.clone())
}

#[cfg(test)]
mod tests;
