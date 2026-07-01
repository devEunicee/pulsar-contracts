import React, { useState, useEffect, useCallback } from "react";
import { useWallet } from "../useWallet";
import { FilterBar } from "./FilterBar";
import { PaymentTable } from "./PaymentTable";
import { fetchPayerHistory } from "../contract";
import type { PaymentRecord, PaymentFilter } from "../contract";

const PAGE_SIZE = 20;

export function PayerDashboard() {
  const { address, error: walletError, connect, disconnect } = useWallet();

  const [records, setRecords] = useState<PaymentRecord[]>([]);
  const [cursorStack, setCursorStack] = useState<(string | null)[]>([null]);
  const [currentPage, setCurrentPage] = useState(0);
  const [nextCursor, setNextCursor] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [fetchError, setFetchError] = useState<string | null>(null);

  const [filter, setFilter] = useState<PaymentFilter>({ status: "Any" });
  const [sortField, setSortField] = useState<"Date" | "Amount">("Date");
  const [sortOrder, setSortOrder] = useState<"Ascending" | "Descending">("Descending");

  const load = useCallback(
    async (cursor: string | null) => {
      if (!address) return;
      setLoading(true);
      setFetchError(null);
      try {
        const page = await fetchPayerHistory(address, cursor, PAGE_SIZE, filter, sortField, sortOrder);
        setRecords(page.records);
        setNextCursor(page.next_cursor);
      } catch (e: any) {
        setFetchError(e?.message ?? "Failed to load payments");
      } finally {
        setLoading(false);
      }
    },
    [address, filter, sortField, sortOrder]
  );

  useEffect(() => {
    setCursorStack([null]);
    setCurrentPage(0);
    load(null);
  }, [load]);

  const goNext = () => {
    if (!nextCursor) return;
    const newStack = [...cursorStack, nextCursor];
    setCursorStack(newStack);
    setCurrentPage(currentPage + 1);
    load(nextCursor);
  };

  const goPrev = () => {
    if (currentPage === 0) return;
    const newStack = cursorStack.slice(0, -1);
    setCursorStack(newStack);
    setCurrentPage(currentPage - 1);
    load(newStack[newStack.length - 1]);
  };

  return (
    <div style={{ maxWidth: 1100, margin: "0 auto", padding: 24, fontFamily: "var(--font-family-sans, system-ui, sans-serif)" }}>
      <h1 style={{ marginBottom: 4, color: "var(--color-text)" }}>Pulsar — Payer Dashboard</h1>

      {!address ? (
        <div>
          <p style={{ color: "var(--color-text-muted)" }}>Connect your Freighter wallet to view your payment history.</p>
          <button onClick={connect} style={btnStyle("var(--color-primary)")}>
            Connect Freighter
          </button>
          {walletError && <p style={{ color: "var(--color-error)" }}>{walletError}</p>}
        </div>
      ) : (
        <>
          <div style={{ display: "flex", alignItems: "center", gap: 12, marginBottom: 20 }}>
            <span
              style={{
                fontFamily: "monospace",
                background: "var(--color-surface)",
                color: "var(--color-text)",
                padding: "4px 10px",
                borderRadius: 6,
                border: "1px solid var(--color-border)",
              }}
            >
              {address.slice(0, 8)}…{address.slice(-4)}
            </span>
            <button onClick={disconnect} style={btnStyle("var(--color-text-muted)")}>
              Disconnect
            </button>
          </div>

          <FilterBar
            filter={filter}
            sortField={sortField}
            sortOrder={sortOrder}
            onChange={setFilter}
            onSortField={setSortField}
            onSortOrder={setSortOrder}
          />

          {fetchError && <p style={{ color: "var(--color-error)" }}>{fetchError}</p>}

          <PaymentTable records={records} loading={loading} />

          <div style={{ display: "flex", gap: 12, marginTop: 16, alignItems: "center" }}>
            <button onClick={goPrev} disabled={currentPage === 0} style={btnStyle("var(--color-text-muted)")}>
              ← Previous
            </button>
            <span style={{ color: "var(--color-text-muted)" }}>Page {currentPage + 1}</span>
            <button onClick={goNext} disabled={!nextCursor} style={btnStyle("var(--color-primary)")}>
              Next →
            </button>
          </div>
        </>
      )}
    </div>
  );
}

function btnStyle(bg: string): React.CSSProperties {
  return {
    background: bg,
    color: "var(--color-text-inverse, #fff)",
    border: "none",
    borderRadius: 6,
    padding: "8px 16px",
    cursor: "pointer",
    fontWeight: 600,
  };
}
