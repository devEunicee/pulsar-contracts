import { useState, useCallback, useRef } from 'react';

interface AsyncState<T> {
  data: T | null;
  error: Error | null;
  isLoading: boolean;
}

interface UseAsyncReturn<T> extends AsyncState<T> {
  execute: (...args: unknown[]) => Promise<T | null>;
  reset: () => void;
}

/** Wraps an async function with loading/error/data state. */
export function useAsync<T>(
  asyncFn: (...args: unknown[]) => Promise<T>,
): UseAsyncReturn<T> {
  const [state, setState] = useState<AsyncState<T>>({
    data: null,
    error: null,
    isLoading: false,
  });

  // Track mounted state to avoid setting state after unmount
  const mountedRef = useRef(true);
  // useRef to keep asyncFn stable across renders
  const fnRef = useRef(asyncFn);
  fnRef.current = asyncFn;

  const execute = useCallback(async (...args: unknown[]): Promise<T | null> => {
    setState({ data: null, error: null, isLoading: true });
    try {
      const data = await fnRef.current(...args);
      if (mountedRef.current) setState({ data, error: null, isLoading: false });
      return data;
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      if (mountedRef.current) setState({ data: null, error, isLoading: false });
      return null;
    }
  }, []);

  const reset = useCallback(() => {
    setState({ data: null, error: null, isLoading: false });
  }, []);

  return { ...state, execute, reset };
}
