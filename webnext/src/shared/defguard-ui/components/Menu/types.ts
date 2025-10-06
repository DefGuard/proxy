import type { HTMLAttributes, Ref } from 'react';
import type { IconKindValue } from '../Icon/icon-types';

export interface MenuProps extends HTMLAttributes<HTMLDivElement> {
  itemGroups: MenuItemsGroup[];
  onClose?: () => void;
  ref?: Ref<HTMLDivElement>;
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
  onClose?: () => void;
}

export interface MenuHeaderProps {
  text: string;
  tooltip?: string;
  onClose?: () => void;
  onHelp?: () => void;
}
