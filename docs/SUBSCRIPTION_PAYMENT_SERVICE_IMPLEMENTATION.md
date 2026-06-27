# Subscription Payment Service Implementation

**Issue:** #280  
**Date:** 2026-06-25  
**Purpose:** Implement service for managing recurring subscription payments

## Overview

The subscription payment service provides comprehensive recurring payment management with automatic processing, retries, invoicing, and full event tracking.

## Architecture

### Components

1. **PaymentProcessor**: Handles subscription and payment management
2. **EventEmitter**: Emits events for subscription lifecycle
3. **InvoiceGenerator**: Creates and tracks invoices
4. **SubscriptionScheduler**: Schedules automated payment processing
5. **Configuration**: Flexible, environment-based configuration

### Service Flow

```
1. Create Subscription → Stores in DB
2. Scheduled Payment Processing → Checks due payments
3. Payment Attempt → Records attempt
4. Success/Failure → Updates status
5. Invoice Generation → Creates invoice
6. Event Emission → Notifies subscribers
7. Retry Logic → Exponential backoff on failure
```

## Key Features Implemented

### Subscription Creation and Configuration
- ✅ Create subscriptions with merchant, customer, amount, currency, frequency
- ✅ Flexible billing frequencies (daily, weekly, bi-weekly, monthly, quarterly, annually)
- ✅ Metadata support for custom data
- ✅ Next payment date calculated based on frequency

### Recurring Payment Scheduling
- ✅ Automatic scheduled payment processing
- ✅ Configurable processing interval (default: hourly)
- ✅ Due payment detection and batch processing
- ✅ Support for multiple concurrent subscriptions

### Subscription Pausing/Resumption
- ✅ Pause active subscriptions
- ✅ Resume paused subscriptions
- ✅ Track pause start date
- ✅ Prevent duplicate pauses

### Automatic Retry on Failure
- ✅ Exponential backoff retry strategy
- ✅ Configurable maximum retries (default: 3)
- ✅ Configurable retry delay (default: 5 min, exponential)
- ✅ Error message tracking
- ✅ Automatic status updates

### Payment Attempt History
- ✅ Record all payment attempts
- ✅ Track attempt number
- ✅ Store error messages
- ✅ Record attempt timestamps
- ✅ Query payment history

### Subscription Cancellation
- ✅ Cancel active subscriptions
- ✅ Store cancellation reason
- ✅ Record cancellation timestamp
- ✅ Update status to cancelled
- ✅ Prevent double cancellation

### Invoice Generation
- ✅ Automatically generate invoices for payments
- ✅ Unique invoice numbers
- ✅ Track invoice status (issued, paid, etc.)
- ✅ Store due dates
- ✅ Line item support
- ✅ Mark invoices as paid

### Event Emission on Payment Attempts
- ✅ Emit events on key lifecycle events
- ✅ Include event data (amount, status, etc.)
- ✅ Full audit trail of events
- ✅ Event type classification
- ✅ Timestamp tracking

## Database Schema

### Tables

**subscriptions**
- Stores subscription data
- Tracks status, payment schedule, metadata
- Foreign keys to merchants and customers

**payment_attempts**
- Records every payment attempt
- Tracks status, error messages, attempt numbers
- References subscriptions

**invoices**
- Stores generated invoices
- Tracks status, payment dates
- References subscriptions

**subscription_events**
- Full audit trail of events
- Event type and data storage
- Ordered by creation time

### Key Constraints

- NOT NULL for required fields (merchant_id, customer_id, amount)
- CHECK constraints for valid frequencies and statuses
- CHECK constraints for positive amounts
- Foreign key constraints for referential integrity
- ON DELETE CASCADE for related records

### Indexes

- merchant_id and customer_id for customer-based queries
- status for filtering by subscription state
- next_payment_at for finding due payments
- created_at for time-range queries
- subscription_id on payment_attempts and invoices

## Configuration

### Environment Variables

```bash
# Database
DATABASE_URL=postgresql://localhost/subscriptions

# Payment Processing
MAX_PAYMENT_RETRIES=3
RETRY_DELAY_SECS=300

# Features
ENABLE_AUTO_PAYMENTS=true
PAYMENT_INTERVAL_SECS=3600
ENABLE_INVOICE_GENERATION=true
ENABLE_EVENTS=true

# Data Retention
PAYMENT_HISTORY_RETENTION_DAYS=365
LATE_PAYMENT_GRACE_HOURS=24
```

### Default Configuration

```rust
SubscriptionConfig {
    database_url: "postgresql://localhost/subscriptions",
    max_payment_retries: 3,
    retry_delay_secs: 300,
    enable_auto_payments: true,
    payment_interval_secs: 3600,
    enable_invoice_generation: true,
    enable_events: true,
    payment_history_retention_days: 365,
    late_payment_grace_hours: 24,
}
```

## Usage Examples

### Create Subscription
```rust
let subscription = Subscription::new(
    "merchant_123".to_string(),
    "customer_456".to_string(),
    "99.99".to_string(),
    "USD".to_string(),
    BillingFrequency::Monthly,
);

let sub_id = service.create_subscription(subscription).await?;
println!("Created subscription: {}", sub_id);
```

### Manage Subscription
```rust
// Get subscription
let sub = service.get_subscription(&sub_id).await?.unwrap();
println!("Status: {}, Next payment: {}", sub.status, sub.next_payment_at);

// Pause
service.pause_subscription(&sub_id).await?;

// Resume
service.resume_subscription(&sub_id).await?;

// Cancel
service.cancel_subscription(&sub_id, "Customer request").await?;
```

### Track Payments
```rust
let attempts = service.get_payment_attempts(&sub_id).await?;
for attempt in attempts {
    println!(
        "Attempt {}: {} at {}",
        attempt.attempt_number,
        attempt.status,
        attempt.attempted_at
    );
}
```

### Generate Invoice
```rust
let invoice = service.generate_invoice(&sub_id).await?;
println!(
    "Invoice {}: {} {} (due: {})",
    invoice.invoice_number,
    invoice.amount,
    invoice.currency,
    invoice.due_date
);
```

### Track Events
```rust
let events = service.emitter().get_events(&sub_id).await?;
for event in events {
    println!("{}: {} at {}", event.event_type, event.data, event.created_at);
}
```

## Payment Processing Flow

1. **Scheduler runs** at configured interval (default: hourly)
2. **Find due payments**: Query subscriptions where next_payment_at <= NOW()
3. **Process payment**: For each due subscription:
   - Record payment attempt
   - Emit "payment_attempted" event
   - Update subscription status
   - Generate invoice (if enabled)
   - Emit payment success/failure event
4. **On failure**:
   - Record error message
   - Check retry count
   - If retries available: set retry time, emit "retrying" event
   - If max retries exceeded: set status to "past_due", emit "failed" event
5. **Update metrics** and audit trail

## Event Types

- `subscription_created`: Subscription created
- `subscription_paused`: Subscription paused
- `subscription_resumed`: Subscription resumed
- `subscription_cancelled`: Subscription cancelled
- `payment_attempted`: Payment attempt started
- `payment_succeeded`: Payment succeeded
- `payment_failed`: Payment failed
- `invoice_generated`: Invoice created
- `invoice_sent`: Invoice sent to customer
- `invoice_paid`: Invoice payment received

## Retry Strategy

Exponential backoff with configurable base delay:

```
Attempt 1: Immediate
Attempt 2: delay * 2^1 (10 min with 5-min base)
Attempt 3: delay * 2^2 (20 min with 5-min base)
Attempt 4+: Failed, move to past_due
```

## Monitoring and Metrics

### Dashboard View
```sql
SELECT * FROM subscription_status_summary;
```

### Revenue Metrics
```sql
SELECT * FROM get_subscription_metrics(30);
```

### Payment Performance
```sql
SELECT 
  status,
  COUNT(*) as attempts,
  ROUND(100.0 * COUNT(*) / SUM(COUNT(*)) OVER (), 2) as percentage
FROM payment_attempts
WHERE attempted_at > NOW() - INTERVAL '30 days'
GROUP BY status;
```

## Acceptance Criteria Met

✅ Subscription creation and configuration  
✅ Recurring payment scheduling  
✅ Subscription pausing/resumption  
✅ Automatic retry on failure  
✅ Payment attempt history  
✅ Subscription cancellation  
✅ Invoice generation  
✅ Event emission on payment attempts  

## Next Steps

1. Integrate with payment processor for actual payment processing
2. Implement webhook notifications
3. Add admin dashboard for subscription management
4. Implement subscription analytics
5. Add customer self-service portal
6. Deploy and monitor in production

## Performance Considerations

- Batch processing of due payments
- Indexed queries for fast lookups
- Event deduplication
- Configurable retention policies
- Automated cleanup of old data

## Error Handling

- Transactional operations for data consistency
- Detailed error messages for debugging
- Automatic retry with exponential backoff
- Event logging for audit trail
- Graceful degradation for failed events
