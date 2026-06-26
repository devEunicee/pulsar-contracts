import { useState, useCallback } from 'react';

/** Persists state to localStorage, synced across tabs via storage event. */
export function useLocalStorage<T>(key: string, initialValue: T): [T, (v: T) => void, () => void] {
  const [value, setValue] = useState<T>(() => {
    try {
      const item = localStorage.getItem(key);
      return item !== null ? (JSON.parse(item) as T) : initialValue;
    } catch {
      return initialValue;
    }
  });

  const set = useCallback(
    (v: T) => {
      setValue(v);
      try {
        localStorage.setItem(key, JSON.stringify(v));
      } catch {
        // quota exceeded — silently ignore
      }
    },
    [key],
  );

  const remove = useCallback(() => {
    setValue(initialValue);
    localStorage.removeItem(key);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [key]);

  return [value, set, remove];
}
