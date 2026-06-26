/**
 * Global toast/alert store.
 * Toasts are priority-ordered (errors first), auto-dismissed, and persisted
 * across route changes. Accessible from any component via useToast.
 */

export type ToastPriority = 'error' | 'warning' | 'success' | 'info';

export interface Toast {
  id: string;
  message: string;
  priority: ToastPriority;
  /** Auto-dismiss duration in ms. 0 = no auto-dismiss. */
  duration: number;
}

type Listener = (toasts: Toast[]) => void;

// Priority rank — lower = shown first
const PRIORITY_RANK: Record<ToastPriority, number> = {
  error: 0,
  warning: 1,
  success: 2,
  info: 3,
};

let counter = 0;
function generateId(): string {
  return `toast-${Date.now()}-${++counter}`;
}

class ToastStore {
  private toasts: Toast[] = [];
  private listeners = new Set<Listener>();
  private timers = new Map<string, ReturnType<typeof setTimeout>>();

  getToasts(): Toast[] {
    return this.toasts;
  }

  subscribe(listener: Listener): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  private notify(): void {
    this.listeners.forEach((l) => l(this.toasts));
  }

  add(
    message: string,
    priority: ToastPriority = 'info',
    duration = 4000,
  ): string {
    const id = generateId();
    const toast: Toast = { id, message, priority, duration };

    // Insert in priority order
    const idx = this.toasts.findIndex(
      (t) => PRIORITY_RANK[t.priority] > PRIORITY_RANK[priority],
    );
    if (idx === -1) {
      this.toasts = [...this.toasts, toast];
    } else {
      this.toasts = [
        ...this.toasts.slice(0, idx),
        toast,
        ...this.toasts.slice(idx),
      ];
    }

    if (duration > 0) {
      this.timers.set(id, setTimeout(() => this.remove(id), duration));
    }

    this.notify();
    return id;
  }

  remove(id: string): void {
    const timer = this.timers.get(id);
    if (timer) {
      clearTimeout(timer);
      this.timers.delete(id);
    }
    this.toasts = this.toasts.filter((t) => t.id !== id);
    this.notify();
  }

  clear(): void {
    this.timers.forEach((t) => clearTimeout(t));
    this.timers.clear();
    this.toasts = [];
    this.notify();
  }
}

export const toastStore = new ToastStore();
