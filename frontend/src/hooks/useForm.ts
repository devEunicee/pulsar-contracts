import { useState, useCallback, type ChangeEvent } from 'react';

type Validator<T> = (value: T[keyof T], values: T) => string | undefined;
type Validators<T> = Partial<Record<keyof T, Validator<T>>>;

interface UseFormReturn<T> {
  values: T;
  errors: Partial<Record<keyof T, string>>;
  touched: Partial<Record<keyof T, boolean>>;
  isValid: boolean;
  handleChange: (e: ChangeEvent<HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement>) => void;
  handleBlur: (e: ChangeEvent<HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement>) => void;
  setFieldValue: (field: keyof T, value: T[keyof T]) => void;
  reset: () => void;
}

/** Form state manager with per-field validation. */
export function useForm<T extends Record<string, unknown>>(
  initialValues: T,
  validators: Validators<T> = {},
): UseFormReturn<T> {
  const [values, setValues] = useState<T>(initialValues);
  const [errors, setErrors] = useState<Partial<Record<keyof T, string>>>({});
  const [touched, setTouched] = useState<Partial<Record<keyof T, boolean>>>({});

  const validate = useCallback(
    (field: keyof T, value: T[keyof T], currentValues: T) => {
      const validator = validators[field];
      return validator ? validator(value, currentValues) : undefined;
    },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [],
  );

  const setFieldValue = useCallback(
    (field: keyof T, value: T[keyof T]) => {
      const next = { ...values, [field]: value };
      setValues(next);
      const err = validate(field, value, next);
      setErrors((prev) => ({ ...prev, [field]: err }));
    },
    [values, validate],
  );

  const handleChange = useCallback(
    (e: ChangeEvent<HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement>) => {
      const { name, value } = e.target;
      setFieldValue(name as keyof T, value as T[keyof T]);
    },
    [setFieldValue],
  );

  const handleBlur = useCallback(
    (e: ChangeEvent<HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement>) => {
      const { name } = e.target;
      setTouched((prev) => ({ ...prev, [name]: true }));
    },
    [],
  );

  const reset = useCallback(() => {
    setValues(initialValues);
    setErrors({});
    setTouched({});
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const isValid =
    Object.values(errors).every((e) => !e) &&
    Object.keys(validators).every((k) => !validate(k as keyof T, values[k as keyof T], values));

  return { values, errors, touched, isValid, handleChange, handleBlur, setFieldValue, reset };
}
