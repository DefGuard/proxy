import './style.scss';

import classNames from 'classnames';
import { ReactNode } from 'react';

import { TimeLeft } from '../TimeLeft/TimeLeft';

type Props = {
  children: ReactNode;
  className?: string;
};

export const EnrollmentStepControls = ({ children, className }: Props) => {
  const cn = classNames('controls', className);

  return (
    <div className={cn}>
      <div className="actions">{children}</div>
      <div className="mobile-info">
        <div className="admin">PLACEHOLDER</div>
        <TimeLeft />
      </div>
    </div>
  );
};
