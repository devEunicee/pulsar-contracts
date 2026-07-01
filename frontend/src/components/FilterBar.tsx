import React from "react";
import type { PaymentFilter } from "../contract";

interface Props {
  filter: PaymentFilter;
  sortField: "Date" | "Amount";
  sortOrder: "Ascending" | "Descending";
  onChange: (f: PaymentFilter) => void;
  onSortField: (f: "Date" | "Amount") => void;
  onSortOrder: (o: "Ascending" | "Descending") => void;
}

const fieldStyle: React.CSSProperties = {
  padding: "4px 8px",
  border: "1px solid var(--color-border)",
  borderRadius: 4,
  background: "var(--color-input-bg)",
  color: "var(--color-text)",
  fontSize: "0.875rem",
};

export function FilterBar({ filter, sortField, sortOrder, onChange, onSortField, onSortOrder }: Props) {
  return (
    <div style={{ display: "flex", gap: 12, flexWrap: "wrap", marginBottom: 16, color: "var(--color-text)" }}>
      <label>
        Status&nbsp;
        <select
          value={filter.status ?? "Any"}
          onChange={(e) => onChange({ ...filter, status: e.target.value as PaymentFilter["status"] })}
          style={fieldStyle}
        >
          {["Any", "Completed", "PartiallyRefunded", "FullyRefunded"].map((s) => (
            <option key={s}>{s}</option>
          ))}
        </select>
      </label>

      <label>
        Min amount&nbsp;
        <input
          type="number"
          value={filter.amount_min ?? ""}
          onChange={(e) => onChange({ ...filter, amount_min: e.target.value ? Number(e.target.value) : undefined })}
          style={{ ...fieldStyle, width: 90 }}
        />
      </label>

      <label>
        Max amount&nbsp;
        <input
          type="number"
          value={filter.amount_max ?? ""}
          onChange={(e) => onChange({ ...filter, amount_max: e.target.value ? Number(e.target.value) : undefined })}
          style={{ ...fieldStyle, width: 90 }}
        />
      </label>

      <label>
        Sort by&nbsp;
        <select value={sortField} onChange={(e) => onSortField(e.target.value as "Date" | "Amount")} style={fieldStyle}>
          <option>Date</option>
          <option>Amount</option>
        </select>
      </label>

      <label>
        Order&nbsp;
        <select value={sortOrder} onChange={(e) => onSortOrder(e.target.value as "Ascending" | "Descending")} style={fieldStyle}>
          <option>Descending</option>
          <option>Ascending</option>
        </select>
      </label>
    </div>
  );
}
