import { FieldValues, useController, UseControllerProps } from 'react-hook-form';

import { Toggle } from '../../layout/Toggle/Toggle';
import { ToggleProps } from '../../layout/Toggle/types';

interface Props<T extends FieldValues, D>
  extends Omit<ToggleProps<D>, 'onChange' | 'selected'> {
  controller: UseControllerProps<T>;
}

export const FormToggle = <T extends FieldValues, D>({
  controller,
  ...rest
}: Props<T, D>) => {
  const {
    field: { onChange, value },
  } = useController(controller);
  return <Toggle {...rest} selected={value} onChange={(v) => onChange(v)} />;
};
