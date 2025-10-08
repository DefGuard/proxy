import './style.scss';
import clsx from 'clsx';
import { Icon } from '../Icon/Icon';
import type { IconButtonProps } from './types';

export const IconButton = ({ icon, ref, disabled = false, onClick }: IconButtonProps) => {
  return (
    <div
      ref={ref}
      className={clsx('icon-button', {
        disabled,
      })}
      onClick={(e) => {
        if (!disabled) {
          onClick?.(e);
        }
      }}
    >
      <Icon icon={icon} size={20} />
    </div>
  );
};
