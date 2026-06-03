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

  // Reload from page 0 whenever filters/sort change
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
    <div style={{ maxWidth: 1100, margin: "0 auto", padding: 24, fontFamily: "system-ui, sans-serif" }}>
      <h1 style={{ marginBottom: 4 }}>Pulsar — Payer Dashboard</h1>

      {!address ? (
        <div>
          <p style={{ color: "#64748b" }}>Connect your Freighter wallet to view your payment history.</p>
          <button onClick={connect} style={btnStyle("#6366f1")}>
            Connect Freighter
          </button>
          {walletError && <p style={{ color: "#ef4444" }}>{walletError}</p>}
        </div>
      ) : (
        <>
          <div style={{ display: "flex", alignItems: "center", gap: 12, marginBottom: 20 }}>
            <span style={{ fontFamily: "monospace", background: "#f1f5f9", padding: "4px 10px", borderRadius: 6 }}>
              {address.slice(0, 8)}…{address.slice(-4)}
            </span>
            <button onClick={disconnect} style={btnStyle("#94a3b8")}>
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

          {fetchError && <p style={{ color: "#ef4444" }}>{fetchError}</p>}

          <PaymentTable records={records} loading={loading} />

          <div style={{ display: "flex", gap: 12, marginTop: 16, alignItems: "center" }}>
            <button onClick={goPrev} disabled={currentPage === 0} style={btnStyle("#64748b")}>
              ← Previous
            </button>
            <span style={{ color: "#64748b" }}>Page {currentPage + 1}</span>
            <button onClick={goNext} disabled={!nextCursor} style={btnStyle("#6366f1")}>
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
    color: "#fff",
    border: "none",
    borderRadius: 6,
    padding: "8px 16px",
    cursor: "pointer",
    fontWeight: 600,
  };
}
