import React from "react";
import { useToast } from "../../hooks/useToast";
import { Toast } from "./Toast";
import "./Toast.css";

/**
 * Fixed toast stack rendered at the app root.
 * Subscribes to the global toast store and persists across route changes.
 */
export function ToastContainer() {
  const { toasts, remove } = useToast();

  if (toasts.length === 0) return null;

  return (
    <div
      className="toast-container"
      aria-label="Notifications"
      role="region"
    >
      {toasts.map((toast) => (
        <Toast
          key={toast.id}
          id={toast.id}
          message={toast.message}
          priority={toast.priority}
          onDismiss={remove}
        />
      ))}
    </div>
  );
}
