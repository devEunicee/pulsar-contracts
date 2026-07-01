import React from "react";
import type { ToastPriority } from "../../store/toastStore";
import "./Toast.css";

export interface ToastProps {
  id: string;
  message: string;
  priority: ToastPriority;
  onDismiss: (id: string) => void;
}

const ICONS: Record<ToastPriority, string> = {
  error: "✕",
  warning: "⚠",
  success: "✓",
  info: "ℹ",
};

const PRIORITY_LABELS: Record<ToastPriority, string> = {
  error: "Error",
  warning: "Warning",
  success: "Success",
  info: "Information",
};

function getA11yRole(priority: ToastPriority): "alert" | "status" {
  return priority === "error" || priority === "warning" ? "alert" : "status";
}

function getLiveRegion(priority: ToastPriority): "assertive" | "polite" {
  return priority === "error" || priority === "warning" ? "assertive" : "polite";
}

export function Toast({ id, message, priority, onDismiss }: ToastProps) {
  return (
    <div
      className={`toast toast--${priority}`}
      role={getA11yRole(priority)}
      aria-live={getLiveRegion(priority)}
      aria-atomic="true"
    >
      <span className="toast__icon" aria-hidden="true">
        {ICONS[priority]}
      </span>
      <p className="toast__message">
        <span className="toast__sr-only">{PRIORITY_LABELS[priority]}: </span>
        {message}
      </p>
      <button
        type="button"
        className="toast__dismiss"
        aria-label={`Dismiss ${PRIORITY_LABELS[priority].toLowerCase()} notification`}
        onClick={() => onDismiss(id)}
      >
        <span aria-hidden="true">×</span>
      </button>
    </div>
  );
}
