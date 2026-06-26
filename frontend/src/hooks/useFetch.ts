import { useEffect, useRef } from 'react';
import { useAsync } from './useAsync';

interface FetchOptions extends RequestInit {
  skip?: boolean;
}

/** Fetch wrapper with loading/error/data state. Re-fetches on url change. */
export function useFetch<T>(url: string, options: FetchOptions = {}) {
  const { skip, ...fetchOptions } = options;
  const { execute, ...state } = useAsync<T>(async () => {
    const res = await fetch(url, fetchOptions);
    if (!res.ok) throw new Error(`HTTP ${res.status}: ${res.statusText}`);
    return res.json() as Promise<T>;
  });

  const optionsRef = useRef(fetchOptions);
  optionsRef.current = fetchOptions;

  useEffect(() => {
    if (!skip) execute();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [url, skip]);

  return { ...state, refetch: execute };
}
