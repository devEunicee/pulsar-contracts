import React from "react";
import { ErrorBoundary } from "./ErrorBoundary";

/** Full-page error boundary — wraps the entire application. */
export function PageErrorBoundary({ children }: { children: React.ReactNode }) {
  return <ErrorBoundary section="The page">{children}</ErrorBoundary>;
}
