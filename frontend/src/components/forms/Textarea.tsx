import React from "react";

export interface TextareaProps
  extends React.TextareaHTMLAttributes<HTMLTextAreaElement> {
  label: string;
  helpText?: string;
  error?: string;
}

export const Textarea: React.FC<TextareaProps> = ({
  label,
  helpText,
  error,
  id,
  ...props
}) => {
  const fieldId = id ?? label.toLowerCase().replace(/\s+/g, "-");
  return (
    <div className="form-field">
      <label htmlFor={fieldId}>{label}</label>
      <textarea
        id={fieldId}
        aria-describedby={error ? `${fieldId}-error` : helpText ? `${fieldId}-help` : undefined}
        aria-invalid={!!error}
        {...props}
      />
      {helpText && !error && <p id={`${fieldId}-help`} className="help-text">{helpText}</p>}
      {error && <p id={`${fieldId}-error`} className="error-text" role="alert">{error}</p>}
    </div>
  );
};
