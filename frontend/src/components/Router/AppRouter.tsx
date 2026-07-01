/**
 * Route-based code splitting for the Pulsar frontend.
 *
 * Each top-level route is a React.lazy() dynamic import — Vite/Webpack
 * automatically splits each import() call into a separate JS chunk.
 *
 * Pattern:
 *   1. React.lazy() wraps the import so the chunk only downloads on demand.
 *   2. <React.Suspense> shows a skeleton while the chunk loads.
 *   3. ChunkErrorBoundary catches network errors (e.g. CDN 404 after a deploy).
 *   4. Sibling chunks are prefetched after idle so navigation feels instant.
 *
 * Related: responsive mobile navigation menu (#240).
 */
import React from "react";
import { ThemeToggle } from "../../theme";
import { RouteLoadingSkeleton } from "./RouteLoadingSkeleton";
import { ChunkErrorBoundary } from "./ChunkErrorBoundary";

// ── Lazy route components ────────────────────────────────────────────────────
// Each React.lazy() call becomes its own bundle chunk.
// Named exports are re-wrapped as default exports for React.lazy compatibility.

const PayerDashboardPage = React.lazy(() =>
  import("../PayerDashboard").then((m) => ({ default: m.PayerDashboard }))
);

// Additional routes — add more as the app grows:
// const MerchantDashboardPage = React.lazy(() => import("../pages/MerchantDashboard"));
// const AdminPage             = React.lazy(() => import("../pages/Admin"));

// ── Route definitions ────────────────────────────────────────────────────────

type RoutePath = "/";

interface RouteDef {
  path: RoutePath;
  label: string;
  element: React.ReactNode;
}

const ROUTES: RouteDef[] = [
  {
    path: "/",
    label: "Dashboard",
    element: <PayerDashboardPage />,
  },
];

// ── Prefetch helper ──────────────────────────────────────────────────────────
// Triggers dynamic imports for sibling routes after a short idle delay so
// the browser downloads and caches those chunks in the background.

function prefetchSiblings(currentPath: RoutePath) {
  const sibs = ROUTES.filter((r) => r.path !== currentPath);
  // Touch the imports — if chunks are already cached, this is a no-op.
  sibs.forEach((r) => {
    if (r.path === "/") import("../PayerDashboard");
    // Add additional prefetch cases as new routes are added.
  });
}

// ── Minimal hash router ──────────────────────────────────────────────────────

function useHashRoute(): RoutePath {
  const parse = (hash: string): RoutePath => {
    const p = hash.replace(/^#/, "") || "/";
    return (ROUTES.some((r) => r.path === p) ? p : "/") as RoutePath;
  };

  const [current, setCurrent] = React.useState<RoutePath>(() =>
    parse(window.location.hash)
  );

  React.useEffect(() => {
    const handler = () => setCurrent(parse(window.location.hash));
    window.addEventListener("hashchange", handler);
    return () => window.removeEventListener("hashchange", handler);
  }, []);

  return current;
}

// ── AppRouter ────────────────────────────────────────────────────────────────

export function AppRouter() {
  const currentPath = useHashRoute();

  // Prefetch sibling routes 300ms after navigation (after the current chunk renders)
  React.useEffect(() => {
    const id = setTimeout(() => prefetchSiblings(currentPath), 300);
    return () => clearTimeout(id);
  }, [currentPath]);

  const route = ROUTES.find((r) => r.path === currentPath) ?? ROUTES[0];

  return (
    <>
      <header className="app-header-bar">
        {ROUTES.length > 1 ? (
          <nav aria-label="Main navigation" style={{ display: "flex", gap: "1rem" }}>
            {ROUTES.map((r) => (
              <a
                key={r.path}
                href={`#${r.path}`}
                aria-current={r.path === currentPath ? "page" : undefined}
                style={{
                  fontWeight: r.path === currentPath ? 600 : 400,
                  color: "var(--color-text)",
                  textDecoration: "none",
                }}
              >
                {r.label}
              </a>
            ))}
          </nav>
        ) : (
          <span className="app-header-bar__title">Pulsar</span>
        )}
        <ThemeToggle />
      </header>

      <main>
        <ChunkErrorBoundary>
          <React.Suspense fallback={<RouteLoadingSkeleton />}>
            {route.element}
          </React.Suspense>
        </ChunkErrorBoundary>
      </main>
    </>
  );
}
