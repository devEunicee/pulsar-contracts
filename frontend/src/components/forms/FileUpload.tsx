import React from "react";

export interface FileUploadProps {
  label: string;
  accept?: string;
  multiple?: boolean;
  onChange: (files: FileList | null) => void;
  helpText?: string;
  error?: string;
}

export const FileUpload: React.FC<FileUploadProps> = ({
  label,
  accept,
  multiple = false,
  onChange,
  helpText,
  error,
}) => {
  const fieldId = label.toLowerCase().replace(/\s+/g, "-");
  return (
    <div className="form-field">
      <label htmlFor={fieldId}>{label}</label>
      <input
        id={fieldId}
        type="file"
        accept={accept}
        multiple={multiple}
        onChange={(e) => onChange(e.target.files)}
        aria-describedby={error ? `${fieldId}-error` : helpText ? `${fieldId}-help` : undefined}
        aria-invalid={!!error}
      />
      {helpText && !error && <p id={`${fieldId}-help`} className="help-text">{helpText}</p>}
      {error && <p id={`${fieldId}-error`} className="error-text" role="alert">{error}</p>}
    </div>
  );
};
