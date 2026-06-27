import React from "react";
import { ErrorBoundary } from "./ErrorBoundary";

interface Props {
  section: string;
  children: React.ReactNode;
}

/**
 * Scoped error boundary for individual page sections (e.g. Payment History,
 * Merchant Panel). Prevents one broken section from crashing the whole page.
 */
export function SectionErrorBoundary({ section, children }: Props) {
  return <ErrorBoundary section={section}>{children}</ErrorBoundary>;
}
