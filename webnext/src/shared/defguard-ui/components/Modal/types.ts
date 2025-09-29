import type { ReactNode } from 'react';

export interface ModalProps {
  title: string;
  children: ReactNode;
  isOpen: boolean;
  hideBackdrop?: boolean;
  size?: 'small' | 'primary';
  onClose?: () => void;
  afterClose?: () => void;
  id?: string;
  positionerClassName?: string;
  contentClassName?: string;
}
