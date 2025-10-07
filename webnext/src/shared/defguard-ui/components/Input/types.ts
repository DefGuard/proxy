import type { HTMLAttributes, HTMLInputAutoCompleteAttribute, Ref } from 'react';
import type { FieldBoxProps, FieldSize } from '../FieldBox/types';

export type InputProps = {
  value: string | null;
  size?: FieldSize;
  type?: 'password' | 'text';
  ref?: Ref<HTMLInputElement>;
  error?: string;
  name?: string;
  label?: string;
  required?: boolean;
  disabled?: boolean;
  placeholder?: string;
  onChange?: (value: string) => void;
  boxProps?: Partial<FieldBoxProps>;
  autocomplete?: HTMLInputAutoCompleteAttribute;
  testId?: string;
} & Pick<HTMLAttributes<HTMLInputElement>, 'onBlur' | 'onFocus'>;

export type FormInputProps = Pick<
  InputProps,
  'name' | 'placeholder' | 'disabled' | 'required' | 'label' | 'autocomplete' | 'type'
> & {
  mapError?: (error: string) => string | undefined;
};
