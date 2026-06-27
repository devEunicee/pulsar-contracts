import React from "react";

export interface CheckboxOption {
  value: string;
  label: string;
}

export interface CheckboxGroupProps {
  legend: string;
  options: CheckboxOption[];
  value: string[];
  onChange: (value: string[]) => void;
  helpText?: string;
  error?: string;
}

export const CheckboxGroup: React.FC<CheckboxGroupProps> = ({
  legend,
  options,
  value,
  onChange,
  helpText,
  error,
}) => {
  const toggle = (v: string) =>
    onChange(value.includes(v) ? value.filter((x) => x !== v) : [...value, v]);

  return (
    <fieldset>
      <legend>{legend}</legend>
      {options.map((opt) => (
        <label key={opt.value} className="checkbox-label">
          <input
            type="checkbox"
            value={opt.value}
            checked={value.includes(opt.value)}
            onChange={() => toggle(opt.value)}
          />
          {opt.label}
        </label>
      ))}
      {helpText && !error && <p className="help-text">{helpText}</p>}
      {error && <p className="error-text" role="alert">{error}</p>}
    </fieldset>
  );
};
