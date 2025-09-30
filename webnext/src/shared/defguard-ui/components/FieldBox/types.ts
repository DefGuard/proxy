import type { HTMLAttributes, MouseEventHandler, PropsWithChildren, Ref } from 'react';
import type { IconKindValue } from '../Icon/icon-types';

export type FieldSize = 'lg' | 'default';

export interface FieldBoxProps extends HTMLAttributes<HTMLDivElement>, PropsWithChildren {
  boxRef?: Ref<HTMLDivElement>;
  interactionRef?: Ref<HTMLButtonElement>;
  error?: boolean;
  disabled?: boolean;
  iconLeft?: IconKindValue;
  iconRight?: IconKindValue;
  size?: FieldSize;
  onInteractionClick?: MouseEventHandler<HTMLButtonElement>;
}
