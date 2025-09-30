import './style.scss';
import clsx from 'clsx';
import { isPresent } from '../../utils/isPresent';
import { Icon } from '../Icon';
import { InteractionBox } from '../InteractionBox/InteractionBox';
import type { FieldBoxProps } from './types';

// generalized field box for components like Input, shouldn't be in layout on it's own
export const FieldBox = ({
  children,
  disabled,
  error,
  className,
  boxRef,
  interactionRef,
  iconLeft,
  iconRight,
  size,
  onInteractionClick,
  ...rest
}: FieldBoxProps) => {
  const hasIconLeft = isPresent(iconLeft);
  const hasIconRight = isPresent(iconRight);
  return (
    <div
      className={clsx('field-box', className, `size-${size}`, {
        'grid-default': !hasIconLeft && !hasIconRight,
        'grid-left': hasIconLeft && !hasIconRight,
        'grid-right': hasIconRight && !hasIconLeft,
        'grid-both': hasIconLeft && hasIconRight,
        disabled,
        error,
      })}
      {...rest}
    >
      {hasIconLeft && <Icon icon={iconLeft} size={20} />}
      {children}
      {hasIconRight && (
        <InteractionBox
          disabled={disabled}
          iconSize={20}
          icon={iconRight}
          onClick={onInteractionClick}
          tabIndex={-1}
        />
      )}
    </div>
  );
};
