import type { MouseEventHandler, Ref } from 'react';
import { Icon } from '../Icon';
import type { IconKindValue } from '../Icon/icon-types';
import './style.scss';
import clsx from 'clsx';

type Props = {
  iconSize: number;
  icon: IconKindValue;
  onClick?: MouseEventHandler<HTMLButtonElement>;
  id?: string;
  className?: string;
  ref?: Ref<HTMLDivElement>;
  disabled?: boolean;
  tabIndex?: number;
};

export const InteractionBox = ({
  iconSize,
  icon,
  onClick,
  className,
  id,
  ref,
  tabIndex,
  disabled = false,
}: Props) => {
  return (
    <div className={clsx('interaction-box', className)} ref={ref} id={id}>
      <Icon icon={icon} size={iconSize} />
      <button
        type="button"
        onClick={onClick}
        disabled={disabled}
        tabIndex={tabIndex}
        onMouseDown={(e) => {
          e.preventDefault();
        }}
      ></button>
    </div>
  );
};
