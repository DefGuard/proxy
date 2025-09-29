import './style.scss';
import clsx from 'clsx';
import type { HTMLAttributes, PropsWithChildren, Ref } from 'react';

export const Tooltip = ({
  ref,
  children,
  className,
  ...rest
}: PropsWithChildren & {
  ref?: Ref<HTMLDivElement>;
} & HTMLAttributes<HTMLDivElement>) => {
  return (
    <div className={clsx('tooltip', className)} ref={ref} {...rest}>
      {children}
    </div>
  );
};
