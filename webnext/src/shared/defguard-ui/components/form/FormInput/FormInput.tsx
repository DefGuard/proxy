import { useMemo } from 'react';
import type { ZodError } from 'zod';
import { useFieldContext } from '../../../form';
import { Input } from '../../Input/Input';
import type { FormInputProps } from '../../Input/types';

export const FormInput = (props: FormInputProps) => {
  const field = useFieldContext<string>();

  const errorMessage = useMemo(() => {
    const fieldZodError = field.state.meta.errors[0] as ZodError | undefined;
    if (fieldZodError) {
      return fieldZodError.message;
    }
    return undefined;
  }, [field.state.meta.errors]);

  return (
    <Input
      onBlur={field.handleBlur}
      onChange={field.handleChange}
      value={field.state.value}
      canError
      error={errorMessage}
      {...props}
    />
  );
};
