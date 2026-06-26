import { create } from "zustand";
import { devtools } from "zustand/middleware";

type ToastType = "success" | "error" | "info";

export interface Toast {
  id: string;
  message: string;
  type: ToastType;
}

interface UIState {
  toasts: Toast[];
  isSidebarOpen: boolean;
  addToast: (message: string, type?: ToastType) => void;
  removeToast: (id: string) => void;
  toggleSidebar: () => void;
  setSidebarOpen: (open: boolean) => void;
}

export const useUIStore = create<UIState>()(
  devtools(
    (set) => ({
      toasts: [],
      isSidebarOpen: true,
      addToast: (message, type = "info") =>
        set((s) => ({
          toasts: [...s.toasts, { id: crypto.randomUUID(), message, type }],
        })),
      removeToast: (id) =>
        set((s) => ({ toasts: s.toasts.filter((t) => t.id !== id) })),
      toggleSidebar: () =>
        set((s) => ({ isSidebarOpen: !s.isSidebarOpen })),
      setSidebarOpen: (isSidebarOpen) => set({ isSidebarOpen }),
    }),
    { name: "ui" }
  )
);

// Selectors
export const selectToasts = (s: UIState) => s.toasts;
export const selectIsSidebarOpen = (s: UIState) => s.isSidebarOpen;
