import './style.scss';
import clsx from 'clsx';
import type { Ref } from 'react';
import { Icon } from '../Icon/Icon';
import type { IconKindValue } from '../Icon/icon-types';

type Props = {
  icon: IconKindValue;
  disabled?: boolean;
  ref?: Ref<HTMLDivElement>;
  onClick?: () => void;
};

export const IconButton = ({ icon, ref, disabled = false, onClick }: Props) => {
  return (
    <div
      ref={ref}
      className={clsx('icon-button', {
        disabled,
      })}
      onClick={() => {
        if (!disabled) {
          onClick?.();
        }
      }}
    >
      <Icon icon={icon} size={20} />
    </div>
  );
};
