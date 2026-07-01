import React from "react";
import type { PaymentRecord } from "../contract";

interface Props {
  records: PaymentRecord[];
  loading: boolean;
}

const STATUS_COLOR: Record<string, string> = {
  Completed: "var(--color-success)",
  PartiallyRefunded: "var(--color-warning)",
  FullyRefunded: "var(--color-info)",
};

export function PaymentTable({ records, loading }: Props) {
  if (loading) return <p style={{ color: "var(--color-text-muted)" }}>Loading…</p>;
  if (!records.length) return <p style={{ color: "var(--color-text-muted)" }}>No payments found.</p>;

  return (
    <table style={{ width: "100%", borderCollapse: "collapse", fontSize: 14, color: "var(--color-text)" }}>
      <thead>
        <tr style={{ background: "var(--color-surface)" }}>
          {["Order ID", "Merchant", "Token", "Amount", "Date", "Status"].map((h) => (
            <th
              key={h}
              style={{
                padding: "8px 12px",
                textAlign: "left",
                borderBottom: "1px solid var(--color-border)",
                color: "var(--color-text)",
              }}
            >
              {h}
            </th>
          ))}
        </tr>
      </thead>
      <tbody>
        {records.map((r) => (
          <tr key={r.order_id} style={{ borderBottom: "1px solid var(--color-border)" }}>
            <td style={{ padding: "8px 12px", fontFamily: "monospace" }}>{r.order_id}</td>
            <td style={{ padding: "8px 12px", fontFamily: "monospace" }}>
              {r.merchant_address.slice(0, 8)}…{r.merchant_address.slice(-4)}
            </td>
            <td style={{ padding: "8px 12px", fontFamily: "monospace" }}>
              {r.token.slice(0, 8)}…{r.token.slice(-4)}
            </td>
            <td style={{ padding: "8px 12px" }}>{r.amount}</td>
            <td style={{ padding: "8px 12px" }}>
              {new Date(r.paid_at * 1000).toLocaleString()}
            </td>
            <td style={{ padding: "8px 12px" }}>
              <span style={{ color: STATUS_COLOR[r.status] ?? "var(--color-text-muted)", fontWeight: 600 }}>
                {r.status}
              </span>
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
