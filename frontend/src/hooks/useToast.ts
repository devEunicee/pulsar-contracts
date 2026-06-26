import { useState, useEffect, useCallback } from 'react';
import { toastStore, type Toast, type ToastPriority } from '../store/toastStore';

interface UseToastReturn {
  toasts: Toast[];
  add: (message: string, priority?: ToastPriority, duration?: number) => string;
  remove: (id: string) => void;
  clear: () => void;
  /** Convenience shortcuts */
  success: (message: string, duration?: number) => string;
  error: (message: string, duration?: number) => string;
  warning: (message: string, duration?: number) => string;
  info: (message: string, duration?: number) => string;
}

/** Access the global toast store from any component. */
export function useToast(): UseToastReturn {
  const [toasts, setToasts] = useState<Toast[]>(() => toastStore.getToasts());

  useEffect(() => toastStore.subscribe(setToasts), []);

  const add = useCallback(
    (message: string, priority?: ToastPriority, duration?: number) =>
      toastStore.add(message, priority, duration),
    [],
  );

  const remove = useCallback((id: string) => toastStore.remove(id), []);
  const clear = useCallback(() => toastStore.clear(), []);

  return {
    toasts,
    add,
    remove,
    clear,
    success: useCallback((m, d) => toastStore.add(m, 'success', d), []),
    error: useCallback((m, d) => toastStore.add(m, 'error', d), []),
    warning: useCallback((m, d) => toastStore.add(m, 'warning', d), []),
    info: useCallback((m, d) => toastStore.add(m, 'info', d), []),
  };
}
