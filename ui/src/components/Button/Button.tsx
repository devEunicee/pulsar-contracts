import React from "react";

export type ButtonVariant = "primary" | "secondary" | "danger";
export type ButtonSize = "sm" | "md" | "lg";

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  /** Visual style of the button */
  variant?: ButtonVariant;
  /** Size of the button */
  size?: ButtonSize;
  /** Show loading spinner and disable interaction */
  loading?: boolean;
  /** Button label */
  children: React.ReactNode;
}

const variantStyles: Record<ButtonVariant, string> = {
  primary: "background:#2563eb;color:#fff;border:none",
  secondary: "background:#e5e7eb;color:#111827;border:1px solid #d1d5db",
  danger: "background:#dc2626;color:#fff;border:none",
};

const sizeStyles: Record<ButtonSize, string> = {
  sm: "padding:4px 12px;font-size:0.75rem;border-radius:4px",
  md: "padding:8px 16px;font-size:0.875rem;border-radius:6px",
  lg: "padding:12px 24px;font-size:1rem;border-radius:8px",
};

/**
 * Primary UI button for user interactions.
 */
export const Button: React.FC<ButtonProps> = ({
  variant = "primary",
  size = "md",
  loading = false,
  children,
  disabled,
  style,
  ...rest
}) => {
  const inlineStyle = [variantStyles[variant], sizeStyles[size], "cursor:pointer"]
    .join(";")
    .split(";")
    .reduce<React.CSSProperties>((acc, rule) => {
      const [k, v] = rule.split(":").map((s) => s.trim());
      if (k && v) (acc as Record<string, string>)[k] = v;
      return acc;
    }, {});

  return (
    <button
      {...rest}
      disabled={disabled || loading}
      aria-busy={loading}
      style={{ ...inlineStyle, opacity: disabled || loading ? 0.6 : 1, ...style }}
    >
      {loading ? "Loading…" : children}
    </button>
  );
};
