import './style.scss';

import dayjs from 'dayjs';
import { useMemo } from 'react';

import { useI18nContext } from '../../../../i18n/i18n-react';
import { useEnrollmentStore } from '../../hooks/store/useEnrollmentStore';

type Props = {
  disableLabel?: boolean;
};

export const TimeLeft = ({ disableLabel }: Props) => {
  const { LL, locale } = useI18nContext();
  const sessionEnd = useEnrollmentStore((state) => state.sessionEnd);

  const dateDisplay = useMemo(() => {
    if (sessionEnd) {
      const now = dayjs();
      const endDay = dayjs(sessionEnd);
      const diff = endDay.diff(now, 'minute');
      return dayjs.duration(diff, 'minutes').locale(locale).humanize();
    }
    return 'not set';
  }, [locale, sessionEnd]);

  if (disableLabel) {
    return <span className="time-left solo">{dateDisplay}</span>;
  }

  return (
    <div className="time-left">
      <p>
        {LL.pages.enrollment.timeLeft()}: <span>{dateDisplay}</span>
      </p>
    </div>
  );
};
