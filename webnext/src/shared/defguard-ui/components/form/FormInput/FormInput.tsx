import { useStore } from '@tanstack/react-form';
import { useMemo } from 'react';
import type { z } from 'zod';
import { useFieldContext, useFormContext } from '../../../form';
import { isPresent } from '../../../utils/isPresent';
import { Input } from '../../Input/Input';
import type { FormInputProps } from '../../Input/types';

export const FormInput = ({ mapError, ...props }: FormInputProps) => {
  const field = useFieldContext<string>();
  const form = useFormContext();

  // allows field to show error even if isPristine is true, this is needed in cases as input required or checkbox checked but user just clicked submit
  const wasSubmittedWithFailure = useStore(
    form.store,
    (store) => !store.isSubmitSuccessful,
  );

  const isPristine = useStore(field.store, (state) => state.meta.isPristine);

  const errorState = useStore(
    field.store,
    // normally this should be ZodIssue but sometime's we want to add some post submit validation and there we probably want to set only error messages
    (state) => state.meta.errors as Array<z.core.$ZodIssue | string>,
  );

  const errorMessage = useMemo(() => {
    // ignore errors unless some touches the field or submit's the form
    if (isPristine && !wasSubmittedWithFailure) return undefined;

    const fieldError = errorState[0];

    if (fieldError) {
      if (typeof fieldError === 'string') {
        if (isPresent(mapError)) {
          return mapError(fieldError);
        }
        return fieldError;
      } else {
        if (isPresent(mapError)) {
          return mapError(fieldError.message);
        }
        return fieldError.message;
      }
    }
    return undefined;
  }, [mapError, errorState[0], isPristine, wasSubmittedWithFailure]);

  return (
    <Input
      testId={`field-${field.name}`}
      onBlur={field.handleBlur}
      onChange={field.handleChange}
      value={field.state.value}
      error={errorMessage}
      {...props}
    />
  );
};
