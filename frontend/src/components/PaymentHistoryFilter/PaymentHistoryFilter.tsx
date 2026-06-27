import React, { useCallback, useEffect, useId, useRef, useState } from "react";
import "./PaymentHistoryFilter.css";

// ── Types ─────────────────────────────────────────────────────────────────────

export type StatusFilter =
  | "Any"
  | "Completed"
  | "PartiallyRefunded"
  | "FullyRefunded";

export type SortField = "Date" | "Amount";
export type SortOrder = "Ascending" | "Descending";

export interface FilterState {
  dateStart: string; // ISO date string "YYYY-MM-DD" or ""
  dateEnd: string;
  amountMin: string; // string to allow empty
  amountMax: string;
  status: StatusFilter;
  search: string; // merchant/payer address search
  sortField: SortField;
  sortOrder: SortOrder;
}

export const DEFAULT_FILTER: FilterState = {
  dateStart: "",
  dateEnd: "",
  amountMin: "",
  amountMax: "",
  status: "Any",
  search: "",
  sortField: "Date",
  sortOrder: "Descending",
};

export interface PaymentHistoryFilterProps {
  value: FilterState;
  onChange: (next: FilterState) => void;
  /** Optional list of known merchant/payer addresses for autocomplete */
  addressSuggestions?: string[];
  className?: string;
}

// ── Component ─────────────────────────────────────────────────────────────────

export function PaymentHistoryFilter({
  value,
  onChange,
  addressSuggestions = [],
  className,
}: PaymentHistoryFilterProps) {
  const id = useId();
  const listboxId = `${id}-suggestions`;
  const [showSuggestions, setShowSuggestions] = useState(false);
  const searchRef = useRef<HTMLInputElement>(null);
  const [activeSuggestion, setActiveSuggestion] = useState(-1);

  const set = useCallback(
    <K extends keyof FilterState>(key: K, val: FilterState[K]) =>
      onChange({ ...value, [key]: val }),
    [onChange, value]
  );

  const filteredSuggestions = addressSuggestions.filter(
    (s) =>
      value.search.length > 0 &&
      s.toLowerCase().includes(value.search.toLowerCase())
  );

  const hasActiveFilters =
    value.dateStart !== DEFAULT_FILTER.dateStart ||
    value.dateEnd !== DEFAULT_FILTER.dateEnd ||
    value.amountMin !== DEFAULT_FILTER.amountMin ||
    value.amountMax !== DEFAULT_FILTER.amountMax ||
    value.status !== DEFAULT_FILTER.status ||
    value.search !== DEFAULT_FILTER.search ||
    value.sortField !== DEFAULT_FILTER.sortField ||
    value.sortOrder !== DEFAULT_FILTER.sortOrder;

  function handleClearAll() {
    onChange({ ...DEFAULT_FILTER });
  }

  function handleSearchKeyDown(e: React.KeyboardEvent<HTMLInputElement>) {
    if (!showSuggestions || filteredSuggestions.length === 0) return;
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setActiveSuggestion((i) => Math.min(i + 1, filteredSuggestions.length - 1));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setActiveSuggestion((i) => Math.max(i - 1, -1));
    } else if (e.key === "Enter" && activeSuggestion >= 0) {
      e.preventDefault();
      set("search", filteredSuggestions[activeSuggestion]);
      setShowSuggestions(false);
      setActiveSuggestion(-1);
    } else if (e.key === "Escape") {
      setShowSuggestions(false);
      setActiveSuggestion(-1);
    }
  }

  // Reset active suggestion when query changes
  useEffect(() => {
    setActiveSuggestion(-1);
  }, [value.search]);

  return (
    <form
      className={`phf${className ? ` ${className}` : ""}`}
      role="search"
      aria-label="Payment history filters"
      onSubmit={(e) => e.preventDefault()}
    >
      {/* ── Row 1: Date range ── */}
      <fieldset className="phf__fieldset">
        <legend className="phf__legend">Date range</legend>
        <div className="phf__row">
          <label className="phf__label" htmlFor={`${id}-date-start`}>
            From
          </label>
          <input
            id={`${id}-date-start`}
            type="date"
            className="phf__input"
            value={value.dateStart}
            max={value.dateEnd || undefined}
            onChange={(e) => set("dateStart", e.target.value)}
            aria-label="Start date"
          />
          <label className="phf__label" htmlFor={`${id}-date-end`}>
            To
          </label>
          <input
            id={`${id}-date-end`}
            type="date"
            className="phf__input"
            value={value.dateEnd}
            min={value.dateStart || undefined}
            onChange={(e) => set("dateEnd", e.target.value)}
            aria-label="End date"
          />
        </div>
      </fieldset>

      {/* ── Row 2: Status ── */}
      <div className="phf__field">
        <label className="phf__label" htmlFor={`${id}-status`}>
          Status
        </label>
        <select
          id={`${id}-status`}
          className="phf__select"
          value={value.status}
          onChange={(e) => set("status", e.target.value as StatusFilter)}
          aria-label="Payment status filter"
        >
          <option value="Any">Any</option>
          <option value="Completed">Completed</option>
          <option value="PartiallyRefunded">Partially refunded</option>
          <option value="FullyRefunded">Fully refunded</option>
        </select>
      </div>

      {/* ── Row 3: Address search with autocomplete ── */}
      <div className="phf__field phf__field--combobox">
        <label className="phf__label" htmlFor={`${id}-search`}>
          Merchant / Payer address
        </label>
        <div className="phf__combobox-wrap">
          <input
            ref={searchRef}
            id={`${id}-search`}
            type="search"
            className="phf__input"
            placeholder="Search by address…"
            value={value.search}
            autoComplete="off"
            role="combobox"
            aria-expanded={showSuggestions && filteredSuggestions.length > 0}
            aria-autocomplete="list"
            aria-controls={listboxId}
            aria-activedescendant={
              activeSuggestion >= 0
                ? `${listboxId}-option-${activeSuggestion}`
                : undefined
            }
            onChange={(e) => {
              set("search", e.target.value);
              setShowSuggestions(true);
            }}
            onFocus={() => setShowSuggestions(true)}
            onBlur={() => setTimeout(() => setShowSuggestions(false), 150)}
            onKeyDown={handleSearchKeyDown}
          />
          {showSuggestions && filteredSuggestions.length > 0 && (
            <ul
              id={listboxId}
              role="listbox"
              aria-label="Address suggestions"
              className="phf__suggestions"
            >
              {filteredSuggestions.map((s, i) => (
                <li
                  key={s}
                  id={`${listboxId}-option-${i}`}
                  role="option"
                  aria-selected={i === activeSuggestion}
                  className={`phf__suggestion${i === activeSuggestion ? " phf__suggestion--active" : ""}`}
                  onMouseDown={() => {
                    set("search", s);
                    setShowSuggestions(false);
                  }}
                >
                  {s}
                </li>
              ))}
            </ul>
          )}
        </div>
      </div>

      {/* ── Row 4: Amount range ── */}
      <fieldset className="phf__fieldset">
        <legend className="phf__legend">Amount range</legend>
        <div className="phf__row">
          <label className="phf__label" htmlFor={`${id}-amount-min`}>
            Min
          </label>
          <input
            id={`${id}-amount-min`}
            type="number"
            className="phf__input phf__input--number"
            placeholder="0"
            value={value.amountMin}
            min={0}
            onChange={(e) => set("amountMin", e.target.value)}
            aria-label="Minimum amount"
          />
          <label className="phf__label" htmlFor={`${id}-amount-max`}>
            Max
          </label>
          <input
            id={`${id}-amount-max`}
            type="number"
            className="phf__input phf__input--number"
            placeholder="∞"
            value={value.amountMax}
            min={value.amountMin || 0}
            onChange={(e) => set("amountMax", e.target.value)}
            aria-label="Maximum amount"
          />
        </div>
      </fieldset>

      {/* ── Row 5: Sort ── */}
      <div className="phf__row phf__row--sort">
        <div className="phf__field">
          <label className="phf__label" htmlFor={`${id}-sort-field`}>
            Sort by
          </label>
          <select
            id={`${id}-sort-field`}
            className="phf__select"
            value={value.sortField}
            onChange={(e) => set("sortField", e.target.value as SortField)}
            aria-label="Sort field"
          >
            <option value="Date">Date</option>
            <option value="Amount">Amount</option>
          </select>
        </div>
        <div className="phf__field">
          <label className="phf__label" htmlFor={`${id}-sort-order`}>
            Order
          </label>
          <select
            id={`${id}-sort-order`}
            className="phf__select"
            value={value.sortOrder}
            onChange={(e) => set("sortOrder", e.target.value as SortOrder)}
            aria-label="Sort order"
          >
            <option value="Ascending">Ascending</option>
            <option value="Descending">Descending</option>
          </select>
        </div>
      </div>

      {/* ── Clear all ── */}
      <div className="phf__actions">
        <button
          type="button"
          className="phf__clear-btn"
          onClick={handleClearAll}
          disabled={!hasActiveFilters}
          aria-label="Clear all filters"
        >
          Clear all filters
        </button>
      </div>
    </form>
  );
}

// ── URL state hook ─────────────────────────────────────────────────────────────

/**
 * Persists filter state in URL search params.
 *
 * Usage:
 *   const [filter, setFilter] = useFilterUrlState();
 *   <PaymentHistoryFilter value={filter} onChange={setFilter} />
 */
export function useFilterUrlState(): [FilterState, (f: FilterState) => void] {
  function fromParams(): FilterState {
    if (typeof window === "undefined") return { ...DEFAULT_FILTER };
    const p = new URLSearchParams(window.location.search);
    return {
      dateStart: p.get("dateStart") ?? DEFAULT_FILTER.dateStart,
      dateEnd: p.get("dateEnd") ?? DEFAULT_FILTER.dateEnd,
      amountMin: p.get("amountMin") ?? DEFAULT_FILTER.amountMin,
      amountMax: p.get("amountMax") ?? DEFAULT_FILTER.amountMax,
      status: (p.get("status") as StatusFilter) ?? DEFAULT_FILTER.status,
      search: p.get("search") ?? DEFAULT_FILTER.search,
      sortField: (p.get("sortField") as SortField) ?? DEFAULT_FILTER.sortField,
      sortOrder:
        (p.get("sortOrder") as SortOrder) ?? DEFAULT_FILTER.sortOrder,
    };
  }

  const [filter, setFilterState] = useState<FilterState>(fromParams);

  const setFilter = useCallback((next: FilterState) => {
    const p = new URLSearchParams();
    (Object.keys(next) as (keyof FilterState)[]).forEach((k) => {
      const v = next[k];
      const def = DEFAULT_FILTER[k];
      if (v !== def) p.set(k, v);
    });
    const search = p.toString();
    const newUrl = search
      ? `${window.location.pathname}?${search}`
      : window.location.pathname;
    window.history.replaceState(null, "", newUrl);
    setFilterState(next);
  }, []);

  return [filter, setFilter];
}
