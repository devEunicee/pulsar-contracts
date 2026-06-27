import React from "react";
import "./Pagination.css";

export interface PaginationProps {
  /** Cursor for the current page (null = first page) */
  currentCursor: string | null;
  /** Cursor to pass when going to next page; null means no next page */
  nextCursor: string | null;
  /** Stack of previous cursors (managed externally or via usePagination) */
  prevCursors: (string | null)[];
  onNext: (cursor: string) => void;
  onPrev: () => void;
  onFirst: () => void;
  /** 1-based page number indicator */
  pageNumber: number;
  /** Total number of items on this page (for screen reader hint) */
  pageSize?: number;
  className?: string;
}

export function Pagination({
  nextCursor,
  prevCursors,
  onNext,
  onPrev,
  onFirst,
  pageNumber,
  pageSize,
  className,
}: PaginationProps) {
  const hasPrev = pageNumber > 1;
  const hasNext = nextCursor != null;

  return (
    <nav
      aria-label="Pagination"
      className={`pagination${className ? ` ${className}` : ""}`}
    >
      <button
        type="button"
        className="pagination__btn"
        onClick={onFirst}
        disabled={!hasPrev}
        aria-label="Go to first page"
        aria-disabled={!hasPrev}
      >
        «
      </button>

      <button
        type="button"
        className="pagination__btn"
        onClick={onPrev}
        disabled={!hasPrev}
        aria-label="Go to previous page"
        aria-disabled={!hasPrev}
      >
        ‹ Prev
      </button>

      <span
        className="pagination__indicator"
        aria-live="polite"
        aria-atomic="true"
        aria-label={`Page ${pageNumber}${pageSize != null ? `, ${pageSize} items` : ""}`}
      >
        Page {pageNumber}
      </span>

      <button
        type="button"
        className="pagination__btn"
        onClick={() => hasNext && onNext(nextCursor!)}
        disabled={!hasNext}
        aria-label="Go to next page"
        aria-disabled={!hasNext}
      >
        Next ›
      </button>
    </nav>
  );
}

// ── Hook ──────────────────────────────────────────────────────────────────────

export interface PaginationState {
  cursor: string | null;
  prevCursors: (string | null)[];
  pageNumber: number;
}

export interface PaginationActions {
  goNext: (nextCursor: string) => void;
  goPrev: () => void;
  goFirst: () => void;
  reset: () => void;
}

/**
 * Manages cursor-based pagination state.
 *
 * Usage:
 *   const [state, actions] = usePagination();
 *   // Pass state.cursor to your data-fetch call.
 *   // Pass nextCursor from the API response to actions.goNext.
 */
export function usePagination(): [PaginationState, PaginationActions] {
  const [state, setState] = React.useState<PaginationState>({
    cursor: null,
    prevCursors: [],
    pageNumber: 1,
  });

  const goNext = React.useCallback((nextCursor: string) => {
    setState((prev) => ({
      cursor: nextCursor,
      prevCursors: [...prev.prevCursors, prev.cursor],
      pageNumber: prev.pageNumber + 1,
    }));
  }, []);

  const goPrev = React.useCallback(() => {
    setState((prev) => {
      if (prev.prevCursors.length === 0) return prev;
      const newPrevCursors = prev.prevCursors.slice(0, -1);
      return {
        cursor: prev.prevCursors[prev.prevCursors.length - 1],
        prevCursors: newPrevCursors,
        pageNumber: prev.pageNumber - 1,
      };
    });
  }, []);

  const goFirst = React.useCallback(() => {
    setState({ cursor: null, prevCursors: [], pageNumber: 1 });
  }, []);

  const reset = goFirst;

  return [state, { goNext, goPrev, goFirst, reset }];
}
