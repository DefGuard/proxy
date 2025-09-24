import clsx from 'clsx';
import './style.scss';
import type { Ref } from 'react';

type Variant = 'neutral' | 'default' | 'action' | 'critical' | 'warning';

type Props = {
  ref?: Ref<HTMLDivElement>;
  className?: string;
  testId?: string;
  id?: string;
  value?: number;
  variant?: Variant;
};

export const CounterLabel = ({ ref, className, id, testId, value, variant }: Props) => {
  return (
    <div
      className={clsx('counter-label', `variant-${variant}`, className)}
      id={id}
      data-testid={testId}
      ref={ref}
    >
      <span>{value ?? 0}</span>
    </div>
  );
};
