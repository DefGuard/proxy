import type { Placement } from '@floating-ui/react';
import type { RefObject } from 'react';
import type { IconKindValue } from '../Icon/icon-types';

export interface MenuProps {
  itemGroups: MenuItemsGroup[];
  referenceRef: RefObject<HTMLElement | null>;
  placement?: Placement;
  isOpen: boolean;
  setOpen: (val: boolean) => void;
  floatingOffset?: number;
}

export interface MenuItemsGroup {
  header?: MenuHeaderProps;
  items: MenuItemProps[];
}

export interface MenuItemProps {
  text: string;
  variant?: 'default' | 'danger';
  icon?: IconKindValue;
  items?: MenuItemProps[];
  disabled?: boolean;
  onClick?: () => void;
}

export interface MenuHeaderProps {
  text: string;
  onHelp?: () => void;
}
