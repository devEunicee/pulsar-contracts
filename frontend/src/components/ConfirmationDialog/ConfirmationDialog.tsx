import React from "react";
import "./ConfirmationDialog.css";

export interface ConfirmationDialogProps {
  open: boolean;
  /** Main heading for the dialog */
  title: string;
  /** Description of the action and its consequences */
  description: string;
  /** Optional list of consequence bullets */
  consequences?: string[];
  /** Additional warning shown below the consequences */
  warning?: string;
  /** Label for the confirm button (default: "Confirm") */
  confirmLabel?: string;
  /** Label for the cancel button (default: "Cancel") */
  cancelLabel?: string;
  onConfirm: () => void;
  onCancel: () => void;
}

/**
 * Modal confirmation dialog for destructive actions.
 *
 * Features:
 * - Focus trapped inside the dialog while open
 * - Escape key cancels
 * - Backdrop click cancels
 * - role="alertdialog" with proper aria-labelledby / aria-describedby
 */
export function ConfirmationDialog({
  open,
  title,
  description,
  consequences,
  warning,
  confirmLabel = "Confirm",
  cancelLabel = "Cancel",
  onConfirm,
  onCancel,
}: ConfirmationDialogProps) {
  const dialogRef = React.useRef<HTMLDivElement>(null);
  const cancelBtnRef = React.useRef<HTMLButtonElement>(null);
  const titleId = React.useId();
  const descId = React.useId();

  // Focus the cancel button when dialog opens
  React.useEffect(() => {
    if (open) cancelBtnRef.current?.focus();
  }, [open]);

  // Trap focus and handle Escape
  React.useEffect(() => {
    if (!open) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        onCancel();
        return;
      }
      if (e.key !== "Tab") return;

      const dialog = dialogRef.current;
      if (!dialog) return;
      const focusable = dialog.querySelectorAll<HTMLElement>(
        'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
      );
      const first = focusable[0];
      const last = focusable[focusable.length - 1];

      if (e.shiftKey ? document.activeElement === first : document.activeElement === last) {
        e.preventDefault();
        (e.shiftKey ? last : first).focus();
      }
    };

    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [open, onCancel]);

  if (!open) return null;

  return (
    <div className="cd-backdrop" onClick={onCancel} aria-hidden="true">
      <div
        ref={dialogRef}
        role="alertdialog"
        aria-modal="true"
        aria-labelledby={titleId}
        aria-describedby={descId}
        className="cd-dialog"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="cd-header">
          <span className="cd-icon" aria-hidden="true">⚠</span>
          <h2 id={titleId} className="cd-title">{title}</h2>
        </div>

        <p id={descId} className="cd-description">{description}</p>

        {consequences && consequences.length > 0 && (
          <ul className="cd-consequences" aria-label="Consequences">
            {consequences.map((c) => (
              <li key={c}>{c}</li>
            ))}
          </ul>
        )}

        {warning && (
          <p className="cd-warning" role="note">{warning}</p>
        )}

        <div className="cd-actions">
          <button
            ref={cancelBtnRef}
            type="button"
            className="cd-btn cd-btn--cancel"
            onClick={onCancel}
          >
            {cancelLabel}
          </button>
          <button
            type="button"
            className="cd-btn cd-btn--confirm"
            onClick={onConfirm}
          >
            {confirmLabel}
          </button>
        </div>
      </div>
    </div>
  );
}
