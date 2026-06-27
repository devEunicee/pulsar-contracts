import React from "react";

export type BadgeStatus =
  | "pending"
  | "approved"
  | "rejected"
  | "completed"
  | "cancelled";

export interface BadgeProps {
  /** Payment or refund status to display */
  status: BadgeStatus;
  /** Optional custom label; defaults to capitalised status */
  label?: string;
}

const statusColors: Record<BadgeStatus, { bg: string; text: string }> = {
  pending: { bg: "#fef9c3", text: "#854d0e" },
  approved: { bg: "#dcfce7", text: "#166534" },
  rejected: { bg: "#fee2e2", text: "#991b1b" },
  completed: { bg: "#dbeafe", text: "#1e40af" },
  cancelled: { bg: "#f3f4f6", text: "#6b7280" },
};

/**
 * Status badge for payments, refunds, and transactions.
 */
export const Badge: React.FC<BadgeProps> = ({ status, label }) => {
  const { bg, text } = statusColors[status];
  return (
    <span
      role="status"
      aria-label={`Status: ${label ?? status}`}
      style={{
        display: "inline-block",
        padding: "2px 10px",
        borderRadius: "9999px",
        fontSize: "0.75rem",
        fontWeight: 600,
        backgroundColor: bg,
        color: text,
        textTransform: "capitalize",
      }}
    >
      {label ?? status}
    </span>
  );
};
