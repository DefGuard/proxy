import { useStore } from '@tanstack/react-form';
import { useMemo } from 'react';
import type { z } from 'zod';
import { useFieldContext } from '../../../form';
import { isPresent } from '../../../utils/isPresent';
import { Input } from '../../Input/Input';
import type { FormInputProps } from '../../Input/types';

export const FormInput = ({ mapError, ...props }: FormInputProps) => {
  const field = useFieldContext<string>();

  const isPristine = useStore(field.store, (state) => state.meta.isPristine);

  const errorState = useStore(
    field.store,
    (state) => state.meta.errors as z.core.$ZodIssue[],
  );

  const errorMessage = useMemo(() => {
    // ignore errors unless some touches the field or submit's the form
    if (isPristine) return undefined;

    const fieldZodError = errorState[0];

    if (fieldZodError) {
      if (isPresent(mapError)) {
        return mapError(fieldZodError.message);
      }
      return fieldZodError.message;
    }
    return undefined;
  }, [mapError, errorState[0], isPristine]);

  return (
    <Input
      onBlur={field.handleBlur}
      onChange={field.handleChange}
      value={field.state.value}
      error={errorMessage}
      {...props}
    />
  );
};
