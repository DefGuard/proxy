import type { ButtonHTMLAttributes, HTMLAttributes, Ref } from 'react';
import type { Direction } from '../../types';
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
  iconRightRotation?: Direction;
  testId?: string;
  disabled?: boolean;
  loading?: boolean;
  ref?: Ref<HTMLButtonElement>;
} & HTMLAttributes<HTMLElement>;
