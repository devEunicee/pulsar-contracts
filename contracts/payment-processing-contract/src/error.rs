use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum PaymentError {
    // Auth
    Unauthorized = 1,
    AdminAlreadySet = 2,

    // Merchant
    MerchantNotFound = 10,
    MerchantAlreadyRegistered = 11,
    MerchantInactive = 12,

    // Payment
    PaymentNotFound = 20,
    PaymentAlreadyExists = 21,
    InvalidAmount = 22,
    InvalidSignature = 23,
    PaymentExpired = 24,
    // InsufficientBalance removed: token transfers from SDK panic on insufficient funds

    // Refund
    RefundNotFound = 30,
    RefundAlreadyExists = 31,
    RefundWindowExpired = 32,
    RefundAmountExceedsPayment = 33,
    RefundNotApproved = 34,
    RefundAlreadyCompleted = 35,

    // Multisig
    MultisigNotFound = 40,
    MultisigAlreadySigned = 41,
    MultisigAlreadyExecuted = 42,
    InsufficientSignatures = 43,

    // General
    InvalidInput = 50,
    StorageError = 51,
    ArithmeticError = 52,

    // SEC-009: Token allowlist
    TokenNotAllowed = 60,
}
