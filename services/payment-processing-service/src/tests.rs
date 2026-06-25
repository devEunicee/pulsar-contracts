#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn validate_amount_accepts_positive_value() {
        assert!(validate_amount(100).is_ok());
    }

    #[test]
    fn validate_amount_rejects_zero_or_negative() {
        assert!(validate_amount(0).is_err());
        assert!(validate_amount(-1).is_err());
    }

    #[test]
    fn validate_order_id_rejects_empty_and_long_values() {
        assert!(validate_order_id("",).is_err());
        assert!(validate_order_id(&"a".repeat(65)).is_err());
    }

    #[test]
    fn create_payment_returns_completed_record() {
        let order = PaymentOrder {
            order_id: "ORDER_001".into(),
            merchant_address: "MERCHANT_001".into(),
            payer_address: "PAYER_001".into(),
            token_address: "TOKEN_001".into(),
            amount: 500,
            description: "Test payment".into(),
            expires_at: Some(Utc::now()),
        };

        let record = create_payment(order.clone(), "signature".into()).unwrap();

        assert_eq!(record.order_id, order.order_id);
        assert_eq!(record.amount, order.amount);
        assert_eq!(record.status, PaymentStatus::Completed);
        assert!(record.completed_at.is_some());
    }
}
