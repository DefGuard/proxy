import './style.scss';
import clsx from 'clsx';
import type { HTMLProps, PropsWithChildren } from 'react';
import { Logo } from '../Logo/Logo';

type Variant = 'home' | 'default';

type Props = PropsWithChildren &
  HTMLProps<HTMLDivElement> & {
    variant?: Variant;
    nav?: boolean;
  };

export const Page = ({ className, children, variant, nav = false, ...rest }: Props) => {
  return (
    <div
      {...rest}
      className={clsx('page', className, `variant-${variant}`, {
        nav,
      })}
    >
      <div className="content-track">
        <div className="content-limiter">
          <main className="page-content">
            <Logo />
            {children}
          </main>
        </div>
      </div>
    </div>
  );
};
