import type { MouseEventHandler, Ref } from 'react';
import type { IconKindValue } from '../Icon/icon-types';

export type IconButtonProps = {
  icon: IconKindValue;
  disabled?: boolean;
  ref?: Ref<HTMLDivElement>;
  onClick?: MouseEventHandler<HTMLDivElement>;
};
