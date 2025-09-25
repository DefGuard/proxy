import './style.scss';
import clsx from 'clsx';
import type { ReactNode } from 'react';
import { m } from '../../../../paraglide/messages';
import { isPresent } from '../../utils/isPresent';
import { Button } from '../Button/Button';
import type { ButtonProps } from '../Button/types';

type Props = {
  submitProps?: ButtonProps;
  cancelProps?: ButtonProps;
  children?: ReactNode;
};

export const ModalControls = ({ submitProps, cancelProps, children }: Props) => {
  return (
    <div
      className={clsx('modal-controls', {
        extras: isPresent(children),
      })}
    >
      {isPresent(children) && <div className="extras">{children}</div>}
      <div className="buttons">
        <Button
          {...cancelProps}
          variant={cancelProps?.variant ?? 'secondary'}
          text={cancelProps?.text ?? m.controls_cancel()}
        />
        <Button
          {...submitProps}
          variant={cancelProps?.variant ?? 'primary'}
          text={cancelProps?.text ?? m.controls_submit()}
        />
      </div>
    </div>
  );
};
