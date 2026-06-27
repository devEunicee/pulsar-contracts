# Subscription Payment Service

A comprehensive service for managing recurring subscription payments in Pulsar.

## Features

- **Subscription Management**: Create, pause, resume, and cancel subscriptions
- **Recurring Payments**: Automatic payment processing at configured intervals
- **Flexible Billing**: Support for daily, weekly, bi-weekly, monthly, quarterly, and annual billing
- **Payment Attempts**: Track and retry failed payments with exponential backoff
- **Invoice Generation**: Automatic invoice creation and delivery
- **Event Emission**: Emit events on payment attempts and status changes
- **Event History**: Full audit trail of subscription and payment events
- **Payment History**: Detailed history of all payment attempts
- **Automatic Retry**: Smart retry logic with configurable attempts and delays
- **Past Due Tracking**: Automatic detection and tracking of past due subscriptions
- **Metrics Dashboard**: Real-time subscription and revenue metrics

## Configuration

Environment variables:

```bash
DATABASE_URL=postgresql://user:password@localhost/subscriptions
MAX_PAYMENT_RETRIES=3
RETRY_DELAY_SECS=300
ENABLE_AUTO_PAYMENTS=true
PAYMENT_INTERVAL_SECS=3600
ENABLE_INVOICE_GENERATION=true
ENABLE_EVENTS=true
PAYMENT_HISTORY_RETENTION_DAYS=365
LATE_PAYMENT_GRACE_HOURS=24
```

## Usage

```rust
use subscription_payment_service::{
    SubscriptionPaymentService, SubscriptionConfig, Subscription, BillingFrequency
};

#[tokio::main]
async fn main() {
    let config = SubscriptionConfig::from_env();
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect(&config.database_url)
        .await
        .unwrap();
    
    let service = SubscriptionPaymentService::new(pool, config).await.unwrap();
    service.start().await.unwrap();
    
    // Create subscription
    let sub = Subscription::new(
        "merchant_1".to_string(),
        "customer_1".to_string(),
        "99.99".to_string(),
        "USD".to_string(),
        BillingFrequency::Monthly,
    );
    
    let sub_id = service.create_subscription(sub).await.unwrap();
    
    // Pause subscription
    service.pause_subscription(&sub_id).await.unwrap();
    
    // Resume subscription
    service.resume_subscription(&sub_id).await.unwrap();
    
    // Cancel subscription
    service.cancel_subscription(&sub_id, "Customer requested").await.unwrap();
}
```

## API Reference

### Subscription Operations

#### Create Subscription
```rust
let subscription = Subscription::new(
    merchant_id,
    customer_id,
    amount,
    currency,
    frequency,
);
let sub_id = service.create_subscription(subscription).await?;
```

#### Get Subscription
```rust
if let Some(sub) = service.get_subscription(&sub_id).await? {
    println!("Status: {}", sub.status);
}
```

#### Pause Subscription
```rust
service.pause_subscription(&sub_id).await?;
```

#### Resume Subscription
```rust
service.resume_subscription(&sub_id).await?;
```

#### Cancel Subscription
```rust
service.cancel_subscription(&sub_id, "Reason").await?;
```

### Payment Operations

#### Get Payment Attempts
```rust
let attempts = service.get_payment_attempts(&sub_id).await?;
for attempt in attempts {
    println!("Attempt {}: {}", attempt.attempt_number, attempt.status);
}
```

### Invoice Operations

#### Generate Invoice
```rust
let invoice = service.generate_invoice(&sub_id).await?;
println!("Invoice {}: {}", invoice.invoice_number, invoice.amount);
```

### Event Operations

#### Emit Event
```rust
let event_id = service.emitter().emit(
    &sub_id,
    "payment_processed",
    serde_json::json!({"amount": "99.99"})
).await?;
```

#### Get Events
```rust
let events = service.emitter().get_events(&sub_id).await?;
for event in events {
    println!("{}: {}", event.event_type, event.data);
}
```

## Billing Frequencies

- **Daily**: Every day
- **Weekly**: Every 7 days
- **BiWeekly**: Every 14 days
- **Monthly**: Every 30 days
- **Quarterly**: Every 90 days
- **Annually**: Every 365 days

## Subscription Statuses

- **Active**: Subscription is active and payments are being processed
- **Paused**: Subscription is temporarily paused, no payments processed
- **PendingPayment**: Payment is pending, will retry
- **PastDue**: Payment is overdue beyond grace period
- **Cancelled**: Subscription has been cancelled

## Payment Retry Logic

Failed payments are automatically retried with exponential backoff:
- Attempt 1: Immediate
- Attempt 2: After 10 minutes (2^1 * 5 minutes)
- Attempt 3: After 20 minutes (2^2 * 5 minutes)
- Attempt 4: After 40 minutes (2^3 * 5 minutes)

After max retries, subscription moves to "past_due" status.

## Database Schema

### Tables

- **subscriptions**: Main subscription records
- **payment_attempts**: History of all payment attempts
- **invoices**: Generated invoices
- **subscription_events**: Event audit trail

### Indexes

All tables have appropriate indexes on frequently queried columns:
- merchant_id, customer_id
- status, next_payment_at
- created_at for time-based queries

### Views

- **subscription_status_summary**: Quick overview of subscriptions by status

### Functions

- **calculate_next_payment_date**: Calculate next payment date based on frequency
- **mark_past_due_subscriptions**: Mark overdue subscriptions
- **get_subscription_metrics**: Get subscription and revenue metrics

## Event Types

- `subscription_created`: New subscription created
- `subscription_paused`: Subscription paused
- `subscription_resumed`: Subscription resumed
- `subscription_cancelled`: Subscription cancelled
- `payment_attempted`: Payment attempt made
- `payment_succeeded`: Payment successful
- `payment_failed`: Payment failed
- `invoice_generated`: Invoice created
- `invoice_paid`: Invoice marked as paid

## Monitoring

### Query subscription metrics
```sql
SELECT * FROM subscription_status_summary;
```

### Get revenue metrics
```sql
SELECT * FROM get_subscription_metrics(30);
```

### Find past due subscriptions
```sql
SELECT * FROM subscriptions WHERE status = 'past_due' ORDER BY next_payment_at;
```

### Check payment failure rates
```sql
SELECT 
  DATE(attempted_at) as date,
  status,
  COUNT(*) as count
FROM payment_attempts
GROUP BY DATE(attempted_at), status
ORDER BY date DESC;
```

## Performance Considerations

- Indexes on merchant_id, customer_id, status for fast lookups
- Batch payment processing for efficiency
- Configurable retention policies to manage database size
- Event deduplication to prevent duplicate processing

## Error Handling

The service handles:
- Failed payment processing with automatic retry
- Subscription state validation
- Database transaction failures
- Event emission failures

All errors are logged with full context for debugging.

## Security

- All database operations use parameterized queries (SQLi prevention)
- Subscription data is encrypted at rest (when using PG encryption)
- Event audit trail for compliance
- Access control via merchant_id and customer_id validation
