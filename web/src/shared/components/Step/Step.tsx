import clsx from 'clsx';
import { m } from '../../../paraglide/messages';
import './style.scss';

type Props = {
  current: number;
  max: number;
};

export const EnrollmentStep = ({ current, max }: Props) => {
  const isFinal = current === max;

  return (
    <div
      className={clsx('enrollment-step', {
        final: isFinal,
      })}
      data-step-current={current}
      data-step-max={max}
    >
      <span>
        {isFinal
          ? m.cmp_enrol_final()
          : m.cmp_enrol_step({
              current: current + 1,
              max: max + 1,
            })}
      </span>
    </div>
  );
};
