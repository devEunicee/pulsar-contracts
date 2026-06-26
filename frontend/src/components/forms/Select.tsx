import React from "react";

export interface SelectOption {
  value: string;
  label: string;
}

export interface SelectProps
  extends React.SelectHTMLAttributes<HTMLSelectElement> {
  label: string;
  options: SelectOption[];
  helpText?: string;
  error?: string;
}

export const Select: React.FC<SelectProps> = ({
  label,
  options,
  helpText,
  error,
  id,
  ...props
}) => {
  const fieldId = id ?? label.toLowerCase().replace(/\s+/g, "-");
  return (
    <div className="form-field">
      <label htmlFor={fieldId}>{label}</label>
      <select id={fieldId} aria-describedby={error ? `${fieldId}-error` : helpText ? `${fieldId}-help` : undefined} aria-invalid={!!error} {...props}>
        {options.map((opt) => (
          <option key={opt.value} value={opt.value}>{opt.label}</option>
        ))}
      </select>
      {helpText && !error && <p id={`${fieldId}-help`} className="help-text">{helpText}</p>}
      {error && <p id={`${fieldId}-error`} className="error-text" role="alert">{error}</p>}
    </div>
  );
};
