import type { Ref } from 'react';
import './style.scss';
import clsx from 'clsx';
import { ThemeSpacing } from '../../types';
import { isPresent } from '../../utils/isPresent';
import { Button } from '../Button/Button';
import type { ButtonProps } from '../Button/types';
import { Icon } from '../Icon';
import type { IconKindValue } from '../Icon/icon-types';
import { SizedBox } from '../SizedBox/SizedBox';

type Props = {
  ref?: Ref<HTMLDivElement>;
  title?: string;
  subtitle?: string;
  icon?: IconKindValue;
  className?: string;
  testId?: string;
  id?: string;
  primaryAction?: ButtonProps;
  secondaryAction?: () => void;
  secondaryActionText?: string;
};

//TODO: icon is incompatible, remove it when this will be needed
export const EmptyState = ({
  ref,
  icon,
  primaryAction,
  secondaryAction,
  secondaryActionText,
  subtitle,
  title,
  className,
  id,
  testId,
}: Props) => {
  return (
    <div
      ref={ref}
      className={clsx('empty-state', className)}
      id={id}
      data-testid={testId}
    >
      {isPresent(icon) && (
        <>
          <div className="empty-icon">
            <Icon icon={icon} size={24} />
          </div>
          <SizedBox height={ThemeSpacing.Lg} />
        </>
      )}
      {isPresent(title) && (
        <>
          <p className="title">{title}</p>
          <SizedBox height={4} />
        </>
      )}
      {isPresent(subtitle) && <p className="subtitle">{subtitle}</p>}
      <SizedBox height={ThemeSpacing.Lg} />
      {isPresent(primaryAction) && (
        <>
          <Button {...primaryAction} />
          <SizedBox height={ThemeSpacing.Lg} />
        </>
      )}
      {isPresent(secondaryAction) && isPresent(secondaryActionText) && (
        <button className="secondary-action" onClick={secondaryAction}>
          {secondaryActionText}
        </button>
      )}
    </div>
  );
};
