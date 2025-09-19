import './style.scss';
import clsx from 'clsx';
import type { HTMLProps, PropsWithChildren } from 'react';

type Variant = 'home' | 'default';

type Props = PropsWithChildren &
  HTMLProps<HTMLDivElement> & {
    variant?: Variant;
  };

export const Page = ({ className, children, variant, ...rest }: Props) => {
  return (
    <div {...rest} className={clsx('page', className, `variant-${variant}`)}>
      <div className="content-track">
        <div className="content-limiter">
          <main className="page-content">{children}</main>
        </div>
      </div>
    </div>
  );
};
