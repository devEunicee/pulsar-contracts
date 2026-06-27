import React from "react";
import "./RouteLoadingSkeleton.css";

/**
 * Placeholder shown while a lazy route chunk is being loaded.
 * Renders animated skeleton rows that approximate a page layout.
 */
export function RouteLoadingSkeleton() {
  return (
    <div className="rls-container" role="status" aria-label="Loading page">
      <div className="rls-header skeleton" />
      <div className="rls-subheader skeleton" />
      <div className="rls-row skeleton" />
      <div className="rls-row skeleton" />
      <div className="rls-row skeleton" />
      <div className="rls-row rls-row--short skeleton" />
      <span className="sr-only">Loading…</span>
    </div>
  );
}
