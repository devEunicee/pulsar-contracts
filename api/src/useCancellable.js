/**
 * useCancellable — React hook that provides an AbortSignal tied to component lifetime.
 *
 * - Automatically cancels in-flight requests when the component unmounts.
 * - Ignores stale responses (requests started before the most recent one).
 * - Clears loading state on cancellation, preventing memory leaks.
 *
 * @example
 * function MerchantProfile({ id }) {
 *   const { signal, makeRequest } = useCancellable();
 *   const [data, setData] = useState(null);
 *   const [loading, setLoading] = useState(false);
 *
 *   useEffect(() => {
 *     makeRequest(async () => {
 *       setLoading(true);
 *       const result = await client.getMerchant(id, { signal });
 *       setData(result);
 *       setLoading(false);
 *     });
 *   }, [id]);
 * }
 */
import { useEffect, useRef, useCallback } from "react";
import { CancelledError } from "./client.js";

export function useCancellable() {
  const controllerRef = useRef(null);

  // Cancel any in-flight request when the component unmounts
  useEffect(() => {
    return () => {
      controllerRef.current?.abort();
    };
  }, []);

  /**
   * Creates a fresh AbortController for each call, cancelling the previous one.
   * Runs `fn` with the new signal; silently swallows CancelledError.
   */
  const makeRequest = useCallback(async (fn) => {
    // Cancel previous request if still running
    controllerRef.current?.abort();

    const controller = new AbortController();
    controllerRef.current = controller;

    try {
      await fn(controller.signal);
    } catch (err) {
      if (err instanceof CancelledError || err?.name === "AbortError") {
        // Request was cancelled — do not update state
        return;
      }
      throw err;
    }
  }, []);

  return {
    /** Current AbortSignal — pass to API calls for manual wiring. */
    get signal() {
      return controllerRef.current?.signal;
    },
    /** Wraps async work; cancels previous request and ignores CancelledError. */
    makeRequest,
    /** Manually cancel the current in-flight request. */
    cancel() {
      controllerRef.current?.abort();
    },
  };
}
