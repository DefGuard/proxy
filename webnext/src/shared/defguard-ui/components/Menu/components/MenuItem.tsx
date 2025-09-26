import clsx from 'clsx';
import { isPresent } from '../../../utils/isPresent';
import { Icon } from '../../Icon';
import type { MenuItemProps } from '../types';

export const MenuItem = ({
  disabled,
  text,
  icon,
  items,
  onClick,
  variant,
}: MenuItemProps) => {
  const hasItems = isPresent(items) && items.length > 0;
  const hasIcon = isPresent(icon);

  return (
    <div
      className={clsx('menu-item', `variant-${variant}`, {
        disabled,
        'grid-default': !hasItems && !hasIcon,
        'grid-group': hasItems && !hasIcon,
        'grid-icon': !hasItems && hasIcon,
        'grid-full': hasIcon && hasItems,
        nested: hasItems,
      })}
      onClick={() => {
        if (!disabled) {
          onClick?.();
        }
      }}
    >
      {isPresent(icon) && <Icon icon={icon} size={20} />}
      <p>{text}</p>
      {hasItems && (
        <div className="suffix">
          <Icon icon="arrow-small" size={20} />
        </div>
      )}
    </div>
  );
};
