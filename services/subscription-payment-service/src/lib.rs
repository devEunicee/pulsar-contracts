//! Subscription Payment Service
//!
//! This service manages recurring subscription payments with support for:
//! - Subscription creation and configuration
//! - Recurring payment scheduling
//! - Subscription pausing/resumption
//! - Automatic retry on failure
//! - Payment attempt history
//! - Subscription cancellation
//! - Invoice generation
//! - Event emission

pub mod config;
pub mod error;
pub mod events;
pub mod invoice;
pub mod models;
pub mod payment;
pub mod scheduler;

pub use config::SubscriptionConfig;
pub use error::{Error, Result};
pub use events::EventEmitter;
pub use invoice::InvoiceGenerator;
pub use models::{Subscription, SubscriptionStatus, PaymentAttempt, Invoice};
pub use payment::PaymentProcessor;
pub use scheduler::SubscriptionScheduler;

use sqlx::PgPool;
use std::sync::Arc;

/// Main service for managing subscriptions
pub struct SubscriptionPaymentService {
    pool: Arc<PgPool>,
    config: SubscriptionConfig,
    processor: Arc<PaymentProcessor>,
    emitter: Arc<EventEmitter>,
    invoice_gen: Arc<InvoiceGenerator>,
    scheduler: Arc<SubscriptionScheduler>,
}

impl SubscriptionPaymentService {
    /// Create a new subscription payment service
    pub async fn new(
        pool: PgPool,
        config: SubscriptionConfig,
    ) -> Result<Self> {
        let pool = Arc::new(pool);
        let processor = Arc::new(PaymentProcessor::new(pool.clone(), config.clone()));
        let emitter = Arc::new(EventEmitter::new(pool.clone()));
        let invoice_gen = Arc::new(InvoiceGenerator::new(pool.clone()));
        let scheduler = Arc::new(SubscriptionScheduler::new(
            pool.clone(),
            processor.clone(),
            emitter.clone(),
            invoice_gen.clone(),
            config.clone(),
        ));

        Ok(Self {
            pool,
            config,
            processor,
            emitter,
            invoice_gen,
            scheduler,
        })
    }

    /// Start the service
    pub async fn start(&self) -> Result<()> {
        tracing::info!("Starting subscription payment service");
        
        // Initialize database
        self.initialize_db().await?;
        
        // Start scheduler
        self.scheduler.start().await?;
        
        tracing::info!("Subscription payment service started successfully");
        Ok(())
    }

    /// Stop the service
    pub async fn stop(&self) -> Result<()> {
        tracing::info!("Stopping subscription payment service");
        self.scheduler.stop().await?;
        Ok(())
    }

    /// Initialize database tables
    async fn initialize_db(&self) -> Result<()> {
        sqlx::query(include_str!("../schema.sql"))
            .execute(self.pool.as_ref())
            .await?;
        Ok(())
    }

    /// Create a new subscription
    pub async fn create_subscription(
        &self,
        subscription: Subscription,
    ) -> Result<String> {
        self.processor.create_subscription(subscription).await
    }

    /// Get subscription details
    pub async fn get_subscription(&self, subscription_id: &str) -> Result<Option<Subscription>> {
        self.processor.get_subscription(subscription_id).await
    }

    /// Pause subscription
    pub async fn pause_subscription(&self, subscription_id: &str) -> Result<()> {
        self.processor.pause_subscription(subscription_id).await
    }

    /// Resume subscription
    pub async fn resume_subscription(&self, subscription_id: &str) -> Result<()> {
        self.processor.resume_subscription(subscription_id).await
    }

    /// Cancel subscription
    pub async fn cancel_subscription(&self, subscription_id: &str, reason: &str) -> Result<()> {
        self.processor.cancel_subscription(subscription_id, reason).await
    }

    /// Get payment attempts for subscription
    pub async fn get_payment_attempts(&self, subscription_id: &str) -> Result<Vec<PaymentAttempt>> {
        self.processor.get_payment_attempts(subscription_id).await
    }

    /// Generate invoice
    pub async fn generate_invoice(&self, subscription_id: &str) -> Result<Invoice> {
        self.invoice_gen.generate_invoice(subscription_id).await
    }

    /// Get payment processor
    pub fn processor(&self) -> &PaymentProcessor {
        &self.processor
    }

    /// Get event emitter
    pub fn emitter(&self) -> &EventEmitter {
        &self.emitter
    }

    /// Get invoice generator
    pub fn invoice_gen(&self) -> &InvoiceGenerator {
        &self.invoice_gen
    }
}
