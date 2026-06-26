import React from "react";

export interface RadioOption {
  value: string;
  label: string;
}

export interface RadioGroupProps {
  legend: string;
  name: string;
  options: RadioOption[];
  value: string;
  onChange: (value: string) => void;
  helpText?: string;
  error?: string;
}

export const RadioGroup: React.FC<RadioGroupProps> = ({
  legend,
  name,
  options,
  value,
  onChange,
  helpText,
  error,
}) => (
  <fieldset>
    <legend>{legend}</legend>
    {options.map((opt) => (
      <label key={opt.value} className="radio-label">
        <input
          type="radio"
          name={name}
          value={opt.value}
          checked={value === opt.value}
          onChange={() => onChange(opt.value)}
        />
        {opt.label}
      </label>
    ))}
    {helpText && !error && <p className="help-text">{helpText}</p>}
    {error && <p className="error-text" role="alert">{error}</p>}
  </fieldset>
);
