/** Maps Pulsar contract error codes (BE-006) to human-readable messages. */
export const CONTRACT_ERRORS: Record<number, string> = {
  1: "Unauthorized: you don't have permission to perform this action.",
  2: "Admin already set.",
  10: "Merchant not found.",
  11: "Merchant already registered.",
  12: "Merchant is deactivated.",
  20: "Payment not found.",
  21: "Payment already exists.",
  22: "Invalid amount.",
  23: "Invalid signature.",
  24: "Payment has expired.",
  25: "Insufficient merchant balance for refund.",
  30: "Refund not found.",
  31: "Refund already exists.",
  32: "Refund window has expired (30-day limit).",
  33: "Refund amount exceeds original payment.",
  34: "Refund has not been approved.",
  35: "Refund already completed or rejected.",
  40: "Multi-sig payment not found.",
  41: "Signer has already signed.",
  42: "Multi-sig payment already executed.",
  43: "Not all required signers have signed.",
  50: "Invalid input.",
};

export function getErrorMessage(err: unknown): string {
  if (err instanceof Error) {
    const match = err.message.match(/Error\(Contract, #(\d+)\)/);
    if (match) {
      const code = parseInt(match[1], 10);
      return CONTRACT_ERRORS[code] ?? `Contract error #${code}.`;
    }
    return err.message;
  }
  return "An unexpected error occurred.";
}
