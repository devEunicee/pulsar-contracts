import React from "react";
import { Badge, BadgeStatus } from "../Badge/Badge";

export interface PaymentCardProps {
  /** Unique order/payment ID */
  orderId: string;
  /** Merchant display name */
  merchantName: string;
  /** Payment amount as a formatted string (e.g. "1,000 XLM") */
  amount: string;
  /** ISO 8601 date string */
  date: string;
  /** Current payment status */
  status: BadgeStatus;
  /** Called when the user clicks "View Details" */
  onViewDetails?: () => void;
}

/**
 * Card displaying a summary of a single payment or transaction.
 */
export const PaymentCard: React.FC<PaymentCardProps> = ({
  orderId,
  merchantName,
  amount,
  date,
  status,
  onViewDetails,
}) => (
  <article
    aria-label={`Payment ${orderId}`}
    style={{
      border: "1px solid #e5e7eb",
      borderRadius: "8px",
      padding: "16px",
      maxWidth: "360px",
      fontFamily: "sans-serif",
    }}
  >
    <header
      style={{
        display: "flex",
        justifyContent: "space-between",
        alignItems: "center",
        marginBottom: "8px",
      }}
    >
      <span style={{ fontWeight: 700, fontSize: "0.95rem" }}>{merchantName}</span>
      <Badge status={status} />
    </header>

    <dl style={{ margin: 0, display: "grid", gridTemplateColumns: "auto 1fr", gap: "4px 12px" }}>
      <dt style={{ color: "#6b7280", fontSize: "0.8rem" }}>Order ID</dt>
      <dd style={{ margin: 0, fontSize: "0.8rem", wordBreak: "break-all" }}>{orderId}</dd>

      <dt style={{ color: "#6b7280", fontSize: "0.8rem" }}>Amount</dt>
      <dd style={{ margin: 0, fontWeight: 600 }}>{amount}</dd>

      <dt style={{ color: "#6b7280", fontSize: "0.8rem" }}>Date</dt>
      <dd style={{ margin: 0, fontSize: "0.8rem" }}>{new Date(date).toLocaleString()}</dd>
    </dl>

    {onViewDetails && (
      <button
        onClick={onViewDetails}
        style={{
          marginTop: "12px",
          background: "none",
          border: "none",
          color: "#2563eb",
          cursor: "pointer",
          fontSize: "0.85rem",
          padding: 0,
        }}
      >
        View Details →
      </button>
    )}
  </article>
);
