import './style.scss';

import classNames from 'classnames';
import { HTMLProps, useMemo } from 'react';

import SvgIconCancel from '../../svg/IconCancel';

interface Props extends HTMLProps<HTMLDivElement> {
  onDispose?: () => void;
  disposable?: boolean;
  text: string;
}

export const Tag = ({ onDispose, disposable, text, className, ...rest }: Props) => {
  const cn = useMemo(
    () => classNames('tag', { disposable: disposable }, className),
    [disposable, className],
  );

  return (
    <div className={cn} {...rest}>
      <span>{text}</span>
      {disposable && (
        <button
          className="dispose"
          onClick={() => {
            onDispose?.();
          }}
        >
          <SvgIconCancel />
        </button>
      )}
    </div>
  );
};
