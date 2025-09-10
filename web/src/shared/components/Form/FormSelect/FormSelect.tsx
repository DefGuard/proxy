import { isUndefined } from 'lodash-es';
import { useMemo } from 'react';
import {
  type FieldValues,
  type UseControllerProps,
  useController,
} from 'react-hook-form';

import { Select } from '../../layout/Select/Select';
import type { SelectProps } from '../../layout/Select/types';

interface Props<T extends FieldValues, Y> extends SelectProps<Y> {
  controller: UseControllerProps<T>;
}

export const FormSelect = <T extends FieldValues, Y extends object>({
  controller,
  ...rest
}: Props<T, Y>) => {
  const {
    field,
    fieldState: { isDirty, isTouched, error },
    formState: { isSubmitted },
  } = useController(controller);

  const isInvalid = useMemo(() => {
    if (
      (!isUndefined(error) && (isDirty || isTouched)) ||
      (!isUndefined(error) && isSubmitted)
    ) {
      return true;
    }
    return false;
  }, [error, isDirty, isSubmitted, isTouched]);

  return (
    <Select
      {...rest}
      selected={field.value}
      invalid={isInvalid}
      errorMessage={error?.message}
      inForm={true}
      onChangeArray={(res) => field.onChange(res)}
      onChangeSingle={(res) => field.onChange(res)}
    />
  );
};
