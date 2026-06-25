//! Invoice generation for subscriptions

use crate::{models::*, Result};
use chrono::{DateTime, Duration, Utc};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

/// Invoice generator
pub struct InvoiceGenerator {
    pool: Arc<PgPool>,
}

impl InvoiceGenerator {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// Generate invoice for subscription
    pub async fn generate_invoice(&self, subscription_id: &str) -> Result<Invoice> {
        let subscription = sqlx::query(
            "SELECT merchant_id, customer_id, amount, currency FROM subscriptions WHERE id = $1"
        )
        .bind(subscription_id)
        .fetch_one(self.pool.as_ref())
        .await?;

        use sqlx::Row;
        let merchant_id: String = subscription.get("merchant_id");
        let customer_id: String = subscription.get("customer_id");
        let amount: String = subscription.get("amount");
        let currency: String = subscription.get("currency");

        let now = Utc::now();
        let due_date = now + Duration::days(30);
        let invoice_id = Uuid::new_v4().to_string();
        let invoice_number = format!("INV-{}-{}", 
            subscription_id.chars().take(8).collect::<String>(),
            now.format("%Y%m%d")
        );

        let invoice = Invoice {
            id: invoice_id.clone(),
            subscription_id: subscription_id.to_string(),
            merchant_id: merchant_id.clone(),
            customer_id: customer_id.clone(),
            amount: amount.clone(),
            currency: currency.clone(),
            status: "issued".to_string(),
            invoice_date: now,
            due_date,
            paid_at: None,
            invoice_number: invoice_number.clone(),
            line_items: vec![InvoiceLineItem {
                description: "Subscription Payment".to_string(),
                amount: amount.clone(),
                quantity: 1,
            }],
            created_at: now,
        };

        // Store invoice in database
        sqlx::query(
            r#"
            INSERT INTO invoices 
            (id, subscription_id, merchant_id, customer_id, amount, currency, status, invoice_date, due_date, invoice_number)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(&invoice.id)
        .bind(&invoice.subscription_id)
        .bind(&invoice.merchant_id)
        .bind(&invoice.customer_id)
        .bind(&invoice.amount)
        .bind(&invoice.currency)
        .bind(&invoice.status)
        .bind(invoice.invoice_date)
        .bind(invoice.due_date)
        .bind(&invoice_number)
        .execute(self.pool.as_ref())
        .await?;

        tracing::info!("Invoice generated: {} for subscription {}", invoice_id, subscription_id);
        Ok(invoice)
    }

    /// Mark invoice as paid
    pub async fn mark_invoice_paid(&self, invoice_id: &str) -> Result<()> {
        sqlx::query(
            "UPDATE invoices SET status = 'paid', paid_at = NOW() WHERE id = $1"
        )
        .bind(invoice_id)
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }
}
