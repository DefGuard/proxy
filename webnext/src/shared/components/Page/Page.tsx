import './style.scss';
import clsx from 'clsx';
import type { HTMLProps, PropsWithChildren } from 'react';

type Props = PropsWithChildren & HTMLProps<HTMLDivElement>;

export const Page = ({ className, children, ...rest }: Props) => {
  return (
    <div {...rest} className={clsx('page', className)}>
      <div className="content-track">
        <div className="content-limiter">
          <main className="page-content">{children}</main>
        </div>
      </div>
    </div>
  );
};
