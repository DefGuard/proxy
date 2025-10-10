import './style.scss';
import clsx from 'clsx';
import type { HTMLProps, PropsWithChildren } from 'react';
import { BorderRadius, type BorderRadiusValue } from '../../defguard-ui/types';

type Props = {
  borderRadius?: BorderRadiusValue;
} & PropsWithChildren &
  HTMLProps<HTMLDivElement>;

export const Container = ({
  className,
  children,
  borderRadius = BorderRadius.Lg,
  ...rest
}: Props) => {
  return (
    <div
      className={clsx('container', 'standard', className)}
      style={{
        borderRadius: borderRadius,
      }}
      {...rest}
    >
      {children}
    </div>
  );
};
