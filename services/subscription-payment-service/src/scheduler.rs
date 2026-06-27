//! Scheduler for subscription payment processing

use crate::{EventEmitter, InvoiceGenerator, PaymentProcessor, SubscriptionConfig, Result};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

/// Subscription scheduler
pub struct SubscriptionScheduler {
    processor: Arc<PaymentProcessor>,
    emitter: Arc<EventEmitter>,
    invoice_gen: Arc<InvoiceGenerator>,
    config: SubscriptionConfig,
    running: Arc<RwLock<bool>>,
}

impl SubscriptionScheduler {
    pub fn new(
        _pool: Arc<sqlx::PgPool>,
        processor: Arc<PaymentProcessor>,
        emitter: Arc<EventEmitter>,
        invoice_gen: Arc<InvoiceGenerator>,
        config: SubscriptionConfig,
    ) -> Self {
        Self {
            processor,
            emitter,
            invoice_gen,
            config,
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start the scheduler
    pub async fn start(&self) -> Result<()> {
        let mut is_running = self.running.write().await;
        if *is_running {
            return Err(crate::Error::SchedulingError(
                "Scheduler is already running".to_string(),
            ));
        }
        *is_running = true;
        drop(is_running);

        tracing::info!("Subscription scheduler started");

        // Spawn payment processing task
        if self.config.enable_auto_payments {
            self.spawn_payment_processor();
        }

        Ok(())
    }

    /// Stop the scheduler
    pub async fn stop(&self) -> Result<()> {
        let mut is_running = self.running.write().await;
        *is_running = false;
        tracing::info!("Subscription scheduler stopped");
        Ok(())
    }

    fn spawn_payment_processor(&self) {
        let processor = self.processor.clone();
        let emitter = self.emitter.clone();
        let _invoice_gen = self.invoice_gen.clone();
        let interval_secs = self.config.payment_interval_secs;
        let running = self.running.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(interval_secs));
            loop {
                interval.tick().await;
                
                let is_running = running.read().await;
                if !*is_running {
                    break;
                }
                drop(is_running);

                // Process due payments
                match processor.process_due_payments().await {
                    Ok(count) => {
                        if count > 0 {
                            tracing::info!("Processed {} due payments", count);
                            
                            // Emit event
                            if let Err(e) = emitter.emit(
                                "system",
                                "payments_processed",
                                serde_json::json!({"count": count})
                            ).await {
                                tracing::error!("Failed to emit payment processed event: {:?}", e);
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Error processing payments: {:?}", e);
                    }
                }
            }
        });
    }
}
