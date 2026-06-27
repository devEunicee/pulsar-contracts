import React from "react";

interface State { failed: boolean }

/**
 * Catches errors thrown when a lazy-loaded chunk fails to load
 * (e.g. network error, 404 after a deploy). Offers a reload button.
 */
export class ChunkErrorBoundary extends React.Component<
  { children: React.ReactNode },
  State
> {
  state: State = { failed: false };

  static getDerivedStateFromError(): State {
    return { failed: true };
  }

  componentDidCatch(error: Error) {
    console.error("[ChunkErrorBoundary] Failed to load route chunk:", error);
  }

  render() {
    if (!this.state.failed) return this.props.children;

    return (
      <div
        role="alert"
        style={{
          padding: "2rem 1.5rem",
          textAlign: "center",
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          gap: "0.75rem",
        }}
      >
        <p style={{ margin: 0 }}>Failed to load this page. Check your connection and try again.</p>
        <button
          type="button"
          onClick={() => window.location.reload()}
          style={{ padding: "0.4rem 1rem", cursor: "pointer" }}
        >
          Reload
        </button>
      </div>
    );
  }
}
