import type { ButtonHTMLAttributes, Ref } from 'react';
import type { IconKindValue } from '../Icon/icon-types';

type DefaultButtonProps = ButtonHTMLAttributes<HTMLButtonElement>;

type ButtonVariant = 'primary' | 'secondary' | 'critical' | 'outlined';

type ButtonSize = 'primary' | 'big';

export type ButtonProps = {
  text: string;
  variant?: ButtonVariant;
  size?: ButtonSize;
  type?: DefaultButtonProps['type'];
  iconLeft?: IconKindValue;
  iconRight?: IconKindValue;
  testId?: string;
  disabled?: boolean;
  loading?: boolean;
  onClick?: () => void;
  ref?: Ref<HTMLButtonElement>;
};
