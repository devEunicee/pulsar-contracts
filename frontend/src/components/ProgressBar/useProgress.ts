import React from "react";

interface UseProgressReturn {
  /** Current progress value (0–100), or null when hidden */
  progress: number | null;
  /** Call before an async operation to start the bar */
  start: () => void;
  /** Advance progress by a fixed increment (default 10) */
  increment: (by?: number) => void;
  /** Jump to a specific value (0–100) */
  set: (value: number) => void;
  /** Complete the bar and hide it after a short delay */
  done: () => void;
}

/**
 * Manages page-level loading progress state.
 *
 * ```tsx
 * const { progress, start, increment, done } = useProgress();
 *
 * const fetchData = async () => {
 *   start();
 *   const result = await api.getPayments();
 *   done();
 *   return result;
 * };
 *
 * return <ProgressBar progress={progress} />;
 * ```
 */
export function useProgress(): UseProgressReturn {
  const [progress, setProgress] = React.useState<number | null>(null);
  const timerRef = React.useRef<ReturnType<typeof setTimeout> | null>(null);

  const clearTimer = () => {
    if (timerRef.current !== null) {
      clearTimeout(timerRef.current);
      timerRef.current = null;
    }
  };

  const start = React.useCallback(() => {
    clearTimer();
    setProgress(10);
  }, []);

  const increment = React.useCallback((by = 10) => {
    setProgress((prev) => {
      if (prev === null) return null;
      return Math.min(prev + by, 90); // never auto-reach 100
    });
  }, []);

  const set = React.useCallback((value: number) => {
    setProgress(Math.max(0, Math.min(100, value)));
  }, []);

  const done = React.useCallback(() => {
    clearTimer();
    setProgress(100);
    timerRef.current = setTimeout(() => setProgress(null), 400);
  }, []);

  // Cleanup on unmount
  React.useEffect(() => () => clearTimer(), []);

  return { progress, start, increment, set, done };
}
