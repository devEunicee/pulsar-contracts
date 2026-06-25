import React from "react";

export interface DatePickerProps
  extends React.InputHTMLAttributes<HTMLInputElement> {
  label: string;
  includeTime?: boolean;
  helpText?: string;
  error?: string;
}

export const DatePicker: React.FC<DatePickerProps> = ({
  label,
  includeTime = false,
  helpText,
  error,
  id,
  ...props
}) => {
  const fieldId = id ?? label.toLowerCase().replace(/\s+/g, "-");
  return (
    <div className="form-field">
      <label htmlFor={fieldId}>{label}</label>
      <input
        id={fieldId}
        type={includeTime ? "datetime-local" : "date"}
        aria-describedby={error ? `${fieldId}-error` : helpText ? `${fieldId}-help` : undefined}
        aria-invalid={!!error}
        {...props}
      />
      {helpText && !error && <p id={`${fieldId}-help`} className="help-text">{helpText}</p>}
      {error && <p id={`${fieldId}-error`} className="error-text" role="alert">{error}</p>}
    </div>
  );
};
