import React from "react";
import "./ErrorBoundary.css";

interface Props {
  children: React.ReactNode;
  /** Section name shown in the fallback heading */
  section?: string;
  /** Custom fallback UI; overrides the default error UI */
  fallback?: React.ReactNode;
}

interface State {
  error: Error | null;
}

/**
 * Catches runtime errors in its subtree and renders a fallback UI.
 * Shows a stack trace in development; a generic message in production.
 */
export class ErrorBoundary extends React.Component<Props, State> {
  state: State = { error: null };

  static getDerivedStateFromError(error: Error): State {
    return { error };
  }

  componentDidCatch(error: Error, info: React.ErrorInfo) {
    // Log for debugging (replace with a real error-reporting service as needed)
    console.error("[ErrorBoundary]", error, info.componentStack);
  }

  private reset = () => this.setState({ error: null });

  render() {
    const { error } = this.state;
    if (!error) return this.props.children;

    if (this.props.fallback) return this.props.fallback;

    const isDev = process.env.NODE_ENV !== "production";
    const section = this.props.section ?? "This section";

    return (
      <div role="alert" className="eb-container">
        <div className="eb-icon" aria-hidden="true">⚠</div>
        <h2 className="eb-heading">Something went wrong</h2>
        <p className="eb-message">
          {isDev ? error.message : `${section} encountered an unexpected error.`}
        </p>

        {isDev && error.stack && (
          <details className="eb-details">
            <summary>Stack trace</summary>
            <pre className="eb-stack">{error.stack}</pre>
          </details>
        )}

        <div className="eb-actions">
          <button
            type="button"
            className="eb-btn eb-btn--primary"
            onClick={() => window.location.reload()}
          >
            Reload page
          </button>
          <button
            type="button"
            className="eb-btn eb-btn--secondary"
            onClick={this.reset}
          >
            Try again
          </button>
          <a href="/" className="eb-btn eb-btn--ghost">
            Go home
          </a>
        </div>
      </div>
    );
  }
}
