import type { HTMLAttributes, HTMLInputTypeAttribute, Ref } from 'react';

export type InputProps = {
  value: string | null;
  size?: 'default' | 'big';
  type?: HTMLInputTypeAttribute;
  ref?: Ref<HTMLInputElement>;
  error?: string;
  name?: string;
  label?: string;
  required?: boolean;
  disabled?: boolean;
  placeholder?: string;
  onChange?: (value: string) => void;
} & Pick<HTMLAttributes<HTMLInputElement>, 'onBlur' | 'onFocus'>;

export type FormInputProps = Pick<
  InputProps,
  'name' | 'placeholder' | 'disabled' | 'required' | 'label'
> & {
  mapError?: (error: string) => string | undefined;
};
