import React, { useCallback, useId, useRef, useState } from "react";
import "./DataTable.css";

export type SortDirection = "asc" | "desc" | "none";

export interface ColumnDef<T> {
  key: string;
  header: string;
  sortable?: boolean;
  render?: (row: T, rowIndex: number) => React.ReactNode;
  /** Primitive value used for sorting when render is a custom element */
  getValue?: (row: T) => string | number;
}

export interface DataTableProps<T> {
  columns: ColumnDef<T>[];
  rows: T[];
  caption?: string;
  /** Unique key per row for stable selection/focus */
  getRowKey: (row: T) => string;
  selectable?: boolean;
  selectedKeys?: Set<string>;
  onSelectionChange?: (keys: Set<string>) => void;
  onSortChange?: (columnKey: string, direction: SortDirection) => void;
  /** Controlled sort state */
  sortColumn?: string;
  sortDirection?: SortDirection;
  className?: string;
  "aria-label"?: string;
}

function nextDirection(current: SortDirection): SortDirection {
  return current === "none" || current === "desc" ? "asc" : "desc";
}

export function DataTable<T>({
  columns,
  rows,
  caption,
  getRowKey,
  selectable = false,
  selectedKeys = new Set(),
  onSelectionChange,
  onSortChange,
  sortColumn,
  sortDirection = "none",
  className,
  "aria-label": ariaLabel,
}: DataTableProps<T>) {
  const tableId = useId();
  const [focusedCell, setFocusedCell] = useState<{
    row: number;
    col: number;
  } | null>(null);
  const cellRefs = useRef<Map<string, HTMLElement>>(new Map());

  const totalCols = selectable ? columns.length + 1 : columns.length;

  function focusCell(row: number, col: number) {
    const key = `${row}-${col}`;
    const el = cellRefs.current.get(key);
    el?.focus();
    setFocusedCell({ row, col });
  }

  function handleCellKeyDown(
    e: React.KeyboardEvent,
    rowIndex: number,
    colIndex: number
  ) {
    const maxRow = rows.length - 1;
    const maxCol = totalCols - 1;
    const moves: Record<string, [number, number]> = {
      ArrowUp: [rowIndex - 1, colIndex],
      ArrowDown: [rowIndex + 1, colIndex],
      ArrowLeft: [rowIndex, colIndex - 1],
      ArrowRight: [rowIndex, colIndex + 1],
      Home: [rowIndex, 0],
      End: [rowIndex, maxCol],
    };
    const next = moves[e.key];
    if (next) {
      const [nr, nc] = next;
      if (nr >= 0 && nr <= maxRow && nc >= 0 && nc <= maxCol) {
        e.preventDefault();
        focusCell(nr, nc);
      }
    }
    if (e.key === "Enter" || e.key === " ") {
      if (selectable && colIndex === 0) {
        e.preventDefault();
        toggleRow(getRowKey(rows[rowIndex]));
      }
    }
  }

  function handleHeaderKeyDown(e: React.KeyboardEvent, col: ColumnDef<T>) {
    if ((e.key === "Enter" || e.key === " ") && col.sortable) {
      e.preventDefault();
      handleSort(col.key);
    }
  }

  function handleSort(key: string) {
    if (!onSortChange) return;
    const current = sortColumn === key ? sortDirection : "none";
    onSortChange(key, nextDirection(current));
  }

  function toggleRow(key: string) {
    if (!onSelectionChange) return;
    const next = new Set(selectedKeys);
    next.has(key) ? next.delete(key) : next.add(key);
    onSelectionChange(next);
  }

  function toggleAll() {
    if (!onSelectionChange) return;
    const allKeys = rows.map(getRowKey);
    const allSelected = allKeys.every((k) => selectedKeys.has(k));
    onSelectionChange(allSelected ? new Set() : new Set(allKeys));
  }

  const allSelected =
    rows.length > 0 && rows.every((r) => selectedKeys.has(getRowKey(r)));
  const someSelected = !allSelected && rows.some((r) => selectedKeys.has(getRowKey(r)));

  const setCellRef = useCallback(
    (el: HTMLElement | null, row: number, col: number) => {
      const key = `${row}-${col}`;
      if (el) cellRefs.current.set(key, el);
      else cellRefs.current.delete(key);
    },
    []
  );

  return (
    <div className={`data-table-wrapper${className ? ` ${className}` : ""}`}>
      <table
        id={tableId}
        role="grid"
        aria-label={ariaLabel}
        aria-rowcount={rows.length}
        aria-colcount={totalCols}
        className="data-table"
      >
        {caption && <caption>{caption}</caption>}
        <thead>
          <tr role="row">
            {selectable && (
              <th scope="col" className="data-table__checkbox-col">
                <input
                  type="checkbox"
                  aria-label="Select all rows"
                  checked={allSelected}
                  ref={(el) => {
                    if (el) el.indeterminate = someSelected;
                  }}
                  onChange={toggleAll}
                />
              </th>
            )}
            {columns.map((col) => {
              const isSorted = sortColumn === col.key;
              const dir = isSorted ? sortDirection : "none";
              return (
                <th
                  key={col.key}
                  scope="col"
                  aria-sort={
                    col.sortable
                      ? dir === "asc"
                        ? "ascending"
                        : dir === "desc"
                        ? "descending"
                        : "none"
                      : undefined
                  }
                  className={`data-table__th${col.sortable ? " data-table__th--sortable" : ""}${isSorted ? " data-table__th--sorted" : ""}`}
                  tabIndex={col.sortable ? 0 : undefined}
                  onClick={col.sortable ? () => handleSort(col.key) : undefined}
                  onKeyDown={(e) => handleHeaderKeyDown(e, col)}
                >
                  {col.header}
                  {col.sortable && (
                    <span
                      className="data-table__sort-icon"
                      aria-hidden="true"
                    >
                      {dir === "asc" ? " ↑" : dir === "desc" ? " ↓" : " ↕"}
                    </span>
                  )}
                </th>
              );
            })}
          </tr>
        </thead>
        <tbody>
          {rows.map((row, rowIndex) => {
            const rowKey = getRowKey(row);
            const isSelected = selectedKeys.has(rowKey);
            return (
              <tr
                key={rowKey}
                role="row"
                aria-rowindex={rowIndex + 1}
                aria-selected={selectable ? isSelected : undefined}
                className={`data-table__row${isSelected ? " data-table__row--selected" : ""}`}
              >
                {selectable && (
                  <td
                    role="gridcell"
                    className="data-table__checkbox-col"
                    tabIndex={
                      focusedCell?.row === rowIndex && focusedCell.col === 0
                        ? 0
                        : -1
                    }
                    ref={(el) => setCellRef(el, rowIndex, 0)}
                    onKeyDown={(e) => handleCellKeyDown(e, rowIndex, 0)}
                  >
                    <input
                      type="checkbox"
                      aria-label={`Select row ${rowIndex + 1}`}
                      checked={isSelected}
                      onChange={() => toggleRow(rowKey)}
                      tabIndex={-1}
                    />
                  </td>
                )}
                {columns.map((col, colIndex) => {
                  const adjustedCol = selectable ? colIndex + 1 : colIndex;
                  const value = col.render
                    ? col.render(row, rowIndex)
                    : col.getValue
                    ? col.getValue(row)
                    : (row as Record<string, unknown>)[col.key];
                  return (
                    <td
                      key={col.key}
                      role="gridcell"
                      aria-colindex={adjustedCol + 1}
                      tabIndex={
                        focusedCell?.row === rowIndex &&
                        focusedCell.col === adjustedCol
                          ? 0
                          : -1
                      }
                      ref={(el) => setCellRef(el, rowIndex, adjustedCol)}
                      onKeyDown={(e) =>
                        handleCellKeyDown(e, rowIndex, adjustedCol)
                      }
                      onFocus={() =>
                        setFocusedCell({ row: rowIndex, col: adjustedCol })
                      }
                      className="data-table__td"
                    >
                      {value as React.ReactNode}
                    </td>
                  );
                })}
              </tr>
            );
          })}
          {rows.length === 0 && (
            <tr role="row">
              <td
                colSpan={totalCols}
                className="data-table__empty"
                role="gridcell"
              >
                No data available.
              </td>
            </tr>
          )}
        </tbody>
      </table>
    </div>
  );
}
